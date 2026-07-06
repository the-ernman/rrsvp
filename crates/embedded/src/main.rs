mod board;
mod display;
mod touch;
mod ui;

use board::*;
use display::Display;
use touch::TouchDriver;
use ui::{draw, FocusMode, FontSize, OrpColor, Settings, UiState};

use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::*;

unsafe fn nvs_open_rsvp(write: bool) -> nvs_handle_t {
    let mut handle: nvs_handle_t = 0;
    let mode = if write {
        nvs_open_mode_t_NVS_READWRITE
    } else {
        nvs_open_mode_t_NVS_READONLY
    };
    let ret = nvs_open(b"rsvp\0".as_ptr() as _, mode, &mut handle);
    if ret != ESP_OK as i32 {
        log::warn!("[NVS] open failed ({}), returning 0", ret);
        return 0;
    }
    handle
}

unsafe fn load_settings(wpm_out: &mut u16) -> Settings {
    let handle = nvs_open_rsvp(false);
    let mut s = Settings::default();
    if handle == 0 {
        return s;
    }

    let mut v_u8: u8 = 0;
    let mut v_i32: i32 = 0;

    if nvs_get_u8(handle, b"dark\0".as_ptr() as _, &mut v_u8) == ESP_OK as i32 {
        s.dark_mode = v_u8 != 0;
    }
    if nvs_get_u8(handle, b"orp_col\0".as_ptr() as _, &mut v_u8) == ESP_OK as i32 {
        s.orp_color = OrpColor::from_u8(v_u8);
    }
    if nvs_get_u8(handle, b"font_sz\0".as_ptr() as _, &mut v_u8) == ESP_OK as i32 {
        s.font_size = FontSize::from_u8(v_u8);
    }
    if nvs_get_u8(handle, b"focus_md\0".as_ptr() as _, &mut v_u8) == ESP_OK as i32 {
        s.focus_mode = FocusMode::from_u8(v_u8);
    }
    if nvs_get_u8(handle, b"bar_col\0".as_ptr() as _, &mut v_u8) == ESP_OK as i32 {
        s.focus_bar_color = OrpColor::from_u8(v_u8);
    }
    if nvs_get_u8(handle, b"phantom_on\0".as_ptr() as _, &mut v_u8) == ESP_OK as i32 {
        s.phantom_words = v_u8 != 0;
    }
    if nvs_get_u8(handle, b"bright\0".as_ptr() as _, &mut v_u8) == ESP_OK as i32 {
        s.brightness = v_u8.clamp(1, 10);
    }
    if nvs_get_i32(handle, b"wpm\0".as_ptr() as _, &mut v_i32) == ESP_OK as i32 {
        *wpm_out = v_i32.clamp(10, 1000) as u16;
    }

    nvs_close(handle);
    log::info!("[NVS] settings loaded");
    s
}

unsafe fn save_settings(s: &Settings, wpm: u16) {
    let handle = nvs_open_rsvp(true);
    if handle == 0 {
        return;
    }

    nvs_set_u8(handle, b"dark\0".as_ptr() as _, s.dark_mode as u8);
    nvs_set_u8(handle, b"orp_col\0".as_ptr() as _, s.orp_color.to_u8());
    nvs_set_u8(handle, b"font_sz\0".as_ptr() as _, s.font_size.to_u8());
    nvs_set_u8(handle, b"focus_md\0".as_ptr() as _, s.focus_mode.to_u8());
    nvs_set_u8(
        handle,
        b"bar_col\0".as_ptr() as _,
        s.focus_bar_color.to_u8(),
    );
    nvs_set_u8(handle, b"phantom_on\0".as_ptr() as _, s.phantom_words as u8);
    nvs_set_u8(handle, b"bright\0".as_ptr() as _, s.brightness);
    nvs_set_i32(handle, b"wpm\0".as_ptr() as _, wpm as i32);

    let commit_ret = nvs_commit(handle);
    if commit_ret != ESP_OK as i32 {
        log::warn!(
            "[NVS] commit failed ({}), settings may not be persisted",
            commit_ret
        );
    }
    nvs_close(handle);
    log::info!("[NVS] settings saved");
}

