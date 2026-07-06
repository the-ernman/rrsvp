/// Returns true if the byte is in the low custom control slot range.
pub fn is_low_custom_slot_byte(value: u8) -> bool {
    matches!(
        value,
        0x01 | 0x02
            | 0x03
            | 0x04
            | 0x05
            | 0x06
            | 0x07
            | 0x08
            | 0x0E
            | 0x0F
            | 0x10
            | 0x11
            | 0x12
            | 0x13
            | 0x14
            | 0x15
            | 0x16
            | 0x17
    )
}

/// Returns true if the byte is in the repurposed Latin-1 range.
pub fn is_repurposed_latin1_byte(value: u8) -> bool {
    matches!(
        value,
        0xA1 | 0xA2
            | 0xA3
            | 0xA4
            | 0xA5
            | 0xA6
            | 0xA7
            | 0xA8
            | 0xA9
            | 0xAA
            | 0xAB
            | 0xAC
            | 0xAE
            | 0xAF
            | 0xB0
            | 0xB1
            | 0xB2
            | 0xB3
            | 0xB4
            | 0xB5
            | 0xB6
            | 0xB7
            | 0xB8
            | 0xB9
            | 0xBA
            | 0xBB
            | 0xBC
            | 0xBD
            | 0xBE
            | 0xBF
            | 0xD7
            | 0xF7
    )
}

/// Maps a Unicode codepoint to its custom slot byte. Returns None if no mapping.
pub fn custom_slot_for_codepoint(codepoint: u32) -> Option<u8> {
    match codepoint {
        0x010E => Some(0x01),
        0x010F => Some(0x02),
        0x011A => Some(0x03),
        0x011B => Some(0x04),
        0x0147 => Some(0x05),
        0x0148 => Some(0x06),
        0x0158 => Some(0x07),
        0x0159 => Some(0x08),
        0x0164 => Some(0x0E),
        0x0165 => Some(0x0F),
        0x016E => Some(0x10),
        0x016F => Some(0x11),
        0x0150 => Some(0x12),
        0x0151 => Some(0x13),
        0x0170 => Some(0x14),
        0x0171 => Some(0x15),
        0x00A1 => Some(0x16),
        0x00BF => Some(0x17),
        0x0152 => Some(0x80),
        0x0153 => Some(0x81),
        0x0141 => Some(0x82),
        0x0142 => Some(0x83),
        0x010C => Some(0x84),
        0x010D => Some(0x85),
        0x0160 => Some(0x86),
        0x0161 => Some(0x87),
        0x017D => Some(0x88),
        0x017E => Some(0x89),
        0x0102 => Some(0x8A),
        0x0103 => Some(0x8B),
        0x0218 => Some(0x8C),
        0x0219 => Some(0x8D),
        0x021A => Some(0x8E),
        0x021B => Some(0x8F),
        0x011E => Some(0x90),
        0x011F => Some(0x91),
        0x015E => Some(0x92),
        0x015F => Some(0x93),
        0x0130 => Some(0x94),
        0x0131 => Some(0x95),
        0x0104 => Some(0x96),
        0x0105 => Some(0x97),
        0x0118 => Some(0x98),
        0x0119 => Some(0x99),
        0x0106 => Some(0x9A),
        0x0107 => Some(0x9B),
        0x0143 => Some(0x9C),
        0x0144 => Some(0x9D),
        0x015A => Some(0x9E),
        0x015B => Some(0x9F),
        0x0179 => Some(0xB2),
        0x017A => Some(0xB3),
        0x017B => Some(0xB4),
        0x017C => Some(0xB5),
        0x0100 => Some(0xA1),
        0x0101 => Some(0xA2),
        0x0112 => Some(0xA3),
        0x0113 => Some(0xA4),
        0x0122 => Some(0xA5),
        0x0123 => Some(0xA6),
        0x012A => Some(0xA7),
        0x012B => Some(0xA8),
        0x0136 => Some(0xA9),
        0x0137 => Some(0xAA),
        0x013B => Some(0xAB),
        0x013C => Some(0xAC),
        0x0145 => Some(0xAE),
        0x0146 => Some(0xAF),
        0x0116 => Some(0xB0),
        0x0117 => Some(0xB1),
        0x012E => Some(0xB6),
        0x012F => Some(0xB7),
        0x0172 => Some(0xB8),
        0x0173 => Some(0xB9),
        0x016A => Some(0xBA),
        0x016B => Some(0xBB),
        0x0110 => Some(0xBC),
        0x0111 => Some(0xBD),
        0x014A => Some(0xBE),
        0x014B => Some(0xBF),
        0x0166 => Some(0xD7),
        0x0167 => Some(0xF7),
        _ => None,
    }
}

/// Maps a codepoint directly to a storage byte, bypassing custom slots.
fn direct_storage_byte_for_codepoint(codepoint: u32) -> Option<u8> {
    if codepoint < 0x00A1 || codepoint > 0x00FF {
        return None;
    }
    let byte = codepoint as u8;
    if is_repurposed_latin1_byte(byte) {
        return None;
    }
    Some(byte)
}

