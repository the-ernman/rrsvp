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

    // Solid-colour test: fill screen with pure white to verify pixels reach the display.
    // 0xFFFF = white in any byte order (symmetric value).
    // If the screen shows white -> display pipeline works.
    // If it shows garbage/lines -> SPI or init still broken.
    log::info!("display test: running color cycle — watch screen for WHITE/RED/GREEN/BLUE");
    display.test_color_cycle();

    std::thread::sleep(std::time::Duration::from_millis(1000));

    log::info!("initialising touch...");
    let mut touch = TouchDriver::new();
    log::info!("touch init complete");

    let mut state = UiState::new();
    let mut last_tp: Option<touch::TPoint> = None;
    let mut last_draw_ms: u32 = 0;
    let mut last_alive_ms: u32 = 0;

    // Button state (true = currently pressed/low)
    let mut boot_was_pressed = false;
    let mut power_was_pressed = false;

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

            let power_low = gpio_get_level(BTN_POWER) == 0;
            if power_low && !power_was_pressed {
                log::info!("[BTN] POWER button pressed (GPIO{})", BTN_POWER);
            } else if !power_low && power_was_pressed {
                log::info!("[BTN] POWER button released");
            }
            power_was_pressed = power_low;
        }

        // ── Touch polling ─────────────────────────────────────────────────────
        let tp = touch.read();
        if tp != last_tp {
            match tp {
                Some(pt) => log::info!("[TOUCH] x={} y={}", pt.x, pt.y),
                None => log::info!("[TOUCH] released"),
            }
            if tp.is_some() {
                state.handle_touch(tp, now_ms);
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
                "[alive] t={}ms  wpm={}  playing={}  tp={:?}",
                now_ms,
                state.reader.wpm(),
                state.playing,
                last_tp
            );
            last_alive_ms = now_ms;
        }

        std::thread::sleep(std::time::Duration::from_millis(4));
    }
}
