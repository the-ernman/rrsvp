use crate::text::latin;

// ── pacing constants (matching C++ exactly) ──────────────────────────────────

const MIN_WPM: u16 = 10;
const LOW_WPM_MAX: u16 = 100;
const LOW_WPM_STEP: u16 = 10;
const MAX_WPM: u16 = 1000;
const HIGH_WPM_STEP: u16 = 25;
const LONG_WORD_AFTER_CHARS: usize = 6;
const LONG_WORD_PERCENT_PER_CHAR: u16 = 6;
const VERY_LONG_WORD_AFTER_CHARS: usize = 10;
const VERY_LONG_WORD_PERCENT_PER_CHAR: u16 = 9;
const ULTRA_LONG_WORD_AFTER_CHARS: usize = 14;
const ULTRA_LONG_WORD_PERCENT_PER_CHAR: u16 = 12;
const LONG_WORD_MAX_PERCENT: u16 = 170;
const COMPOUND_JOINER_PERCENT: u16 = 14;
const LONG_COMPOUND_WORD_PERCENT: u16 = 18;
const TECHNICAL_CONNECTOR_PERCENT: u16 = 8;
const SYLLABLE_BONUS_AFTER_COUNT: usize = 2;
const SYLLABLE_BONUS_PERCENT_PER_GROUP: u16 = 10;
const SYLLABLE_BONUS_MAX_PERCENT: u16 = 50;
const ALL_CAPS_COMPLEXITY_PERCENT: u16 = 14;
const MIXED_TOKEN_COMPLEXITY_PERCENT: u16 = 22;
const NUMERIC_TOKEN_COMPLEXITY_PERCENT: u16 = 10;
const DENSE_CONNECTOR_COMPLEXITY_PERCENT: u16 = 12;
const COMPLEX_WORD_MAX_PERCENT: u16 = 85;
const COMMA_PAUSE_PERCENT: u16 = 45;
const DASH_PAUSE_PERCENT: u16 = 60;
const CLAUSE_PAUSE_PERCENT: u16 = 80;
const ELLIPSIS_PAUSE_PERCENT: u16 = 110;
const SENTENCE_PAUSE_PERCENT: u16 = 135;
const STRONG_SENTENCE_PAUSE_PERCENT: u16 = 150;
const MAX_CATCH_UP_WORDS: u8 = 4;
const MAX_PACING_DELAY_MS: u16 = 600;

/// Pacing tunables (matching C++ PacingConfig).
#[derive(Debug, Clone)]
pub struct PacingConfig {
    pub long_word_delay_ms: u16,
    pub complex_word_delay_ms: u16,
    pub punctuation_delay_ms: u16,
    pub long_word_scale_percent: u8,
    pub complex_word_scale_percent: u8,
    pub punctuation_scale_percent: u8,
}

impl Default for PacingConfig {
    fn default() -> Self {
        Self {
            long_word_delay_ms: 200,
            complex_word_delay_ms: 200,
            punctuation_delay_ms: 200,
            long_word_scale_percent: 100,
            complex_word_scale_percent: 100,
            punctuation_scale_percent: 100,
        }
    }
}

// ── word classifier helpers ───────────────────────────────────────────────────

fn is_letter(c: u8) -> bool {
    latin::is_letter(c)
}
fn is_digit(c: u8) -> bool {
    latin::is_digit(c)
}
fn is_lowercase(c: u8) -> bool {
    latin::is_lowercase_letter(c)
}
fn is_uppercase(c: u8) -> bool {
    latin::is_uppercase_letter(c)
}
fn is_vowel(c: u8) -> bool {
    latin::is_vowel(c)
}
fn is_word_char(c: u8) -> bool {
    latin::is_word_character(c)
}

fn is_segment_separator(c: u8) -> bool {
    matches!(c, b'-' | b'/' | b'_')
}
fn is_technical_connector(c: u8) -> bool {
    matches!(c, b'-' | b'/' | b'_' | b'.' | b'+' | b'\\')
}
fn is_ignored_trailing(c: u8) -> bool {
    matches!(c, b'"' | b'\'' | b')' | b']' | b'}')
}

fn letter_count(word: &str) -> usize {
    word.bytes().filter(|&b| is_letter(b)).count()
}

