mod board;
mod display;
mod touch;
mod ui;

use board::*;
use display::Display;
use touch::TouchDriver;
use ui::{draw, UiState};

use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::*;

fn main() {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();
    log::info!("=== RRSVP booting on ESP32-S3-Touch-LCD-3.49 ===");

    // --- Button GPIO setup (active-low, pulled up) ---
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
    }

    log::info!("initialising display...");
    let mut display = Display::new();

    display.fill_solid(0xFFFF); // white flash to confirm display is alive
    std::thread::sleep(std::time::Duration::from_millis(300));

    log::info!("initialising touch...");
    let mut touch = TouchDriver::new();
    log::info!("touch init complete");

    let mut state = UiState::new();
    let mut last_tp: Option<touch::TPoint> = None;
    let mut last_draw_ms: u32 = 0;
    let mut last_alive_ms: u32 = 0;
    // issue #3: debounce — only register touch actions 350ms apart
    let mut last_touch_action_ms: u32 = 0;
    const TOUCH_DEBOUNCE_MS: u32 = 350;

    // issue #2: power button hold tracking
    let mut power_hold_start_ms: u32 = 0;
    let mut boot_was_pressed = false;

    log::info!("=== Entering main loop ===");

    loop {
        let now_ms = unsafe { (esp_idf_svc::sys::esp_timer_get_time() / 1000) as u32 };

        // ── Button polling ────────────────────────────────────────────────────
        unsafe {
            let boot_low = gpio_get_level(BTN_BOOT) == 0;
            if boot_low && !boot_was_pressed {
                log::info!("[BTN] BOOT button pressed (GPIO{})", BTN_BOOT);
            } else if !boot_low && boot_was_pressed {
                log::info!("[BTN] BOOT button released");
            }
            boot_was_pressed = boot_low;

            // issue #2: POWER long-press → deep sleep
            let power_low = gpio_get_level(BTN_POWER) == 0;
            if power_low {
                if power_hold_start_ms == 0 {
                    power_hold_start_ms = now_ms;
                    log::info!("[BTN] POWER pressed — hold 1.5 s to power off");
                } else if now_ms.wrapping_sub(power_hold_start_ms) >= 1500 {
                    log::info!("[POWER] Holding POWER 1.5 s — entering deep sleep");
                    gpio_set_level(LCD_BL, 1);
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    let ret = esp_sleep_enable_ext0_wakeup(BTN_POWER as gpio_num_t, 0);
                    if ret != 0 {
                        log::warn!("[POWER] ext0 wakeup config failed ({}), sleeping anyway", ret);
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

        // ── Touch polling — issue #3: debounce ───────────────────────────────
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

        // ── Reading loop ──────────────────────────────────────────────────────
        if state.playing {
            state.reader.update(now_ms, true);
        }

        // ── Redraw at ~60 fps ─────────────────────────────────────────────────
        if now_ms.wrapping_sub(last_draw_ms) >= 16 {
            draw(&mut display, &state, now_ms);
            display.flush();
            last_draw_ms = now_ms;
        }

        // ── Periodic alive log every 5 s ──────────────────────────────────────
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
