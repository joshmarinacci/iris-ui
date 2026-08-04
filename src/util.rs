use crate::font::FontKind;
use crate::geom::{Bounds, Size};
use embedded_graphics::geometry::Size as ESize;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::primitives::Rectangle;

pub fn calc_bounds(bounds: Bounds, font: FontKind, title: &str) -> Bounds {
    let cw = font.char_width();
    let ch = font.char_height();
    let width = font.str_width(title) + cw * 2;
    let height = ch + (ch / 2) * 2;
    Bounds::new(bounds.x(), bounds.y(), width, height)
}

pub fn calc_size(font: FontKind, title: &str) -> Size {
    let cw = font.char_width();
    let ch = font.char_height();
    let width = font.str_width(title) + cw * 2;
    let height = ch + (ch / 2) * 2;
    Size {
        w: width,
        h: height,
    }
}

pub fn bounds_to_rect(bounds: &Bounds) -> Rectangle {
    if bounds.is_empty() {
        return Rectangle::zero();
    }
    Rectangle::new(
        embedded_graphics::geometry::Point::new(bounds.position.x, bounds.position.y),
        ESize::new(bounds.size.w as u32, bounds.size.h as u32),
    )
}

/// Convert a hex character (0-9, A-F, a-f) to a number, compile-time safe
const fn hex_char_to_digit(c: u8) -> u8 {
    if c >= b'0' && c <= b'9' {
        c - b'0'
    } else if c >= b'a' && c <= b'f' {
        c - b'a' + 10
    } else if c >= b'A' && c <= b'F' {
        c - b'A' + 10
    } else {
        panic!("Invalid hex character in hex string. Use 0-9, a-f, or A-F.");
    }
}

/// Parse two hex characters into a u8
const fn parse_hex_byte(s: &[u8], i: usize) -> u8 {
    (hex_char_to_digit(s[i]) << 4) | hex_char_to_digit(s[i + 1])
}

/// Convert a hex string "#RRGGBB" or "RRGGBB" to Rgb565
pub const fn hex_str_to_rgb565(s: &str) -> Rgb565 {
    let bytes = s.as_bytes();
    let start = if bytes.len() == 7 && bytes[0] == b'#' {
        1
    } else if bytes.len() == 6 {
        0
    } else {
        panic!("Hex string must be in the format \"#RRGGBB\" or \"RRGGBB\"");
    };

    let r = parse_hex_byte(bytes, start) as u16;
    let g = parse_hex_byte(bytes, start + 2) as u16;
    let b = parse_hex_byte(bytes, start + 4) as u16;

    let r5 = ((r * 31 + 127) / 255) as u8;
    let g6 = ((g * 63 + 127) / 255) as u8;
    let b5 = ((b * 31 + 127) / 255) as u8;

    Rgb565::new(r5, g6, b5)
}
