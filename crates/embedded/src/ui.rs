use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};

use crate::display::Display;
use rrsvp_core::reader::ReadingLoop;

// ── Screen layout (640 × 172) ─────────────────────────────────────────────────
const SCREEN_W: i32 = 640;
const SCREEN_H: i32 = 172;
const HEADER_H: i32 = 24;
const FOOTER_H: i32 = 20;
const WORD_AREA_Y: i32 = HEADER_H;
const WORD_AREA_H: i32 = SCREEN_H - HEADER_H - FOOTER_H;

// ── Colours (RGB565) ──────────────────────────────────────────────────────────
const BLACK: Rgb565 = Rgb565::BLACK;
const WHITE: Rgb565 = Rgb565::WHITE;
const ACCENT: Rgb565 = Rgb565::new(31, 10, 0); // warm orange-red
const DIM: Rgb565 = Rgb565::new(10, 20, 10); // muted green-grey

/// All state the UI needs to render a frame.
pub struct UiState {
    pub reader: ReadingLoop,
    pub playing: bool,
    pub wpm_overlay_ms: u32, // show WPM label until this timestamp
}

impl UiState {
    pub fn new() -> Self {
        let mut reader = ReadingLoop::new();
        reader.begin(0);
        Self {
            reader,
            playing: false,
            wpm_overlay_ms: 0,
        }
    }

    pub fn handle_touch(&mut self, tp: Option<crate::touch::TPoint>, now_ms: u32) {
        let Some(pt) = tp else { return };

        if pt.y < HEADER_H as u16 {
            // Header tap: toggle play/pause
            self.playing = !self.playing;
            if self.playing {
                self.reader.start(now_ms);
            }
            return;
        }

        if pt.y > (SCREEN_H - FOOTER_H) as u16 {
            // Footer tap: adjust WPM
            if pt.x > (SCREEN_W / 2) as u16 {
                self.reader.adjust_wpm(1);
            } else {
                self.reader.adjust_wpm(-1);
            }
            self.wpm_overlay_ms = now_ms + 1500;
            return;
        }

        // Body tap: scrub
        if pt.x < (SCREEN_W / 3) as u16 {
            self.reader.rewind_sentence();
        } else if pt.x > (2 * SCREEN_W / 3) as u16 {
            self.reader.scrub(1);
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

/// Draw the full UI into the display framebuffer.
pub fn draw(display: &mut Display, state: &UiState, now_ms: u32) {
    // Clear background
    Rectangle::new(
        Point::new(0, 0),
        Size::new(SCREEN_W as u32, SCREEN_H as u32),
    )
    .into_styled(PrimitiveStyle::with_fill(BLACK))
    .draw(display)
    .ok();

    draw_header(display, state, now_ms);
    draw_word(display, state);
    draw_footer(display, state, now_ms);
}

fn draw_header(display: &mut Display, state: &UiState, _now_ms: u32) {
    let style = MonoTextStyle::new(&FONT_6X10, DIM);
    let total = state.reader.word_count();
    let current = state.reader.current_index();
    let pct = if total > 0 {
        (current * 100) / total
    } else {
        0
    };
    let label = format!("{:>3}%  {} WPM", pct, state.reader.wpm());
    Text::with_alignment(
        &label,
        Point::new(SCREEN_W / 2, 14),
        style,
        Alignment::Center,
    )
    .draw(display)
    .ok();
}

fn draw_word(display: &mut Display, state: &UiState) {
    let word = state.reader.current_word();
    if word.is_empty() {
        return;
    }

    let center_y = WORD_AREA_Y + WORD_AREA_H / 2;

    // Attempt to find the ORP (Optimal Recognition Point) letter index.
    let orp_idx = orp_index(word);

    let word_style = MonoTextStyle::new(&FONT_10X20, WHITE);
    let orp_style = MonoTextStyle::new(&FONT_10X20, ACCENT);

    // Character width for FONT_10X20 is 10px.
    let char_w = 10i32;
    let total_w = word.len() as i32 * char_w;
    let start_x = (SCREEN_W - total_w) / 2;

    // Draw each character individually so we can highlight the ORP.
    for (i, ch) in word.chars().enumerate() {
        let x = start_x + i as i32 * char_w;
        let style = if i == orp_idx { orp_style } else { word_style };
        let ch_str = ch.to_string();
        Text::new(&ch_str, Point::new(x, center_y + 7), style)
            .draw(display)
            .ok();
    }
}

fn draw_footer(display: &mut Display, state: &UiState, now_ms: u32) {
    let footer_y = SCREEN_H - FOOTER_H / 2 + 4;

    // Show WPM overlay if recently adjusted, otherwise show play state.
    let label = if now_ms < state.wpm_overlay_ms {
        format!("WPM: {}", state.reader.wpm())
    } else if state.playing {
        "▶".to_string()
    } else {
        "⏸".to_string()
    };

    let style = MonoTextStyle::new(&FONT_6X10, DIM);
    Text::with_alignment(
        &label,
        Point::new(SCREEN_W / 2, footer_y),
        style,
        Alignment::Center,
    )
    .draw(display)
    .ok();
}

/// Compute the ORP (Optimal Recognition Point) character index for a word.
/// For a word of length N: index = N/4, clamped so there's always a char to highlight.
fn orp_index(word: &str) -> usize {
    let len = word.chars().count();
    if len == 0 {
        return 0;
    }
    ((len.saturating_sub(1)) / 4).min(len - 1)
}