fn digit_count(word: &str) -> usize {
    word.bytes().filter(|&b| is_digit(b)).count()
}

fn uppercase_count(word: &str) -> usize {
    word.bytes().filter(|&b| is_uppercase(b)).count()
}

fn readable_char_count(word: &str) -> usize {
    word.bytes().filter(|&b| is_word_char(b)).count()
}

fn approximate_syllable_groups(word: &str) -> usize {
    let mut groups = 0usize;
    let mut letter_count_local = 0usize;
    let mut prev_was_vowel = false;
    let mut letters_only = Vec::new();

    for b in word.bytes() {
        if !is_letter(b) {
            prev_was_vowel = false;
            continue;
        }
        letter_count_local += 1;
        let lc = latin::to_lowercase_byte(b);
        letters_only.push(lc);
        let vowel = is_vowel(lc);
        if vowel && !prev_was_vowel {
            groups += 1;
        }
        prev_was_vowel = vowel;
    }

    if groups > 1
        && letter_count_local > 3
        && letters_only.last() == Some(&b'e')
        && !(letters_only.len() >= 2 && letters_only[letters_only.len() - 2] == b'l')
        && !(letters_only.len() >= 2 && letters_only[letters_only.len() - 2] == b'y')
    {
        groups -= 1;
    }

    if groups == 0 && letter_count_local > 0 {
        groups = 1;
    }

    groups
}

fn compound_joiner_count(word: &str) -> usize {
    let bytes = word.as_bytes();
    let mut count = 0usize;
    for i in 1..bytes.len().saturating_sub(1) {
        if is_segment_separator(bytes[i])
            && is_word_char(bytes[i - 1])
            && is_word_char(bytes[i + 1])
        {
            count += 1;
        }
    }
    count
}

fn technical_connector_count(word: &str) -> usize {
    let bytes = word.as_bytes();
    let mut count = 0usize;
    for i in 1..bytes.len().saturating_sub(1) {
        if is_technical_connector(bytes[i])
            && is_word_char(bytes[i - 1])
            && is_word_char(bytes[i + 1])
        {
            count += 1;
        }
    }
    count
}

fn last_meaningful_char_index(word: &str) -> Option<usize> {
    let bytes = word.as_bytes();
    for i in (0..bytes.len()).rev() {
        if !is_ignored_trailing(bytes[i]) {
            return Some(i);
        }
    }
    None
}

fn trailing_rhythm_char(word: &str) -> u8 {
    last_meaningful_char_index(word)
        .map(|i| word.as_bytes()[i])
        .unwrap_or(0)
}

fn trailing_repeated_char_count(word: &str, target: u8) -> usize {
    let mut count = 0usize;
    if let Some(end) = last_meaningful_char_index(word) {
        let bytes = word.as_bytes();
        for i in (0..=end).rev() {
            if bytes[i] != target {
                break;
            }
            count += 1;
        }
    }
    count
}

fn ends_with_ellipsis(word: &str) -> bool {
    trailing_repeated_char_count(word, b'.') >= 3
}

fn starts_with_lowercase(word: &str) -> bool {
    for b in word.bytes() {
        if is_lowercase(b) {
            return true;
        }
        if is_letter(b) {
            return false;
        }
    }
    false
}

fn is_dotted_initialism(word: &str) -> bool {
    let end = match last_meaningful_char_index(word) {
        Some(i) if i > 0 => i,
        _ => return false,
    };
    let bytes = word.as_bytes();
    let mut letter_count_local = 0usize;
    let mut expect_letter = true;
    for i in 0..=end {
        let c = bytes[i];
        if expect_letter {
            if !is_letter(c) {
                return false;
            }
            letter_count_local += 1;
            expect_letter = false;
        } else if c == b'.' {
            expect_letter = true;
        } else {
            return false;
        }
    }
    expect_letter && letter_count_local >= 2
}

const KNOWN_ABBREVIATIONS: &[&str] = &[
    "mr.", "mrs.", "ms.", "dr.", "prof.", "sr.", "jr.", "st.", "vs.", "etc.", "e.g.", "i.e.",
    "cf.", "no.", "fig.", "eq.", "inc.", "ltd.", "co.", "dept.", "mt.", "ft.",
];

