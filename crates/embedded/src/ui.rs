use embedded_graphics::{
    mono_font::{
        ascii::{FONT_10X20, FONT_6X10, FONT_9X15},
        MonoTextStyle,
    },
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};

use crate::display::Display;
use rrsvp_core::reader::ReadingLoop;

// Dimensions
const W: i32 = 640;
const H: i32 = 172;
const HDR: i32 = 26;
const FTR: i32 = 20;

// ORP colour palette
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OrpColor {
    Red,
    Blue,
    Green,
    Orange,
    Yellow,
    Purple,
    White,
}

const ORP_ALL: [OrpColor; 7] = [
    OrpColor::Red,
    OrpColor::Blue,
    OrpColor::Green,
    OrpColor::Orange,
    OrpColor::Yellow,
    OrpColor::Purple,
    OrpColor::White,
];

impl OrpColor {
    fn rgb(self) -> Rgb565 {
        match self {
            OrpColor::Red => Rgb565::new(31, 0, 0),
            OrpColor::Blue => Rgb565::new(0, 0, 31),
            OrpColor::Green => Rgb565::new(0, 48, 0),
            OrpColor::Orange => Rgb565::new(31, 26, 0),
            OrpColor::Yellow => Rgb565::new(31, 63, 0),
            OrpColor::Purple => Rgb565::new(18, 0, 18),
            OrpColor::White => Rgb565::WHITE,
        }
    }
    fn label(self) -> &'static str {
        match self {
            OrpColor::Red => "R",
            OrpColor::Blue => "B",
            OrpColor::Green => "G",
            OrpColor::Orange => "O",
            OrpColor::Yellow => "Y",
            OrpColor::Purple => "P",
            OrpColor::White => "W",
        }
    }
    pub fn to_u8(self) -> u8 {
        match self {
            OrpColor::Red => 0,
            OrpColor::Blue => 1,
            OrpColor::Green => 2,
            OrpColor::Orange => 3,
            OrpColor::Yellow => 4,
            OrpColor::Purple => 5,
            OrpColor::White => 6,
        }
    }
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => OrpColor::Red,
            1 => OrpColor::Blue,
            2 => OrpColor::Green,
            3 => OrpColor::Orange,
            4 => OrpColor::Yellow,
            5 => OrpColor::Purple,
            _ => OrpColor::White,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FontSize {
    Small,
    Medium,
    Large,
    XLarge,
}

impl FontSize {
    fn char_w(self) -> i32 {
        match self {
            FontSize::Small => 6,
            FontSize::Medium => 9,
            FontSize::Large => 10,
            FontSize::XLarge => 13,
        }
    }
    fn line_h(self) -> i32 {
        match self {
            FontSize::Small => 12,
            FontSize::Medium => 17,
            FontSize::Large => 22,
            FontSize::XLarge => 26,
        }
    }
    fn baseline(self) -> i32 {
        match self {
            FontSize::Small => 9,
            FontSize::Medium => 13,
            FontSize::Large => 17,
            FontSize::XLarge => 20,
        }
    }
    fn rsvp_y_offset(self) -> i32 {
        match self {
            FontSize::Small => 4,
            FontSize::Medium => 6,
            FontSize::Large => 7,
            FontSize::XLarge => 7,
        }
    }

