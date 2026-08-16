use crate::font::FontKind;
use crate::geom::{Bounds, Point as GPoint};
use crate::gfx::{DrawingContext, TextStyle};
use crate::view::Align;
use core::ops::Add;
use embedded_graphics::Drawable;
use embedded_graphics::Pixel;
use embedded_graphics::draw_target::DrawTargetExt;
use embedded_graphics::geometry::{Dimensions, Point as EPoint, Size as ESize};
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::{BinaryColor, PixelColor, Rgb565};
use embedded_graphics::prelude::{DrawTarget, RgbColor};
use embedded_graphics::primitives::{Line, Primitive, PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Alignment, Baseline, Text, TextStyleBuilder};

/// Converts a logical/theme color `Src` into the display's native color type.
/// Implement this for any (native, logical) color pair you want to drive with
/// `EmbeddedDrawingContext`.
pub trait FromColor<Src: PixelColor>: PixelColor {
    fn from_color(color: Src) -> Self;
}

/// Identity conversion: a display whose native color matches the theme's logical color.
impl<C: PixelColor> FromColor<C> for C {
    #[inline]
    fn from_color(color: C) -> Self {
        color
    }
}

impl FromColor<Rgb565> for BinaryColor {
    /// Black maps to `On` (ink/foreground); any other color maps to `Off` (paper/background).
    /// This matches `LcdWhite` where On=dark ink, Off=light paper.
    #[inline]
    fn from_color(color: Rgb565) -> Self {
        if color == Rgb565::BLACK {
            BinaryColor::On
        } else {
            BinaryColor::Off
        }
    }
}

pub struct EmbeddedDrawingContext<'a, T, C = <T as DrawTarget>::Color>
where
    T: DrawTarget,
    T::Color: FromColor<C>,
    C: PixelColor,
{
    pub display: &'a mut T,
    pub clip: Bounds,
    offset: EPoint,
    scale: u32,
    _logical_color: core::marker::PhantomData<C>,
}

impl<'a, T, C> EmbeddedDrawingContext<'a, T, C>
where
    T: DrawTarget,
    T::Color: FromColor<C>,
    C: PixelColor,
{
    pub fn new(display: &'a mut T) -> Self {
        EmbeddedDrawingContext {
            display,
            clip: Bounds::new_empty(),
            offset: EPoint::new(0, 0),
            scale: 1,
            _logical_color: core::marker::PhantomData,
        }
    }

    pub fn new_with_scale(display: &'a mut T, scale: u32) -> Self {
        EmbeddedDrawingContext {
            display,
            clip: Bounds::new_empty(),
            offset: EPoint::new(0, 0),
            scale,
            _logical_color: core::marker::PhantomData,
        }
    }
}

fn bounds_to_rect(bounds: &Bounds) -> Rectangle {
    Rectangle::new(
        EPoint::new(bounds.position.x, bounds.position.y),
        ESize::new(bounds.size.w as u32, bounds.size.h as u32),
    )
}

fn bounds_to_scaled_rect(bounds: &Bounds, scale: u32) -> Rectangle {
    let s = scale as i32;
    Rectangle::new(
        EPoint::new(bounds.position.x * s, bounds.position.y * s),
        ESize::new((bounds.size.w * s) as u32, (bounds.size.h * s) as u32),
    )
}

/// Wraps a DrawTarget so that each drawn pixel becomes a scale×scale block.
/// Accepts logical coordinates; emits physical pixels to the inner display.
struct ScaledDisplay<T> {
    inner: T,
    scale: u32,
}

impl<T: DrawTarget> Dimensions for ScaledDisplay<T> {
    fn bounding_box(&self) -> Rectangle {
        let bb = self.inner.bounding_box();
        let s = self.scale as i32;
        Rectangle::new(
            EPoint::new(bb.top_left.x / s, bb.top_left.y / s),
            ESize::new(bb.size.width / self.scale, bb.size.height / self.scale),
        )
    }
}

impl<T: DrawTarget> DrawTarget for ScaledDisplay<T> {
    type Color = T::Color;
    type Error = T::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let s = self.scale as i32;
        for Pixel(point, color) in pixels {
            let _ = Rectangle::new(
                EPoint::new(point.x * s, point.y * s),
                ESize::new(self.scale, self.scale),
            )
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(&mut self.inner);
        }
        Ok(())
    }
}