fn looks_like_abbreviation(word: &str, next_starts_lowercase: bool) -> bool {
    let lowered: String = word
        .bytes()
        .map(|b| latin::to_lowercase_byte(b) as char)
        .collect();
    for &abbr in KNOWN_ABBREVIATIONS {
        if lowered == abbr {
            return true;
        }
    }
    if !lowered.ends_with('.') {
        return false;
    }
    if is_dotted_initialism(word) {
        return true;
    }
    if readable_char_count(&lowered) <= 2 {
        return true;
    }
    if next_starts_lowercase && readable_char_count(&lowered) <= 4 {
        return true;
    }
    false
}

// ── pacing calculation ────────────────────────────────────────────────────────

fn clamp_pacing_delay(delay_ms: u16) -> u16 {
    delay_ms.min(MAX_PACING_DELAY_MS)
}

fn clamp_scale_percent(percent: u8) -> u8 {
    percent.max(25)
}

fn scaled_percent(base_percent: u16, scale_percent: u8) -> u16 {
    ((base_percent as u32 * clamp_scale_percent(scale_percent) as u32) / 100) as u16
}

fn scaled_delay_ms(bonus_percent: u16, delay_ms: u16) -> u32 {
    (bonus_percent as u32 * clamp_pacing_delay(delay_ms) as u32) / 100
}

fn length_bonus_percent(word: &str) -> u16 {
    let readable = readable_char_count(word);
    if readable == 0 {
        return 0;
    }
    let mut bonus: u16 = 0;

    if readable > LONG_WORD_AFTER_CHARS {
        bonus += ((readable - LONG_WORD_AFTER_CHARS) as u16) * LONG_WORD_PERCENT_PER_CHAR;
    }
    if readable > VERY_LONG_WORD_AFTER_CHARS {
        bonus += ((readable - VERY_LONG_WORD_AFTER_CHARS) as u16) * VERY_LONG_WORD_PERCENT_PER_CHAR;
    }
    if readable > ULTRA_LONG_WORD_AFTER_CHARS {
        bonus +=
            ((readable - ULTRA_LONG_WORD_AFTER_CHARS) as u16) * ULTRA_LONG_WORD_PERCENT_PER_CHAR;
    }

    let joiners = compound_joiner_count(word);
    if joiners > 0 {
        bonus += joiners as u16 * COMPOUND_JOINER_PERCENT;
        if readable >= VERY_LONG_WORD_AFTER_CHARS {
            bonus += LONG_COMPOUND_WORD_PERCENT;
        }
    }

    let tech = technical_connector_count(word);
    if tech > joiners {
        bonus += ((tech - joiners) as u16) * TECHNICAL_CONNECTOR_PERCENT;
    }

    bonus.min(LONG_WORD_MAX_PERCENT)
}

fn complexity_bonus_percent(word: &str) -> u16 {
    let mut bonus: u16 = 0;
    let syllables = approximate_syllable_groups(word);
    if syllables > SYLLABLE_BONUS_AFTER_COUNT {
        let extra = syllables - SYLLABLE_BONUS_AFTER_COUNT;
        bonus +=
            ((extra as u16) * SYLLABLE_BONUS_PERCENT_PER_GROUP).min(SYLLABLE_BONUS_MAX_PERCENT);
    }

    let lc = letter_count(word);
    let dc = digit_count(word);
    let uc = uppercase_count(word);

    if lc > 0 && dc > 0 {
        bonus += MIXED_TOKEN_COMPLEXITY_PERCENT;
    } else if dc >= 3 {
        bonus += NUMERIC_TOKEN_COMPLEXITY_PERCENT;
    }

    if uc >= 2 && uc == lc {
        bonus += ALL_CAPS_COMPLEXITY_PERCENT;
    }

    let tech = technical_connector_count(word);
    if tech >= 2 {
        bonus += ((tech - 1) as u16) * DENSE_CONNECTOR_COMPLEXITY_PERCENT;
    }

    bonus.min(COMPLEX_WORD_MAX_PERCENT)
}