    fn font_height(self) -> i32 {
        match self {
            FontSize::Small => 10,
            FontSize::Medium => 15,
            FontSize::Large => 20,
            FontSize::XLarge => 22,
        }
    }
    pub fn to_u8(self) -> u8 {
        match self {
            FontSize::Small => 0,
            FontSize::Medium => 1,
            FontSize::Large => 2,
            FontSize::XLarge => 3,
        }
    }
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => FontSize::Small,
            1 => FontSize::Medium,
            2 => FontSize::Large,
            _ => FontSize::XLarge,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FocusMode {
    None,
    BarsOnly,
    BarsAndLine,
    LineOnly,
}

impl FocusMode {
    pub fn to_u8(self) -> u8 {
        match self {
            FocusMode::None => 0,
            FocusMode::BarsOnly => 1,
            FocusMode::BarsAndLine => 2,
            FocusMode::LineOnly => 3,
        }
    }
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => FocusMode::None,
            1 => FocusMode::BarsOnly,
            2 => FocusMode::BarsAndLine,
            _ => FocusMode::LineOnly,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReadingMode {
    Rsvp,
    Scroll,
}

pub struct Settings {
    pub dark_mode: bool,
    pub orp_color: OrpColor,
    pub font_size: FontSize,
    pub focus_mode: FocusMode,
    pub focus_bar_color: OrpColor,
    pub phantom_words: bool,
    pub brightness: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dark_mode: true,
            orp_color: OrpColor::Red,
            font_size: FontSize::Large,
            focus_mode: FocusMode::None,
            focus_bar_color: OrpColor::White,
            phantom_words: false,
            brightness: 10,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    Books,
    Settings,
    Reader,
}

pub struct UiState {
    pub screen: Screen,
    pub reader: ReadingLoop,
    pub playing: bool,
    pub reading_mode: ReadingMode,
    pub settings: Settings,
    pub wpm_overlay_ms: u32,
}

impl UiState {
    pub fn new() -> Self {
        let mut reader = ReadingLoop::new();
        reader.begin(0);
        Self {
            screen: Screen::MainMenu,
            reader,
            playing: false,
            reading_mode: ReadingMode::Rsvp,
            settings: Settings::default(),
            wpm_overlay_ms: 0,
        }
    }

    pub fn with_settings(settings: Settings, wpm: u16) -> Self {
        let mut reader = ReadingLoop::new();
        reader.begin(0);
        reader.set_wpm(wpm);
        Self {
            screen: Screen::MainMenu,
            reader,
            playing: false,
            reading_mode: ReadingMode::Rsvp,
            settings,
            wpm_overlay_ms: 0,
        }
    }

    pub fn handle_touch(&mut self, tp: Option<crate::touch::TPoint>, now_ms: u32) {
        let Some(pt) = tp else { return };
        match self.screen {
            Screen::MainMenu => self.touch_menu(pt),
            Screen::Books => self.touch_books(pt),
            Screen::Settings => self.touch_settings(pt),
            Screen::Reader => self.touch_reader(pt, now_ms),
        }
    }

    fn touch_menu(&mut self, pt: crate::touch::TPoint) {
        if pt.x < (W / 3) as u16 {
            self.screen = Screen::Settings;
        } else if pt.x < (2 * W / 3) as u16 {
            self.screen = Screen::Books;
        } else {
            self.screen = Screen::Reader;
        }
    }

    fn touch_books(&mut self, pt: crate::touch::TPoint) {
        if pt.y < HDR as u16 {
            self.screen = Screen::MainMenu;
        }
    }

    fn touch_settings(&mut self, pt: crate::touch::TPoint) {
        let y = pt.y as i32;
        let x = pt.x as i32;
        if y < 24 {
            self.screen = Screen::MainMenu;
            return;
        }
        let lw = 160i32;
        let avail = W - lw;
        if y < 45 {
            let half = lw + avail / 2;
            self.settings.dark_mode = x >= half;
        } else if y < 66 {
            if x >= lw {
                let idx = ((x - lw) * ORP_ALL.len() as i32 / avail).min(ORP_ALL.len() as i32 - 1)
                    as usize;
                self.settings.orp_color = ORP_ALL[idx];
            }
        } else if y < 87 {
            if x >= lw {
                let idx = (x - lw) * 4 / avail;
                self.settings.font_size = match idx {
                    0 => FontSize::Small,
                    1 => FontSize::Medium,
                    2 => FontSize::Large,
                    _ => FontSize::XLarge,
                };
            }
        } else if y < 108 {
            if x >= lw {
                let idx = (x - lw) * 4 / avail;
                self.settings.focus_mode = match idx {
                    0 => FocusMode::None,
                    1 => FocusMode::BarsOnly,
                    2 => FocusMode::BarsAndLine,
                    _ => FocusMode::LineOnly,
                };
            }
        } else if y < 129 {
            if x >= lw {
                let idx = ((x - lw) * ORP_ALL.len() as i32 / avail).min(ORP_ALL.len() as i32 - 1)
                    as usize;
                self.settings.focus_bar_color = ORP_ALL[idx];
            }
        } else if y < 150 {
            let half = lw + avail / 2;
            self.settings.phantom_words = x >= half;
        } else {
            if x >= lw {
                let idx = ((x - lw) * 10 / avail).clamp(0, 9);
                self.settings.brightness = (idx + 1) as u8;
            }
        }
    }