/// Rasterize a TrueType string glyph-by-glyph and draw each pixel to `display`.
/// `baseline_y` is the screen y-coordinate of the text baseline.
/// Alpha values above 127 are drawn; lower values are skipped (binary threshold).
#[cfg(feature = "ttf")]
fn draw_ttf_glyphs<T: DrawTarget>(
    display: &mut T,
    text: &str,
    font: &fontdue::Font,
    size: f32,
    mut cursor_x: i32,
    baseline_y: i32,
    color: T::Color,
) {
    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, size);
        for row in 0..metrics.height {
            for col in 0..metrics.width {
                if bitmap[row * metrics.width + col] > 127 {
                    let px = cursor_x + metrics.xmin + col as i32;
                    // row=0 is the top of the bitmap; ymin is the signed distance
                    // from the baseline to the bottom of the glyph bounding box.
                    let py = baseline_y - metrics.ymin - metrics.height as i32 + row as i32;
                    let _ = display.draw_iter(core::iter::once(Pixel(EPoint::new(px, py), color)));
                }
            }
        }
        cursor_x += (metrics.advance_width + 0.5) as i32;
    }
}

impl<'a, T, C> DrawingContext<C> for EmbeddedDrawingContext<'a, T, C>
where
    T: DrawTarget,
    T::Color: FromColor<C>,
    C: PixelColor,
{
    fn fill_rect(&mut self, bounds: &Bounds, color: &C) {
        let c = T::Color::from_color(*color);
        let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
        let mut display = display.translated(self.offset);
        let _ = bounds_to_scaled_rect(bounds, self.scale)
            .into_styled(PrimitiveStyle::with_fill(c))
            .draw(&mut display);
    }

    fn stroke_rect(&mut self, bounds: &Bounds, color: &C) {
        let c = T::Color::from_color(*color);
        let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
        let mut display = display.translated(self.offset);
        let _ = bounds_to_scaled_rect(bounds, self.scale)
            .into_styled(PrimitiveStyle::with_stroke(c, self.scale))
            .draw(&mut display);
    }

    fn line(&mut self, start: &GPoint, end: &GPoint, color: &C) {
        let c = T::Color::from_color(*color);
        let s = self.scale as i32;
        let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
        let mut display = display.translated(self.offset);
        let line = Line::new(
            EPoint::new(start.x * s, start.y * s),
            EPoint::new(end.x * s, end.y * s),
        );
        let _ = line
            .into_styled(PrimitiveStyle::with_stroke(c, self.scale))
            .draw(&mut display);
    }

    fn fill_text(&mut self, bounds: &Bounds, text: &str, text_style: &TextStyle<C>) {
        match text_style.font {
            FontKind::Bitmap(mono_font) => {
                let c = T::Color::from_color(*text_style.color);
                let mut text_builder = MonoTextStyleBuilder::new().font(&mono_font).text_color(c);
                if text_style.underline {
                    text_builder = text_builder.underline();
                }
                let style = text_builder.build();
                let w = mono_font.character_size.width as i32 * text.len() as i32;

                if self.scale == 1 {
                    let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
                    let mut display = display.translated(self.offset);
                    let mut pt = EPoint::new(bounds.position.x, bounds.position.y);
                    pt.y += bounds.size.h / 2;
                    pt.y += (mono_font.baseline as i32) / 2;
                    match text_style.halign {
                        Align::Start => pt.x += 5,
                        Align::Center => pt.x += (bounds.size.w - w) / 2,
                        Align::End => {}
                    }
                    let _ = Text::new(text, pt, style).draw(&mut display);
                } else {
                    let s = self.scale as i32;
                    let clipped = self.display.clipped(&bounds_to_rect(&self.clip));
                    let mut scaled = ScaledDisplay {
                        inner: clipped,
                        scale: self.scale,
                    };
                    let logical_offset = EPoint::new(self.offset.x / s, self.offset.y / s);
                    let mut display = scaled.translated(logical_offset);
                    let mut pt = EPoint::new(bounds.position.x, bounds.position.y);
                    pt.y += bounds.size.h / 2;
                    pt.y += (mono_font.baseline as i32) / 2;
                    match text_style.halign {
                        Align::Start => pt.x += 5,
                        Align::Center => pt.x += (bounds.size.w - w) / 2,
                        Align::End => {}
                    }
                    let _ = Text::new(text, pt, style).draw(&mut display);
                }
            }

            #[cfg(feature = "ttf")]
            FontKind::TrueType { font, size } => {
                let total_w: i32 = text
                    .chars()
                    .map(|c| (font.metrics(c, size).advance_width + 0.5) as i32)
                    .sum();
                // Glyphs draw starting at cursor_x + xmin, not cursor_x.
                // Subtract the first character's xmin so the visible text is centered,
                // not the cursor range.
                let first_xmin = text
                    .chars()
                    .next()
                    .map(|c| font.metrics(c, size).xmin)
                    .unwrap_or(0);
                let start_x = match text_style.halign {
                    Align::Start => bounds.position.x + 5,
                    Align::Center => bounds.position.x + (bounds.size.w - total_w - first_xmin) / 2,
                    Align::End => bounds.position.x + bounds.size.w - total_w - first_xmin,
                };
                // Center the baseline vertically within bounds
                let baseline_y = bounds.position.y + bounds.size.h / 2 + (size * 0.25) as i32;
                let color = T::Color::from_color(*text_style.color);

                if self.scale == 1 {
                    let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
                    let mut display = display.translated(self.offset);
                    draw_ttf_glyphs(&mut display, text, font, size, start_x, baseline_y, color);
                } else {
                    let s = self.scale as i32;
                    let clipped = self.display.clipped(&bounds_to_rect(&self.clip));
                    let mut scaled = ScaledDisplay {
                        inner: clipped,
                        scale: self.scale,
                    };
                    let logical_offset = EPoint::new(self.offset.x / s, self.offset.y / s);
                    let mut display = scaled.translated(logical_offset);
                    draw_ttf_glyphs(&mut display, text, font, size, start_x, baseline_y, color);
                }
            }
        }
    }

    fn text(&mut self, text: &str, position: &GPoint, style: &TextStyle<C>) {
        match style.font {
            FontKind::Bitmap(mono_font) => {
                let c = T::Color::from_color(*style.color);
                let mut text_builder = MonoTextStyleBuilder::new().font(&mono_font).text_color(c);
                if style.underline {
                    text_builder = text_builder.underline();
                }
                let estyle = text_builder.build();
                let text_style = TextStyleBuilder::new()
                    .alignment(Alignment::Center)
                    .baseline(Baseline::Middle)
                    .build();

                if self.scale == 1 {
                    let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
                    let mut display = display.translated(self.offset);
                    let pt = EPoint::new(position.x, position.y);
                    let _ = Text {
                        position: pt,
                        text,
                        character_style: estyle,
                        text_style,
                    }
                    .draw(&mut display);
                } else {
                    let s = self.scale as i32;
                    let clipped = self.display.clipped(&bounds_to_rect(&self.clip));
                    let mut scaled = ScaledDisplay {
                        inner: clipped,
                        scale: self.scale,
                    };
                    let logical_offset = EPoint::new(self.offset.x / s, self.offset.y / s);
                    let mut display = scaled.translated(logical_offset);
                    let pt = EPoint::new(position.x, position.y);
                    let _ = Text {
                        position: pt,
                        text,
                        character_style: estyle,
                        text_style,
                    }
                    .draw(&mut display);
                }
            }

            #[cfg(feature = "ttf")]
            FontKind::TrueType { font, size } => {
                let total_w: i32 = text
                    .chars()
                    .map(|c| (font.metrics(c, size).advance_width + 0.5) as i32)
                    .sum();
                let first_xmin = text
                    .chars()
                    .next()
                    .map(|c| font.metrics(c, size).xmin)
                    .unwrap_or(0);
                let cursor_x = match style.halign {
                    Align::Center => position.x - (total_w + first_xmin) / 2,
                    Align::End => position.x - total_w,
                    Align::Start => position.x,
                };
                let baseline_y = match style.valign {
                    Align::Center => {
                        if let Some(lm) = font.horizontal_line_metrics(size) {
                            position.y + ((lm.ascent + lm.descent) * 0.5) as i32
                        } else {
                            position.y + (size * 0.25) as i32
                        }
                    }
                    Align::Start | Align::End => position.y,
                };
                let color = T::Color::from_color(*style.color);
                if self.scale == 1 {
                    let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
                    let mut display = display.translated(self.offset);
                    draw_ttf_glyphs(&mut display, text, font, size, cursor_x, baseline_y, color);
                } else {
                    let s = self.scale as i32;
                    let clipped = self.display.clipped(&bounds_to_rect(&self.clip));
                    let mut scaled = ScaledDisplay {
                        inner: clipped,
                        scale: self.scale,
                    };
                    let logical_offset = EPoint::new(self.offset.x / s, self.offset.y / s);
                    let mut display = scaled.translated(logical_offset);
                    draw_ttf_glyphs(&mut display, text, font, size, cursor_x, baseline_y, color);
                }
            }
        }
    }

    fn translate(&mut self, offset: &GPoint) {
        let s = self.scale as i32;
        self.offset = self.offset.add(EPoint::new(offset.x * s, offset.y * s));
    }

    fn put_pixel(&mut self, x: i32, y: i32, color: &C) {
        let c = T::Color::from_color(*color);
        if self.scale == 1 {
            let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
            let mut display = display.translated(self.offset);
            let _ = display.draw_iter(core::iter::once(Pixel(EPoint::new(x, y), c)));
        } else {
            let s = self.scale as i32;
            let clipped = self.display.clipped(&bounds_to_rect(&self.clip));
            let mut scaled = ScaledDisplay {
                inner: clipped,
                scale: self.scale,
            };
            let logical_offset = EPoint::new(self.offset.x / s, self.offset.y / s);
            let mut display = scaled.translated(logical_offset);
            let _ = display.draw_iter(core::iter::once(Pixel(EPoint::new(x, y), c)));
        }
    }
}