fn punctuation_pause_percent(word: &str, next_starts_lowercase: bool) -> u16 {
    if ends_with_ellipsis(word) {
        return ELLIPSIS_PAUSE_PERCENT;
    }
    match trailing_rhythm_char(word) {
        b',' => COMMA_PAUSE_PERCENT,
        b'-' => DASH_PAUSE_PERCENT,
        b';' | b':' => CLAUSE_PAUSE_PERCENT,
        b'.' => {
            if !looks_like_abbreviation(word, next_starts_lowercase) {
                SENTENCE_PAUSE_PERCENT
            } else {
                0
            }
        }
        b'!' | b'?' => STRONG_SENTENCE_PAUSE_PERCENT,
        _ => 0,
    }
}

fn pacing_bonus_ms(word: &str, next_starts_lowercase: bool, config: &PacingConfig) -> u32 {
    if word.is_empty() {
        return 0;
    }
    scaled_delay_ms(
        scaled_percent(length_bonus_percent(word), config.long_word_scale_percent),
        config.long_word_delay_ms,
    ) + scaled_delay_ms(
        scaled_percent(
            complexity_bonus_percent(word),
            config.complex_word_scale_percent,
        ),
        config.complex_word_delay_ms,
    ) + scaled_delay_ms(
        scaled_percent(
            punctuation_pause_percent(word, next_starts_lowercase),
            config.punctuation_scale_percent,
        ),
        config.punctuation_delay_ms,
    )
}

fn duration_for_word(
    word: &str,
    next_starts_lowercase: bool,
    base_interval_ms: u32,
    config: &PacingConfig,
) -> u32 {
    if base_interval_ms == 0 {
        return 0;
    }
    base_interval_ms + pacing_bonus_ms(word, next_starts_lowercase, config)
}

fn word_ends_sentence(word: &str, next_starts_lowercase: bool) -> bool {
    if ends_with_ellipsis(word) {
        return false;
    }
    match trailing_rhythm_char(word) {
        b'.' => !looks_like_abbreviation(word, next_starts_lowercase),
        b'!' | b'?' => true,
        _ => false,
    }
}

// ── demo words ────────────────────────────────────────────────────────────────

const DEMO_WORDS: &[&str] = &[
    "This",
    "is",
    "the",
    "minimal",
    "RSVP",
    "demo",
    "reader",
    "running",
    "on",
    "the",
    "Waveshare",
    "AMOLED",
    "board.",
    "Rapid",
    "Serial",
    "Visual",
    "Presentation,",
    "or",
    "RSVP,",
    "is",
    "a",
    "reading",
    "technique",
    "that",
    "displays",
    "text",
    "one",
    "word",
    "at",
    "a",
    "time",
    "in",
    "a",
    "fixed",
    "position",
    "on",
    "the",
    "screen.",
    "Instead",
    "of",
    "moving",
    "your",
    "eyes",
    "across",
    "lines",
    "and",
    "paragraphs,",
    "you",
    "keep",
    "your",
    "gaze",
    "locked",
    "on",
    "a",
    "single",
    "point",
    "while",
    "words",
    "flash",
    "in",
    "sequence.",
    "This",
    "eliminates",
    "saccades,",
    "the",
    "small",
    "rapid",
    "eye",
    "movements",
    "that",
    "consume",
    "a",
    "surprising",
    "amount",
    "of",
    "time",
    "during",
    "traditional",
    "reading.",
    "The",
    "speed",
    "is",
    "measured",
    "in",
    "words",
    "per",
    "minute,",
    "or",
    "WPM.",
    "Average",
    "silent",
    "reading",
    "speed",
    "is",
    "around",
    "200",
    "to",
    "250",
    "WPM.",
    "With",
    "RSVP,",
    "many",
    "people",
    "comfortably",
    "reach",
    "300",
    "to",
    "500",
    "WPM",
    "after",
    "a",
    "short",
    "adjustment",
    "period.",
    "RSVP",
    "is",
    "particularly",
    "effective",
    "on",
    "mobile",
    "devices",
    "where",
    "screen",
    "space",
    "is",
    "limited.",
    "You",
    "simply",
    "hold,",
    "read,",
    "and",
    "let",
    "the",
    "words",
    "come",
    "to",
    "you.",
];

// ── ReadingLoop ───────────────────────────────────────────────────────────────

/// RSVP timing engine. Port of C++ ReadingLoop.
pub struct ReadingLoop {
    current_index: usize,
    last_advance_ms: u32,
    wpm: u16,
    pacing_config: PacingConfig,
    current_word: String,
    loaded_words: Vec<String>,
    word_source: Option<Box<dyn crate::reader::word_source::WordSource + Send>>,
}