    fn touch_reader(&mut self, pt: crate::touch::TPoint, now_ms: u32) {
        if pt.y < HDR as u16 {
            if pt.x < (W / 4) as u16 {
                self.screen = Screen::MainMenu;
                self.playing = false;
            } else if pt.x >= (3 * W / 4) as u16 {
                self.reading_mode = match self.reading_mode {
                    ReadingMode::Rsvp => ReadingMode::Scroll,
                    ReadingMode::Scroll => ReadingMode::Rsvp,
                };
                if self.reading_mode == ReadingMode::Scroll {
                    self.playing = false;
                }
            } else {
            }
            return;
        }
        if pt.y > (H - FTR) as u16 {
            match self.reading_mode {
                ReadingMode::Rsvp => {
                    if pt.x >= (3 * W / 4) as u16 {
                        self.playing = !self.playing;
                        if self.playing {
                            self.reader.start(now_ms);
                        }
                    } else if pt.x >= (W / 2) as u16 {
                        self.reader.adjust_wpm(1);
                        self.wpm_overlay_ms = now_ms + 1500;
                    } else {
                        self.reader.adjust_wpm(-1);
                        self.wpm_overlay_ms = now_ms + 1500;
                    }
                }
                ReadingMode::Scroll => {
                    if pt.x > (W / 2) as u16 {
                        self.reader.scrub(10);
                    } else {
                        self.reader.scrub(-10);
                    }
                }
            }
            return;
        }
        if pt.x < (W / 3) as u16 {
            match self.reading_mode {
                ReadingMode::Scroll => self.reader.scrub(-1),
                ReadingMode::Rsvp => self.reader.rewind_sentence(),
            }
        } else if pt.x > (2 * W / 3) as u16 {
            self.reader.scrub(1);
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

// Palette helpers
fn bg(dark: bool) -> Rgb565 {
    if dark {
        Rgb565::new(1, 2, 1)
    } else {
        Rgb565::WHITE
    }
}
fn fg_color(dark: bool) -> Rgb565 {
    if dark {
        Rgb565::WHITE
    } else {
        Rgb565::BLACK
    }
}
fn dim_color(dark: bool) -> Rgb565 {
    if dark {
        Rgb565::new(8, 16, 8)
    } else {
        Rgb565::new(16, 32, 16)
    }
}
fn panel_bg(dark: bool) -> Rgb565 {
    if dark {
        Rgb565::new(4, 8, 4)
    } else {
        Rgb565::new(22, 44, 22)
    }
}

// Draw dispatch
pub fn draw(display: &mut Display, state: &UiState, now_ms: u32) {
    match state.screen {
        Screen::MainMenu => draw_menu(display, state),
        Screen::Books => draw_books(display, state),
        Screen::Settings => draw_settings(display, state),
        Screen::Reader => draw_reader(display, state, now_ms),
    }
}

// Main Menu
fn draw_menu(display: &mut Display, state: &UiState) {
    let dark = state.settings.dark_mode;
    fill(display, 0, 0, W, H, bg(dark));
    Text::with_alignment(
        "RRSVP",
        Point::new(W / 2, 32),
        MonoTextStyle::new(&FONT_10X20, fg_color(dark)),
        Alignment::Center,
    )
    .draw(display)
    .ok();
    Text::with_alignment(
        "Rapid Serial Visual Presentation",
        Point::new(W / 2, 52),
        MonoTextStyle::new(&FONT_6X10, dim_color(dark)),
        Alignment::Center,
    )
    .draw(display)
    .ok();
    fill(display, 0, 62, W, 1, dim_color(dark));

    let btn_top = 66i32;
    let btn_h = H - btn_top;
    let btn_w = W / 3;
    // issue #4: Settings left, Books middle, Resume right
    let buttons = [
        ("SETTINGS", "Preferences"),
        ("BOOKS", "Browse library"),
        ("RESUME >", "Start reading"),
    ];
    for (i, (title, sub)) in buttons.iter().enumerate() {
        let bx = i as i32 * btn_w;
        fill(display, bx + 2, btn_top, btn_w - 4, btn_h, panel_bg(dark));
        Text::with_alignment(
            title,
            Point::new(bx + btn_w / 2, btn_top + btn_h / 2),
            MonoTextStyle::new(&FONT_9X15, fg_color(dark)),
            Alignment::Center,
        )
        .draw(display)
        .ok();
        Text::with_alignment(
            sub,
            Point::new(bx + btn_w / 2, btn_top + btn_h / 2 + 16),
            MonoTextStyle::new(&FONT_6X10, dim_color(dark)),
            Alignment::Center,
        )
        .draw(display)
        .ok();
        if i < 2 {
            fill(display, bx + btn_w - 1, btn_top, 2, btn_h, dim_color(dark));
        }
    }
}

// Books
fn draw_books(display: &mut Display, state: &UiState) {
    let dark = state.settings.dark_mode;
    fill(display, 0, 0, W, H, bg(dark));
    draw_header_bar(display, dark, "< BACK", "BOOKS");
    Text::with_alignment(
        "No books loaded",
        Point::new(W / 2, H / 2 + 4),
        MonoTextStyle::new(&FONT_9X15, dim_color(dark)),
        Alignment::Center,
    )
    .draw(display)
    .ok();
    Text::with_alignment(
        "Insert SD card with .rsvp files in /books/books",
        Point::new(W / 2, H / 2 + 22),
        MonoTextStyle::new(&FONT_6X10, dim_color(dark)),
        Alignment::Center,
    )
    .draw(display)
    .ok();
}

// Settings
fn draw_settings(display: &mut Display, state: &UiState) {
    let dark = state.settings.dark_mode;
    let s = &state.settings;
    fill(display, 0, 0, W, H, bg(dark));
    draw_header_bar(display, dark, "< BACK", "SETTINGS");
    settings_toggle(display, dark, 24, "Dark Mode", s.dark_mode);
    settings_colors(display, dark, 45, "ORP Colour", s.orp_color);
    settings_fonts(display, dark, 66, "Font Size", s.font_size);
    settings_focus(display, dark, 87, "Focus", s.focus_mode);
    settings_colors(display, dark, 108, "Bar Colour", s.focus_bar_color);
    settings_toggle(display, dark, 129, "Phantom Wds", s.phantom_words);
    settings_brightness(display, dark, 150, "Brightness", s.brightness);
}

fn draw_header_bar(display: &mut Display, dark: bool, back: &str, title: &str) {
    fill(display, 0, 0, W, 23, panel_bg(dark));
    fill(display, 0, 23, W, 1, dim_color(dark));
    Text::new(
        back,
        Point::new(6, 16),
        MonoTextStyle::new(&FONT_6X10, dim_color(dark)),
    )
    .draw(display)
    .ok();
    Text::with_alignment(
        title,
        Point::new(W / 2, 16),
        MonoTextStyle::new(&FONT_9X15, fg_color(dark)),
        Alignment::Center,
    )
    .draw(display)
    .ok();
}

fn settings_toggle(display: &mut Display, dark: bool, y: i32, label: &str, on: bool) {
    fill(display, 0, y, W, 1, dim_color(dark));
    Text::new(
        label,
        Point::new(6, y + 14),
        MonoTextStyle::new(&FONT_6X10, fg_color(dark)),
    )
    .draw(display)
    .ok();
    let lw = 160i32;
    let half = lw + (W - lw) / 2;
    let rh = 19i32;
    let off_bg = if !on { fg_color(dark) } else { panel_bg(dark) };
    let off_fg = if !on { bg(dark) } else { fg_color(dark) };
    fill(display, lw + 4, y + 1, (W - lw) / 2 - 8, rh, off_bg);
    Text::with_alignment(
        "OFF",
        Point::new(lw + 4 + (W - lw) / 4, y + rh / 2 + 4),
        MonoTextStyle::new(&FONT_6X10, off_fg),
        Alignment::Center,
    )
    .draw(display)
    .ok();
    let on_bg = if on { fg_color(dark) } else { panel_bg(dark) };
    let on_fg = if on { bg(dark) } else { fg_color(dark) };
    fill(display, half + 4, y + 1, (W - lw) / 2 - 8, rh, on_bg);
    Text::with_alignment(
        "ON",
        Point::new(half + 4 + (W - lw) / 4, y + rh / 2 + 4),
        MonoTextStyle::new(&FONT_6X10, on_fg),
        Alignment::Center,
    )
    .draw(display)
    .ok();
}

fn settings_colors(display: &mut Display, dark: bool, y: i32, label: &str, sel: OrpColor) {
    fill(display, 0, y, W, 1, dim_color(dark));
    Text::new(
        label,
        Point::new(6, y + 14),
        MonoTextStyle::new(&FONT_6X10, fg_color(dark)),
    )
    .draw(display)
    .ok();
    let lw = 160i32;
    let avail = W - lw;
    let n = ORP_ALL.len() as i32;
    let bw = avail / n;
    for (i, &c) in ORP_ALL.iter().enumerate() {
        let bx = lw + i as i32 * bw;
        let is_sel = c == sel;
        let btn_bg = if is_sel { c.rgb() } else { panel_bg(dark) };
        let txt_fg = if is_sel { bg(dark) } else { c.rgb() };
        fill(display, bx + 1, y + 2, bw - 2, 17, btn_bg);
        Text::with_alignment(
            c.label(),
            Point::new(bx + bw / 2, y + 12),
            MonoTextStyle::new(&FONT_6X10, txt_fg),
            Alignment::Center,
        )
        .draw(display)
        .ok();
    }
}

fn settings_fonts(display: &mut Display, dark: bool, y: i32, label: &str, sel: FontSize) {
    fill(display, 0, y, W, 1, dim_color(dark));
    Text::new(
        label,
        Point::new(6, y + 14),
        MonoTextStyle::new(&FONT_6X10, fg_color(dark)),
    )
    .draw(display)
    .ok();
    let lw = 160i32;
    let avail = W - lw;
    let bw = avail / 4;
    let sizes = [
        (FontSize::Small, "S"),
        (FontSize::Medium, "M"),
        (FontSize::Large, "L"),
        (FontSize::XLarge, "XL"),
    ];
    for (i, (sz, lbl)) in sizes.iter().enumerate() {
        let bx = lw + i as i32 * bw;
        let is_sel = *sz == sel;
        let btn_bg = if is_sel {
            fg_color(dark)
        } else {
            panel_bg(dark)
        };
        let txt_fg = if is_sel { bg(dark) } else { fg_color(dark) };
        fill(display, bx + 2, y + 2, bw - 4, 17, btn_bg);
        Text::with_alignment(
            lbl,
            Point::new(bx + bw / 2, y + 12),
            MonoTextStyle::new(&FONT_6X10, txt_fg),
            Alignment::Center,
        )
        .draw(display)
        .ok();
    }
}

fn settings_focus(display: &mut Display, dark: bool, y: i32, label: &str, sel: FocusMode) {
    fill(display, 0, y, W, 1, dim_color(dark));
    Text::new(
        label,
        Point::new(6, y + 14),
        MonoTextStyle::new(&FONT_6X10, fg_color(dark)),
    )
    .draw(display)
    .ok();
    let lw = 160i32;
    let avail = W - lw;
    let bw = avail / 4;
    let modes = [
        (FocusMode::None, "OFF"),
        (FocusMode::BarsOnly, "BARS"),
        (FocusMode::BarsAndLine, "BARS+"),
        (FocusMode::LineOnly, "LINE"),
    ];
    for (i, (mode, lbl)) in modes.iter().enumerate() {
        let bx = lw + i as i32 * bw;
        let is_sel = *mode == sel;
        let btn_bg = if is_sel {
            fg_color(dark)
        } else {
            panel_bg(dark)
        };
        let txt_fg = if is_sel { bg(dark) } else { fg_color(dark) };
        fill(display, bx + 1, y + 2, bw - 2, 17, btn_bg);
        Text::with_alignment(
            lbl,
            Point::new(bx + bw / 2, y + 12),
            MonoTextStyle::new(&FONT_6X10, txt_fg),
            Alignment::Center,
        )
        .draw(display)
        .ok();
    }
}

fn settings_brightness(display: &mut Display, dark: bool, y: i32, label: &str, level: u8) {
    fill(display, 0, y, W, 1, dim_color(dark));
    Text::new(
        label,
        Point::new(6, y + 14),
        MonoTextStyle::new(&FONT_6X10, fg_color(dark)),
    )
    .draw(display)
    .ok();
    let lw = 160i32;
    let avail = W - lw;
    let n = 10i32;
    let bw = avail / n;
    for i in 0..n {
        let bx = lw + i * bw;
        let is_sel = (i + 1) as u8 == level;
        let is_on = (i + 1) as u8 <= level;
        let btn_bg = if is_sel {
            fg_color(dark)
        } else if is_on {
            dim_color(dark)
        } else {
            panel_bg(dark)
        };
        let txt_fg = if is_sel { bg(dark) } else { fg_color(dark) };
        fill(display, bx + 1, y + 2, bw - 2, 16, btn_bg);
        if is_sel {
            let s = format!("{}", i + 1);
            Text::with_alignment(
                &s,
                Point::new(bx + bw / 2, y + 13),
                MonoTextStyle::new(&FONT_6X10, txt_fg),
                Alignment::Center,
            )
            .draw(display)
            .ok();
        }
    }
}

// Reader
fn draw_reader(display: &mut Display, state: &UiState, now_ms: u32) {
    let dark = state.settings.dark_mode;
    fill(display, 0, 0, W, H, bg(dark));
    reader_header(display, state);
    match state.reading_mode {
        ReadingMode::Rsvp => reader_rsvp_word(display, state),
        ReadingMode::Scroll => reader_scroll_view(display, state),
    }
    reader_footer(display, state, now_ms);
}

fn reader_header(display: &mut Display, state: &UiState) {
    let dark = state.settings.dark_mode;
    let total = state.reader.word_count();
    let idx = state.reader.current_index();
    let pct = if total > 0 { (idx * 100) / total } else { 0 };
    let icon = if state.playing { ">" } else { "||" };
    let mode_lbl = match state.reading_mode {
        ReadingMode::Rsvp => "RSVP",
        ReadingMode::Scroll => "SCROLL",
    };

    fill(display, 0, HDR, W, 1, dim_color(dark));
    Text::new(
        "< MENU",
        Point::new(4, HDR - 6),
        MonoTextStyle::new(&FONT_6X10, dim_color(dark)),
    )
    .draw(display)
    .ok();

    let info = format!("{pct}% {icon} {} wpm", state.reader.wpm());
    Text::with_alignment(
        &info,
        Point::new(W / 2, HDR - 6),
        MonoTextStyle::new(&FONT_6X10, dim_color(dark)),
        Alignment::Center,
    )
    .draw(display)
    .ok();

    Text::with_alignment(
        mode_lbl,
        Point::new(W - 4, HDR - 6),
        MonoTextStyle::new(&FONT_6X10, dim_color(dark)),
        Alignment::Right,
    )
    .draw(display)
    .ok();
}

fn reader_rsvp_word(display: &mut Display, state: &UiState) {
    let dark = state.settings.dark_mode;
    let s = &state.settings;
    let word = state.reader.current_word();
    if word.is_empty() {
        return;
    }

    let word_area_h = H - HDR - FTR;
    let center_y = HDR + word_area_h / 2;
    let char_w = s.font_size.char_w();
    let orp_idx = orp_index(word);

    let orp_x = W / 2 - char_w / 2;
    let start_x = orp_x - orp_idx as i32 * char_w;

    let bar_top = HDR + 3;
    let bar_bot = H - FTR - 5;
    match s.focus_mode {
        FocusMode::None => {}
        FocusMode::BarsOnly => {
            let bc = s.focus_bar_color.rgb();
            fill(display, 0, bar_top, W, 2, bc);
            fill(display, 0, bar_bot, W, 2, bc);
        }
        FocusMode::BarsAndLine => {
            let bc = s.focus_bar_color.rgb();
            fill(display, 0, bar_top, W, 2, bc);
            fill(display, 0, bar_bot, W, 2, bc);
            draw_focus_line(display, bc, bar_top, bar_bot, center_y, s.font_size);
        }
        FocusMode::LineOnly => {
            let bc = s.focus_bar_color.rgb();
            draw_focus_line(display, bc, bar_top, bar_bot, center_y, s.font_size);
        }
    }

    if s.phantom_words {
        let ph = MonoTextStyle::new(&FONT_6X10, dim_color(dark));
        let ci = state.reader.current_index();
        if ci > 0 {
            let pw = state.reader.word_at(ci - 1);
            Text::with_alignment(&pw, Point::new(W / 6, center_y + 4), ph, Alignment::Center)
                .draw(display)
                .ok();
        }
        let ni = ci + 1;
        if ni < state.reader.word_count() {
            let nw = state.reader.word_at(ni);
            Text::with_alignment(
                &nw,
                Point::new(5 * W / 6, center_y + 4),
                ph,
                Alignment::Center,
            )
            .draw(display)
            .ok();
        }
    }

    let orp_col = s.orp_color.rgb();
    let y_off = s.font_size.rsvp_y_offset();
    for (i, ch) in word.chars().enumerate() {
        let x = start_x + i as i32 * char_w;
        if x + char_w < 0 || x >= W {
            continue;
        }
        let col = if i == orp_idx {
            orp_col
        } else {
            fg_color(dark)
        };
        draw_word_char(
            display,
            &ch.to_string(),
            x,
            center_y + y_off,
            s.font_size,
            col,
        );
    }
}

fn draw_word_char(display: &mut Display, ch: &str, x: i32, y: i32, size: FontSize, color: Rgb565) {
    match size {
        FontSize::Small => {
            Text::new(ch, Point::new(x, y), MonoTextStyle::new(&FONT_6X10, color))
                .draw(display)
                .ok();
        }
        FontSize::Medium => {
            Text::new(ch, Point::new(x, y), MonoTextStyle::new(&FONT_9X15, color))
                .draw(display)
                .ok();
        }
        FontSize::Large => {
            Text::new(ch, Point::new(x, y), MonoTextStyle::new(&FONT_10X20, color))
                .draw(display)
                .ok();
        }
        FontSize::XLarge => {
            for dx in 0i32..=1 {
                for dy in 0i32..=1 {
                    Text::new(
                        ch,
                        Point::new(x + dx, y + dy),
                        MonoTextStyle::new(&FONT_10X20, color),
                    )
                    .draw(display)
                    .ok();
                }
            }
        }
    }
}

fn reader_scroll_view(display: &mut Display, state: &UiState) {
    let dark = state.settings.dark_mode;
    let s = &state.settings;
    let char_w = s.font_size.char_w();
    let line_h = s.font_size.line_h();
    let baseline = s.font_size.baseline();

    let area_top = HDR + 2;
    let area_h = H - HDR - FTR - 2;
    let vis_lines = (area_h / line_h) as usize;
    let max_per_line = (W / char_w) as usize;

    let total = state.reader.word_count();
    let cur = state.reader.current_index();
    if total == 0 {
        return;
    }

    let avg_word_len = 5usize;
    let approx_words_per_line = max_per_line / (avg_word_len + 1);
    let scan_back = (vis_lines / 2 + 1) * (approx_words_per_line + 1);
    let scan_start = cur.saturating_sub(scan_back);

    let max_lines = vis_lines * 3;
    let mut line_starts: Vec<usize> = Vec::with_capacity(max_lines + 1);
    let mut cur_line: usize = 0;
    let mut wi = scan_start;

    while wi < total && line_starts.len() < max_lines {
        line_starts.push(wi);
        let line_idx = line_starts.len() - 1;
        let mut line_chars = 0usize;

        while wi < total {
            let word = state.reader.word_at(wi);
            let needed = word.len() + if line_chars > 0 { 1 } else { 0 };
            if line_chars > 0 && line_chars + needed > max_per_line {
                break;
            }
            if wi == cur {
                cur_line = line_idx;
            }
            line_chars += needed;
            wi += 1;
        }

        if line_starts.last().copied() == Some(wi) {
            wi += 1;
        }
    }
    let total_lines = line_starts.len();
    line_starts.push(wi.min(total));

    let half = vis_lines / 2;
    let view_start = cur_line.saturating_sub(half);

    for disp_line in 0..vis_lines {
        let line_idx = view_start + disp_line;
        if line_idx >= total_lines {
            break;
        }

        let word_start = line_starts[line_idx];
        let word_end = line_starts[line_idx + 1].min(total);
        let y = area_top + disp_line as i32 * line_h;
        let mut x = 0i32;

        for word_i in word_start..word_end {
            if word_i >= total {
                break;
            }
            let word = state.reader.word_at(word_i);
            let col = if word_i == cur {
                s.orp_color.rgb()
            } else {
                fg_color(dark)
            };
            if x > 0 {
                x += char_w;
            }
            draw_word_char(display, &word, x, y + baseline, s.font_size, col);
            x += word.len() as i32 * char_w;
            if x >= W {
                break;
            }
        }
    }
}

// Reader footer
fn reader_footer(display: &mut Display, state: &UiState, now_ms: u32) {
    let dark = state.settings.dark_mode;
    let top = H - FTR;
    let btn_x = 3 * W / 4;
    fill(display, 0, top, W, 1, dim_color(dark));
    match state.reading_mode {
        ReadingMode::Scroll => {
            let label = "< word   |   scroll   |   word >";
            Text::with_alignment(
                label,
                Point::new(W / 2, top + FTR - 4),
                MonoTextStyle::new(&FONT_6X10, dim_color(dark)),
                Alignment::Center,
            )
            .draw(display)
            .ok();
        }
        ReadingMode::Rsvp => {
            // WPM label confined to left 3/4 of footer
            let label = if now_ms < state.wpm_overlay_ms {
                format!("WPM: {}", state.reader.wpm())
            } else {
                String::from("< wpm   |   wpm >")
            };
            Text::with_alignment(
                &label,
                Point::new(btn_x / 2, top + FTR - 4),
                MonoTextStyle::new(&FONT_6X10, dim_color(dark)),
                Alignment::Center,
            )
            .draw(display)
            .ok();
            let btn_bg = if state.playing {
                fg_color(dark)
            } else {
                panel_bg(dark)
            };
            fill(display, btn_x, top + 1, W - btn_x, FTR - 1, btn_bg);
            let icon = if state.playing { "||" } else { ">" };
            let icon_col = if state.playing {
                bg(dark)
            } else {
                fg_color(dark)
            };
            Text::with_alignment(
                icon,
                Point::new(btn_x + (W - btn_x) / 2, top + FTR - 4),
                MonoTextStyle::new(&FONT_6X10, icon_col),
                Alignment::Center,
            )
            .draw(display)
            .ok();
        }
    }
}

// ORP index
fn orp_index(word: &str) -> usize {
    let n = word.chars().count();
    if n == 0 {
        0
    } else {
        ((n.saturating_sub(1)) / 4).min(n - 1)
    }
}
/// Draw the vertical focus line in two segments: above and below the word area.
/// The gap prevents the line from passing through the displayed word.
fn draw_focus_line(
    display: &mut Display,
    color: Rgb565,
    bar_top: i32,
    bar_bot: i32,
    center_y: i32,
    font: FontSize,
) {
    let word_baseline = center_y + font.rsvp_y_offset();
    let word_top = word_baseline - font.font_height();
    let word_bottom = word_baseline;
    let gap = 4i32; // px clearance above and below the word

    let cx = W / 2;

    // Top segment: from just below the top bar to just above the word
    let seg_top_start = bar_top + 2; // 2 = bar thickness
    let seg_top_end = word_top - gap;
    if seg_top_end > seg_top_start {
        fill(
            display,
            cx,
            seg_top_start,
            1,
            seg_top_end - seg_top_start,
            color,
        );
    }

    // Bottom segment: from just below the word to just above the bottom bar
    let seg_bot_start = word_bottom + gap;
    let seg_bot_end = bar_bot; // bar_bot is the top edge of the bottom bar
    if seg_bot_end > seg_bot_start {
        fill(
            display,
            cx,
            seg_bot_start,
            1,
            seg_bot_end - seg_bot_start,
            color,
        );
    }
}
// Fill helper
fn fill(display: &mut Display, x: i32, y: i32, w: i32, h: i32, color: Rgb565) {
    if w <= 0 || h <= 0 {
        return;
    }
    Rectangle::new(Point::new(x, y), Size::new(w as u32, h as u32))
        .into_styled(PrimitiveStyle::with_fill(color))
        .draw(display)
        .ok();
}