unsafe fn apply_brightness(level: u8) {
    let level = level.clamp(1, 10);
    let duty = ((10u32 - level as u32) * 255 / 10) as u32;
    ledc_set_duty(
        ledc_mode_t_LEDC_LOW_SPEED_MODE,
        ledc_channel_t_LEDC_CHANNEL_0,
        duty,
    );
    ledc_update_duty(
        ledc_mode_t_LEDC_LOW_SPEED_MODE,
        ledc_channel_t_LEDC_CHANNEL_0,
    );
}

fn main() {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();
    log::info!("=== RRSVP booting on ESP32-S3-Touch-LCD-3.49 ===");

    unsafe {
        gpio_reset_pin(BTN_BOOT);
        gpio_set_direction(BTN_BOOT, gpio_mode_t_GPIO_MODE_INPUT);
        gpio_set_pull_mode(BTN_BOOT, gpio_pull_mode_t_GPIO_PULLUP_ONLY);

        gpio_reset_pin(BTN_POWER);
        gpio_set_direction(BTN_POWER, gpio_mode_t_GPIO_MODE_INPUT);
        gpio_set_pull_mode(BTN_POWER, gpio_pull_mode_t_GPIO_PULLUP_ONLY);
        log::info!(
            "buttons: GPIO{} (BOOT) and GPIO{} (POWER) configured as inputs",
            BTN_BOOT,
            BTN_POWER
        );

        let mut nvs_ret = nvs_flash_init();
        if nvs_ret == ESP_ERR_NVS_NO_FREE_PAGES as i32
            || nvs_ret == ESP_ERR_NVS_NEW_VERSION_FOUND as i32
        {
            log::warn!("[NVS] erasing and reinitialising NVS flash");
            nvs_flash_erase();
            nvs_ret = nvs_flash_init();
        }
        if nvs_ret != ESP_OK as i32 {
            log::error!("[NVS] nvs_flash_init failed: {}", nvs_ret);
        } else {
            log::info!("[NVS] flash init OK");
        }
    }

    log::info!("initialising display...");
    let mut display = Display::new();

    unsafe {
        let mut timer_cfg: ledc_timer_config_t = core::mem::zeroed();
        timer_cfg.speed_mode = ledc_mode_t_LEDC_LOW_SPEED_MODE;
        timer_cfg.timer_num = ledc_timer_t_LEDC_TIMER_0;
        timer_cfg.duty_resolution = ledc_timer_bit_t_LEDC_TIMER_8_BIT;
        timer_cfg.freq_hz = 5000;
        timer_cfg.clk_cfg = soc_periph_ledc_clk_src_legacy_t_LEDC_AUTO_CLK;
        let t_ret = ledc_timer_config(&timer_cfg);
        log::info!("[BL] ledc_timer_config ret={}", t_ret);

        let mut ch_cfg: ledc_channel_config_t = core::mem::zeroed();
        ch_cfg.gpio_num = LCD_BL;
        ch_cfg.speed_mode = ledc_mode_t_LEDC_LOW_SPEED_MODE;
        ch_cfg.channel = ledc_channel_t_LEDC_CHANNEL_0;
        ch_cfg.intr_type = ledc_intr_type_t_LEDC_INTR_DISABLE;
        ch_cfg.timer_sel = ledc_timer_t_LEDC_TIMER_0;
        ch_cfg.duty = 255;
        ch_cfg.hpoint = 0;
        let c_ret = ledc_channel_config(&ch_cfg);
        log::info!("[BL] ledc_channel_config ret={}", c_ret);
    }

    display.fill_solid(0xFFFF);
    std::thread::sleep(std::time::Duration::from_millis(300));

    log::info!("initialising touch...");
    let mut touch = TouchDriver::new();
    log::info!("touch init complete");

    let mut wpm_initial: u16 = 300;
    let initial_settings = unsafe { load_settings(&mut wpm_initial) };
    let mut state = UiState::with_settings(initial_settings, wpm_initial);
    let mut last_tp: Option<touch::TPoint> = None;
    let mut last_draw_ms: u32 = 0;
    let mut last_alive_ms: u32 = 0;
    let mut last_touch_action_ms: u32 = 0;
    const TOUCH_DEBOUNCE_MS: u32 = 350;
    let mut power_hold_start_ms: u32 = 0;
    let mut boot_was_pressed = false;
    let mut last_brightness = state.settings.brightness;
    let mut last_screen = state.screen;
    unsafe {
        apply_brightness(last_brightness);
    }

    log::info!("=== Entering main loop ===");

    loop {
        let now_ms = unsafe { (esp_idf_svc::sys::esp_timer_get_time() / 1000) as u32 };

        unsafe {
            let boot_low = gpio_get_level(BTN_BOOT) == 0;
            if boot_low && !boot_was_pressed {
                log::info!("[BTN] BOOT button pressed (GPIO{})", BTN_BOOT);
            } else if !boot_low && boot_was_pressed {
                log::info!("[BTN] BOOT button released");
            }
            boot_was_pressed = boot_low;

            let power_low = gpio_get_level(BTN_POWER) == 0;
            if power_low {
                if power_hold_start_ms == 0 {
                    power_hold_start_ms = now_ms;
                    log::info!("[BTN] POWER pressed — hold 3 s to power off");
                } else if now_ms.wrapping_sub(power_hold_start_ms) >= 3000 {
                    log::info!("[POWER] Holding POWER 3 s — entering deep sleep");
                    log::info!("[POWER] saving settings before sleep...");
                    save_settings(&state.settings, state.reader.wpm());
                    ledc_set_duty(
                        ledc_mode_t_LEDC_LOW_SPEED_MODE,
                        ledc_channel_t_LEDC_CHANNEL_0,
                        255,
                    );
                    ledc_update_duty(
                        ledc_mode_t_LEDC_LOW_SPEED_MODE,
                        ledc_channel_t_LEDC_CHANNEL_0,
                    );
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    let ret = esp_sleep_enable_ext0_wakeup(BTN_POWER as gpio_num_t, 0);
                    if ret != 0 {
                        log::warn!(
                            "[POWER] ext0 wakeup config failed ({}), sleeping anyway",
                            ret
                        );
                    }
                    esp_deep_sleep_start();
                }
            } else {
                if power_hold_start_ms != 0 {
                    log::info!("[BTN] POWER released (short press — no action)");
                }
                power_hold_start_ms = 0;
            }
        }

        let tp = touch.read();
        if tp != last_tp {
            match tp {
                Some(pt) => {
                    log::info!("[TOUCH] x={} y={}", pt.x, pt.y);
                    if now_ms.wrapping_sub(last_touch_action_ms) >= TOUCH_DEBOUNCE_MS {
                        state.handle_touch(tp, now_ms);
                        last_touch_action_ms = now_ms;
                    }
                }
                None => log::info!("[TOUCH] released"),
            }
            last_tp = tp;
        }

        if state.playing {
            state.reader.update(now_ms, true);
            if state.reader.at_end() {
                state.playing = false;
            }
        }

        if now_ms.wrapping_sub(last_draw_ms) >= 16 {
            draw(&mut display, &state, now_ms);
            display.flush();
            last_draw_ms = now_ms;
        }

        if state.settings.brightness != last_brightness {
            last_brightness = state.settings.brightness;
            unsafe {
                apply_brightness(last_brightness);
            }
        }

        if last_screen == ui::Screen::Settings && state.screen != ui::Screen::Settings {
            unsafe {
                save_settings(&state.settings, state.reader.wpm());
            }
        }
        last_screen = state.screen;

        if now_ms.wrapping_sub(last_alive_ms) >= 5_000 {
            log::info!(
                "[alive] t={}ms  wpm={}  playing={}  screen={:?}",
                now_ms,
                state.reader.wpm(),
                state.playing,
                matches!(state.screen, ui::Screen::Reader)
            );
            last_alive_ms = now_ms;
        }

        std::thread::sleep(std::time::Duration::from_millis(4));
    }
}
