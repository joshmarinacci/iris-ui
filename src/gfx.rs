use crate::font::FontKind;
use crate::geom::{Bounds, Point};
use crate::view::Align;
use embedded_graphics::pixelcolor::PixelColor;

pub struct TextStyle<'a, C: PixelColor> {
    pub halign: Align,
    pub valign: Align,
    pub underline: bool,
    pub font: FontKind,
    pub color: &'a C,
}

impl<'a, C: PixelColor> TextStyle<'a, C> {
    pub fn new(font: FontKind, color: &'a C) -> TextStyle<'a, C> {
        TextStyle {
            font,
            color,
            underline: false,
            valign: Align::Center,
            halign: Align::Start,
        }
    }
    pub fn with_underline(&self, underline: bool) -> Self {
        TextStyle {
            color: self.color,
            font: self.font,
            underline,
            halign: self.halign,
            valign: self.valign,
        }
    }
    pub fn with_halign(&self, halign: Align) -> Self {
        TextStyle {
            color: self.color,
            font: self.font,
            underline: self.underline,
            halign,
            valign: self.valign,
        }
    }
}

pub trait DrawingContext<C: PixelColor> {
    fn fill_rect(&mut self, bounds: &Bounds, color: &C);
    fn stroke_rect(&mut self, bounds: &Bounds, color: &C);
    fn line(&mut self, start: &Point, end: &Point, color: &C);
    fn fill_text(&mut self, bounds: &Bounds, text: &str, style: &TextStyle<C>);
    fn text(&mut self, text: &str, position: &Point, style: &TextStyle<C>);
    fn translate(&mut self, offset: &Point);
    /// Draw a single pixel at logical coordinates `(x, y)` in screen space
    /// (i.e. after the current translation offset has been applied by the
    /// caller).  The coordinate system matches that of `fill_rect` / `line`.
    /// The default implementation is a no-op so existing impls keep compiling.
    fn put_pixel(&mut self, _x: i32, _y: i32, _color: &C) {}
}

pub fn draw_centered_text<C: PixelColor>(
    ctx: &mut dyn DrawingContext<C>,
    text: &str,
    bounds: &Bounds,
    font: FontKind,
    color: &C,
) {
    ctx.text(
        text,
        &bounds.center(),
        &TextStyle {
            font,
            color,
            valign: Align::Center,
            halign: Align::Center,
            underline: false,
        },
    )
}