/// Maps a Unicode codepoint to the storage byte used in the .rsvp format.
pub fn storage_byte_for_codepoint(codepoint: u32) -> Option<u8> {
    if codepoint >= 32 && codepoint <= 126 {
        return Some(codepoint as u8);
    }
    if let Some(slot) = custom_slot_for_codepoint(codepoint) {
        return Some(slot);
    }
    direct_storage_byte_for_codepoint(codepoint)
}

/// Maps a custom-slot uppercase byte to its lowercase equivalent.
fn custom_lowercase_byte(value: u8) -> Option<u8> {
    match value {
        0x01 => Some(0x02),
        0x03 => Some(0x04),
        0x05 => Some(0x06),
        0x07 => Some(0x08),
        0x0E => Some(0x0F),
        0x10 => Some(0x11),
        0x12 => Some(0x13),
        0x14 => Some(0x15),
        0x80 => Some(0x81),
        0x82 => Some(0x83),
        0x84 => Some(0x85),
        0x86 => Some(0x87),
        0x88 => Some(0x89),
        0x8A => Some(0x8B),
        0x8C => Some(0x8D),
        0x8E => Some(0x8F),
        0x90 => Some(0x91),
        0x92 => Some(0x93),
        0x94 => Some(0x95),
        0x96 => Some(0x97),
        0x98 => Some(0x99),
        0x9A => Some(0x9B),
        0x9C => Some(0x9D),
        0x9E => Some(0x9F),
        0xA1 => Some(0xA2),
        0xA3 => Some(0xA4),
        0xA5 => Some(0xA6),
        0xA7 => Some(0xA8),
        0xA9 => Some(0xAA),
        0xAB => Some(0xAC),
        0xAE => Some(0xAF),
        0xB0 => Some(0xB1),
        0xB2 => Some(0xB3),
        0xB4 => Some(0xB5),
        0xB6 => Some(0xB7),
        0xB8 => Some(0xB9),
        0xBA => Some(0xBB),
        0xBC => Some(0xBD),
        0xBE => Some(0xBF),
        0xD7 => Some(0xF7),
        _ => None,
    }
}

fn is_custom_uppercase_letter(value: u8) -> bool {
    custom_lowercase_byte(value).is_some()
}

fn is_custom_lowercase_letter(value: u8) -> bool {
    matches!(
        value,
        0x02 | 0x04
            | 0x06
            | 0x08
            | 0x0F
            | 0x11
            | 0x13
            | 0x15
            | 0x81
            | 0x83
            | 0x85
            | 0x87
            | 0x89
            | 0x8B
            | 0x8D
            | 0x8F
            | 0x91
            | 0x93
            | 0x95
            | 0x97
            | 0x99
            | 0x9B
            | 0x9D
            | 0x9F
            | 0xA2
            | 0xA4
            | 0xA6
            | 0xA8
            | 0xAA
            | 0xAC
            | 0xAF
            | 0xB1
            | 0xB3
            | 0xB5
            | 0xB7
            | 0xB9
            | 0xBB
            | 0xBD
            | 0xBF
            | 0xF7
    )
}

pub fn is_digit(value: u8) -> bool {
    value >= b'0' && value <= b'9'
}

pub fn is_uppercase_letter(value: u8) -> bool {
    (value >= b'A' && value <= b'Z')
        || (value >= 0xC0 && value <= 0xD6)
        || (value >= 0xD8 && value <= 0xDE)
        || is_custom_uppercase_letter(value)
}

pub fn is_lowercase_letter(value: u8) -> bool {
    (value >= b'a' && value <= b'z')
        || value == 0xDF
        || (value >= 0xE0 && value <= 0xF6)
        || (value >= 0xF8)
        || is_custom_lowercase_letter(value)
}

pub fn is_letter(value: u8) -> bool {
    is_uppercase_letter(value) || is_lowercase_letter(value)
}

pub fn is_word_character(value: u8) -> bool {
    is_letter(value) || is_digit(value)
}

pub fn to_lowercase_byte(value: u8) -> u8 {
    if value >= b'A' && value <= b'Z' {
        return value + 32;
    }
    if (value >= 0xC0 && value <= 0xD6) || (value >= 0xD8 && value <= 0xDE) {
        return value + 32;
    }
    if let Some(lc) = custom_lowercase_byte(value) {
        return lc;
    }
    value
}

pub fn is_vowel(value: u8) -> bool {
    matches!(
        to_lowercase_byte(value),
        b'a' | b'e'
            | b'i'
            | b'o'
            | b'u'
            | b'y'
            | 0xE0
            | 0xE1
            | 0xE2
            | 0xE3
            | 0xE4
            | 0xE5
            | 0xE6
            | 0xE8
            | 0xE9
            | 0xEA
            | 0xEB
            | 0xEC
            | 0xED
            | 0xEE
            | 0xEF
            | 0xF2
            | 0xF3
            | 0xF4
            | 0xF5
            | 0xF6
            | 0xF8
            | 0xF9
            | 0xFA
            | 0xFB
            | 0xFC
            | 0xFD
            | 0xFF
            | 0x81
            | 0x8B
            | 0x95
            | 0x97
            | 0x99
            | 0x04
            | 0xA2
            | 0xA4
            | 0xA8
            | 0xB1
            | 0xB7
            | 0xB9
            | 0xBB
            | 0x11
            | 0x13
            | 0x15
    )
}
