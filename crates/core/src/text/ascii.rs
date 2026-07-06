pub const fn is_whitespace(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r' | 0x0C | 0x0B)
}

pub const fn is_alpha(c: u8) -> bool {
    (c >= b'A' && c <= b'Z') || (c >= b'a' && c <= b'z')
}

pub const fn is_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

pub const fn is_alphanumeric(c: u8) -> bool {
    is_alpha(c) || is_digit(c)
}

pub const fn is_hex_digit(c: u8) -> bool {
    is_digit(c) || (c >= b'a' && c <= b'f') || (c >= b'A' && c <= b'F')
}

pub const fn hex_value(c: u8) -> i32 {
    if is_digit(c) {
        (c - b'0') as i32
    } else if c >= b'a' && c <= b'f' {
        (c - b'a' + 10) as i32
    } else if c >= b'A' && c <= b'F' {
        (c - b'A' + 10) as i32
    } else {
        -1
    }
}

pub const fn to_lower(c: u8) -> u8 {
    if c >= b'A' && c <= b'Z' {
        c + (b'a' - b'A')
    } else {
        c
    }
}
