#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum UiLanguage {
    #[default]
    English = 0,
}

impl UiLanguage {
    pub const COUNT: usize = 6;

    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::English,
            _ => Self::English,
        }
    }

    pub fn next(self) -> Self {
        Self::from_u8((self as u8 + 1) % Self::COUNT as u8)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::English => "English",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UiText {
    Resume = 0,
    Chapters,
    Library,
    Settings,
    UsbTransfer,
    PowerOff,
    Back,
    Display,
    TypographyTune,
    WordPacing,
    Theme,
    Brightness,
    Language,
    ReadingMode,
    LongWords,
    Complexity,
    Punctuation,
    ResetPacing,
    Night,
    Dark,
    Light,
    On,
    Off,
    FontSize,
    Typeface,
    PhantomWords,
    RedHighlight,
    Tracking,
    Anchor,
    GuideWidth,
    GuideGap,
    Reset,
    Typography,
    TapToExit,
    TapToReset,
    TapChangeSample,
    TapExitSample,
    TapToggleSample,
    TapCycleSample,
    CurrentBook,
    Start,
    StartOfBook,
    RestartBook,
    AreYouSure,
    NoKeepPlace,
    YesRestart,
    NoSamples,
    Large,
    Medium,
    Small,
    Standard,
    RsvpMode,
    ScrollMode,
    TimeEstimate,
    TimeEstimateAccurate,
    TimeEstimateFast,
}

static STRINGS_EN: &[&str] = &[
    "Resume",
    "Chapters",
    "Library",
    "Settings",
    "USB Transfer",
    "Power Off",
    "Back",
    "Display",
    "Typography Tune",
    "Word Pacing",
    "Theme",
    "Brightness",
    "Language",
    "Reading Mode",
    "Long Words",
    "Complexity",
    "Punctuation",
    "Reset Pacing",
    "Night",
    "Dark",
    "Light",
    "On",
    "Off",
    "Font Size",
    "Typeface",
    "Phantom Words",
    "Red Highlight",
    "Tracking",
    "Anchor",
    "Guide Width",
    "Guide Gap",
    "Reset",
    "Typography",
    "Tap to Exit",
    "Tap to Reset",
    "Tap to Change",
    "Tap to Exit",
    "Tap to Toggle",
    "Tap to Cycle",
    "Current Book",
    "Start",
    "Start of Book",
    "Restart Book",
    "Are you sure?",
    "No, keep place",
    "Yes, restart",
    "No samples",
    "Large",
    "Medium",
    "Small",
    "Standard",
    "RSVP Mode",
    "Scroll Mode",
    "Time estimate",
    "Time estimate (accurate)",
    "Time estimate (fast)",
];

pub fn get_string(lang: UiLanguage, text: UiText) -> &'static str {
    let idx = text as usize;
    let table: &[&str] = match lang {
        UiLanguage::English => STRINGS_EN,
    };
    table.get(idx).copied().unwrap_or("")
}
