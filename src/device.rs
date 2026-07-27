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
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::DrawTarget;
use embedded_graphics::primitives::{Line, Primitive, PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Alignment, Baseline, Text, TextStyleBuilder};

pub struct EmbeddedDrawingContext<'a, T>
where
    T: DrawTarget<Color = Rgb565>,
{
    pub display: &'a mut T,
    pub clip: Bounds,
    offset: EPoint,
    scale: u32,
}

impl<'a, T> EmbeddedDrawingContext<'a, T>
where
    T: DrawTarget<Color = Rgb565>,
{
    pub fn new(display: &'a mut T) -> Self {
        EmbeddedDrawingContext {
            display,
            clip: Bounds::new_empty(),
            offset: EPoint::new(0, 0),
            scale: 1,
        }
    }

    pub fn new_with_scale(display: &'a mut T, scale: u32) -> Self {
        EmbeddedDrawingContext {
            display,
            clip: Bounds::new_empty(),
            offset: EPoint::new(0, 0),
            scale,
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

impl<T: DrawTarget<Color = Rgb565>> Dimensions for ScaledDisplay<T> {
    fn bounding_box(&self) -> Rectangle {
        let bb = self.inner.bounding_box();
        let s = self.scale as i32;
        Rectangle::new(
            EPoint::new(bb.top_left.x / s, bb.top_left.y / s),
            ESize::new(bb.size.width / self.scale, bb.size.height / self.scale),
        )
    }
}

impl<T: DrawTarget<Color = Rgb565>> DrawTarget for ScaledDisplay<T> {
    type Color = Rgb565;
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
fn draw_ttf_glyphs<T: DrawTarget<Color = Rgb565>>(
    display: &mut T,
    text: &str,
    font: &fontdue::Font,
    size: f32,
    mut cursor_x: i32,
    baseline_y: i32,
    color: Rgb565,
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
                    let _ = display
                        .draw_iter(core::iter::once(Pixel(EPoint::new(px, py), color)));
                }
            }
        }
        cursor_x += metrics.advance_width as i32;
    }
}

impl<'a, T> DrawingContext for EmbeddedDrawingContext<'a, T>
where
    T: DrawTarget<Color = Rgb565>,
{
    fn fill_rect(&mut self, bounds: &Bounds, color: &Rgb565) {
        let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
        let mut display = display.translated(self.offset);
        let _ = bounds_to_scaled_rect(bounds, self.scale)
            .into_styled(PrimitiveStyle::with_fill(*color))
            .draw(&mut display);
    }

    fn stroke_rect(&mut self, bounds: &Bounds, color: &Rgb565) {
        let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
        let mut display = display.translated(self.offset);
        let _ = bounds_to_scaled_rect(bounds, self.scale)
            .into_styled(PrimitiveStyle::with_stroke(*color, self.scale))
            .draw(&mut display);
    }

    fn line(&mut self, start: &GPoint, end: &GPoint, color: &Rgb565) {
        let s = self.scale as i32;
        let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
        let mut display = display.translated(self.offset);
        let line = Line::new(
            EPoint::new(start.x * s, start.y * s),
            EPoint::new(end.x * s, end.y * s),
        );
        let _ = line
            .into_styled(PrimitiveStyle::with_stroke(*color, self.scale))
            .draw(&mut display);
    }

    fn fill_text(&mut self, bounds: &Bounds, text: &str, text_style: &TextStyle) {
        match text_style.font {
            FontKind::Bitmap(mono_font) => {
                let mut text_builder = MonoTextStyleBuilder::new()
                    .font(&mono_font)
                    .text_color(*text_style.color);
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
                    let mut scaled = ScaledDisplay { inner: clipped, scale: self.scale };
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
                    .map(|c| font.metrics(c, size).advance_width as i32)
                    .sum();
                let start_x = match text_style.halign {
                    Align::Start => bounds.position.x + 5,
                    Align::Center => bounds.position.x + (bounds.size.w - total_w) / 2,
                    Align::End => bounds.position.x + bounds.size.w - total_w,
                };
                // Center the baseline vertically within bounds
                let baseline_y = bounds.position.y + bounds.size.h / 2 + (size * 0.25) as i32;

                if self.scale == 1 {
                    let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
                    let mut display = display.translated(self.offset);
                    draw_ttf_glyphs(&mut display, text, font, size, start_x, baseline_y, *text_style.color);
                } else {
                    let s = self.scale as i32;
                    let clipped = self.display.clipped(&bounds_to_rect(&self.clip));
                    let mut scaled = ScaledDisplay { inner: clipped, scale: self.scale };
                    let logical_offset = EPoint::new(self.offset.x / s, self.offset.y / s);
                    let mut display = scaled.translated(logical_offset);
                    draw_ttf_glyphs(&mut display, text, font, size, start_x, baseline_y, *text_style.color);
                }
            }
        }
    }

    fn text(&mut self, text: &str, position: &GPoint, style: &TextStyle) {
        match style.font {
            FontKind::Bitmap(mono_font) => {
                let mut text_builder = MonoTextStyleBuilder::new()
                    .font(&mono_font)
                    .text_color(*style.color);
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
                    let _ = Text { position: pt, text, character_style: estyle, text_style }
                        .draw(&mut display);
                } else {
                    let s = self.scale as i32;
                    let clipped = self.display.clipped(&bounds_to_rect(&self.clip));
                    let mut scaled = ScaledDisplay { inner: clipped, scale: self.scale };
                    let logical_offset = EPoint::new(self.offset.x / s, self.offset.y / s);
                    let mut display = scaled.translated(logical_offset);
                    let pt = EPoint::new(position.x, position.y);
                    let _ = Text { position: pt, text, character_style: estyle, text_style }
                        .draw(&mut display);
                }
            }

            #[cfg(feature = "ttf")]
            FontKind::TrueType { font, size } => {
                // For the free-position `text()` method, use position.y as the baseline
                if self.scale == 1 {
                    let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
                    let mut display = display.translated(self.offset);
                    draw_ttf_glyphs(&mut display, text, font, size, position.x, position.y, *style.color);
                } else {
                    let s = self.scale as i32;
                    let clipped = self.display.clipped(&bounds_to_rect(&self.clip));
                    let mut scaled = ScaledDisplay { inner: clipped, scale: self.scale };
                    let logical_offset = EPoint::new(self.offset.x / s, self.offset.y / s);
                    let mut display = scaled.translated(logical_offset);
                    draw_ttf_glyphs(&mut display, text, font, size, position.x, position.y, *style.color);
                }
            }
        }
    }

    fn translate(&mut self, offset: &GPoint) {
        let s = self.scale as i32;
        self.offset = self.offset.add(EPoint::new(offset.x * s, offset.y * s));
    }
}
