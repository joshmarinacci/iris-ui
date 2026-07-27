use embedded_graphics::mono_font::MonoFont;

/// Abstracts over bitmap and (optionally) TrueType fonts.
/// `Copy` so it can be stored by value in `Theme` and `TextStyle` without lifetime friction.
#[derive(Copy, Clone)]
pub enum FontKind {
    /// An embedded-graphics `MonoFont` bitmap font (fixed pixel size, zero allocation).
    Bitmap(MonoFont<'static>),
    /// A fontdue TrueType/OpenType font with a pixel size.
    /// The font must be stored in a `static` so the reference is `'static`.
    #[cfg(feature = "ttf")]
    TrueType {
        font: &'static fontdue::Font,
        size: f32,
    },
}

impl core::fmt::Debug for FontKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FontKind::Bitmap(_) => write!(f, "FontKind::Bitmap"),
            #[cfg(feature = "ttf")]
            FontKind::TrueType { size, .. } => write!(f, "FontKind::TrueType({size}px)"),
        }
    }
}

impl FontKind {
    /// Width of a single character cell — used for layout sizing.
    /// For TrueType, uses the advance width of 'M' as an em-width proxy.
    pub fn char_width(&self) -> i32 {
        match self {
            FontKind::Bitmap(f) => f.character_size.width as i32,
            #[cfg(feature = "ttf")]
            FontKind::TrueType { font, size } => {
                font.metrics('M', *size).advance_width as i32
            }
        }
    }

    /// Height of a character cell — used for row/line height in layout.
    pub fn char_height(&self) -> i32 {
        match self {
            FontKind::Bitmap(f) => f.character_size.height as i32,
            #[cfg(feature = "ttf")]
            FontKind::TrueType { size, .. } => *size as i32,
        }
    }

    /// Baseline offset from the top of the character cell.
    pub fn baseline(&self) -> i32 {
        match self {
            FontKind::Bitmap(f) => f.baseline as i32,
            #[cfg(feature = "ttf")]
            FontKind::TrueType { size, .. } => (*size * 0.75) as i32,
        }
    }

    /// Pixel width of a complete string — used for horizontal alignment.
    pub fn str_width(&self, text: &str) -> i32 {
        match self {
            FontKind::Bitmap(f) => f.character_size.width as i32 * text.len() as i32,
            #[cfg(feature = "ttf")]
            FontKind::TrueType { font, size } => text
                .chars()
                .map(|c| font.metrics(c, *size).advance_width as i32)
                .sum(),
        }
    }
}