impl Default for ReadingLoop {
    fn default() -> Self {
        Self {
            current_index: 0,
            last_advance_ms: 0,
            wpm: 300,
            pacing_config: PacingConfig::default(),
            current_word: String::new(),
            loaded_words: Vec::new(),
            word_source: None,
        }
    }
}

impl ReadingLoop {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn begin(&mut self, now_ms: u32) {
        self.current_index = 0;
        self.last_advance_ms = now_ms;
        self.set_current_word_from_index();
    }

    pub fn start(&mut self, now_ms: u32) {
        self.last_advance_ms = now_ms;
    }

    pub fn set_words(&mut self, words: Vec<String>, now_ms: u32) {
        self.word_source = None;
        self.loaded_words = words;
        self.current_index = 0;
        self.last_advance_ms = now_ms;
        self.set_current_word_from_index();
    }

    pub fn set_word_source(
        &mut self,
        source: Box<dyn crate::reader::word_source::WordSource + Send>,
        now_ms: u32,
    ) {
        self.loaded_words.clear();
        self.word_source = Some(source);
        self.current_index = 0;
        self.last_advance_ms = now_ms;
        self.set_current_word_from_index();
    }

    pub fn clear_loaded_book(&mut self, now_ms: u32) {
        self.word_source = None;
        self.loaded_words.clear();
        self.current_index = 0;
        self.last_advance_ms = now_ms;
        self.set_current_word_from_index();
    }

    /// Advance time — returns true if the current word changed.
    pub fn update(&mut self, now_ms: u32, allow_catch_up: bool) -> bool {
        let mut changed = false;
        let max_catch_up = if allow_catch_up {
            MAX_CATCH_UP_WORDS
        } else {
            1
        };

        for _ in 0..max_catch_up {
            let duration_ms = self.current_word_duration_ms();
            if duration_ms == 0 || now_ms.wrapping_sub(self.last_advance_ms) < duration_ms {
                break;
            }
            self.last_advance_ms = self.last_advance_ms.wrapping_add(duration_ms);
            if !self.advance(1) {
                break;
            }
            changed = true;
        }
        changed
    }

    pub fn scrub(&mut self, steps: i32) {
        let base = self.current_index;
        self.seek_relative(base, steps);
    }

    pub fn seek_to(&mut self, word_index: usize) {
        let count = self.word_count();
        if count == 0 {
            self.current_word.clear();
            return;
        }
        self.current_index = word_index.min(count - 1);
        self.set_current_word_from_index();
    }

    pub fn seek_relative(&mut self, base_index: usize, steps: i32) {
        let count = self.word_count();
        if count == 0 {
            return;
        }
        let base = base_index.min(count - 1) as i64;
        let mut next = base + steps as i64;
        if self.using_loaded_book() {
            next = next.clamp(0, (count as i64) - 1);
        } else {
            next = next.rem_euclid(count as i64);
        }
        self.current_index = next as usize;
        self.set_current_word_from_index();
    }

    pub fn rewind_sentence(&mut self) {
        let count = self.word_count();
        if count == 0 {
            return;
        }
        let current_start = self.sentence_start_at_or_before(self.current_index);
        if current_start == self.current_index && self.current_index > 0 {
            let prev_start = self.sentence_start_at_or_before(self.current_index - 1);
            self.seek_to(prev_start);
        } else {
            self.seek_to(current_start);
        }
    }

    pub fn adjust_wpm(&mut self, delta: i32) {
        if delta == 0 {
            return;
        }
        let mut next = self.wpm as i32;
        if delta > 0 {
            next += if next < LOW_WPM_MAX as i32 {
                LOW_WPM_STEP as i32
            } else {
                HIGH_WPM_STEP as i32
            };
            if next > LOW_WPM_MAX as i32 && self.wpm < LOW_WPM_MAX {
                next = LOW_WPM_MAX as i32;
            }
        } else {
            next -= if next <= LOW_WPM_MAX as i32 {
                LOW_WPM_STEP as i32
            } else {
                HIGH_WPM_STEP as i32
            };
            if next < LOW_WPM_MAX as i32 && self.wpm > LOW_WPM_MAX {
                next = LOW_WPM_MAX as i32;
            }
        }
        self.wpm = (next.clamp(MIN_WPM as i32, MAX_WPM as i32)) as u16;
    }

    pub fn set_wpm(&mut self, wpm: u16) {
        self.wpm = wpm.clamp(MIN_WPM, MAX_WPM);
    }

    pub fn set_pacing_config(&mut self, config: PacingConfig) {
        self.pacing_config = config;
    }

    pub fn pacing_config(&self) -> &PacingConfig {
        &self.pacing_config
    }

    pub fn current_word(&self) -> &str {
        &self.current_word
    }

    pub fn word_at(&self, index: usize) -> String {
        if let Some(src) = &self.word_source {
            if index < src.word_count() {
                return src.word_at(index);
            }
        } else if index < self.loaded_words.len() {
            return self.loaded_words[index].clone();
        }
        if index < DEMO_WORDS.len() {
            return DEMO_WORDS[index].to_string();
        }
        String::new()
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn word_count(&self) -> usize {
        if let Some(src) = &self.word_source {
            src.word_count()
        } else if !self.loaded_words.is_empty() {
            self.loaded_words.len()
        } else {
            DEMO_WORDS.len()
        }
    }

    pub fn wpm(&self) -> u16 {
        self.wpm
    }

    pub fn word_interval_ms(&self) -> u32 {
        60_000 / self.wpm as u32
    }

    pub fn current_word_duration_ms(&self) -> u32 {
        let next_idx = self.current_index + 1;
        let next_lower = if next_idx < self.word_count() {
            starts_with_lowercase(&self.word_at(next_idx))
        } else {
            false
        };
        duration_for_word(
            &self.current_word,
            next_lower,
            self.word_interval_ms(),
            &self.pacing_config,
        )
    }

    pub fn word_pacing_bonus_ms_at(&self, index: usize) -> u32 {
        let count = self.word_count();
        if count == 0 || index >= count {
            return 0;
        }
        let word = self.word_at(index);
        let next_lower = self.next_word_starts_lowercase_at(index);
        pacing_bonus_ms(&word, next_lower, &self.pacing_config)
    }

    pub fn elapsed_in_current_word_ms(&self, now_ms: u32) -> u32 {
        if now_ms <= self.last_advance_ms {
            0
        } else {
            now_ms - self.last_advance_ms
        }
    }

    pub fn current_word_ends_sentence(&self) -> bool {
        self.word_ends_sentence_at(self.current_index)
    }

    pub fn at_end(&self) -> bool {
        let count = self.word_count();
        count == 0 || self.current_index + 1 >= count
    }

    // ── private helpers ───────────────────────────────────────────────────────

    fn using_loaded_book(&self) -> bool {
        self.word_source.is_some() || !self.loaded_words.is_empty()
    }

    fn advance(&mut self, steps: usize) -> bool {
        let count = self.word_count();
        if count == 0 {
            return false;
        }
        let next = if self.using_loaded_book() {
            let n = self.current_index + steps;
            if n >= count {
                return false;
            }
            n
        } else {
            (self.current_index + steps) % count
        };
        self.current_index = next;
        self.set_current_word_from_index();
        true
    }

    fn set_current_word_from_index(&mut self) {
        let count = self.word_count();
        if count == 0 {
            self.current_word.clear();
            return;
        }
        let idx = self.current_index.min(count - 1);
        self.current_word = self.word_at(idx);
    }

    fn next_word_starts_lowercase_at(&self, index: usize) -> bool {
        let next = index + 1;
        if next < self.word_count() {
            starts_with_lowercase(&self.word_at(next))
        } else {
            false
        }
    }

    fn word_ends_sentence_at(&self, index: usize) -> bool {
        if index >= self.word_count() {
            return false;
        }
        let word = self.word_at(index);
        let next_lower = self.next_word_starts_lowercase_at(index);
        word_ends_sentence(&word, next_lower)
    }

    fn sentence_start_at_or_before(&self, index: usize) -> usize {
        let count = self.word_count();
        if count == 0 || index == 0 {
            return 0;
        }
        let bounded = index.min(count - 1);
        let mut i = bounded;
        loop {
            if i == 0 {
                break;
            }
            if self.word_ends_sentence_at(i - 1) {
                break;
            }
            i -= 1;
        }
        i
    }
}
