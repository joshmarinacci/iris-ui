use crate::geom::{Bounds, Point as GPoint};
use crate::gfx::{DrawingContext, TextStyle};
use crate::view::Align;
use core::ops::Add;
use embedded_graphics::Drawable;
use embedded_graphics::draw_target::DrawTargetExt;
use embedded_graphics::geometry::{Point as EPoint, Size as ESize};
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::mono_font::ascii::FONT_6X10;
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
        }
    }
}

fn bounds_to_rect(bounds: &Bounds) -> Rectangle {
    Rectangle::new(
        EPoint::new(bounds.position.x, bounds.position.y),
        ESize::new(bounds.size.w as u32, bounds.size.h as u32),
    )
}

impl<'a, T> DrawingContext for EmbeddedDrawingContext<'a, T>
where
    T: DrawTarget<Color = Rgb565>,
{
    fn fill_rect(&mut self, bounds: &Bounds, color: &Rgb565) {
        let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
        let mut display = display.translated(self.offset);
        bounds_to_rect(bounds)
            .into_styled(PrimitiveStyle::with_fill(*color))
            .draw(&mut display);
    }
    fn stroke_rect(&mut self, bounds: &Bounds, color: &Rgb565) {
        let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
        let mut display = display.translated(self.offset);
        bounds_to_rect(bounds)
            .into_styled(PrimitiveStyle::with_stroke(*color, 1))
            .draw(&mut display);
    }
    fn line(&mut self, start: &GPoint, end: &GPoint, color: &Rgb565) {
        let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
        let mut display = display.translated(self.offset);
        let line = Line::new(EPoint::new(start.x, start.y), EPoint::new(end.x, end.y));
        line.into_styled(PrimitiveStyle::with_stroke(*color, 1))
            .draw(&mut display);
    }
    fn fill_text(&mut self, bounds: &Bounds, text: &str, text_style: &TextStyle) {
        let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
        let mut display = display.translated(self.offset);

        let mut text_builder = MonoTextStyleBuilder::new()
            .font(text_style.font)
            .text_color(*text_style.color);
        if text_style.underline {
            text_builder = text_builder.underline();
        }
        let style = text_builder.build(); // MonoTextStyle::new(&FONT_6X10,  *text_style.color);
        let mut pt = EPoint::new(bounds.position.x, bounds.position.y);
        pt.y += bounds.size.h / 2;
        pt.y += (FONT_6X10.baseline as i32) / 2;

        let w = (FONT_6X10.character_size.width as i32) * (text.len() as i32);

        match text_style.halign {
            Align::Start => {
                pt.x += 5;
            }
            Align::Center => {
                pt.x += (bounds.size.w - w) / 2;
            }
            Align::End => {}
        }

        Text::new(text, pt, style).draw(&mut display);
    }
    fn text(&mut self, text: &str, position: &GPoint, style: &TextStyle) {
        let mut display = self.display.clipped(&bounds_to_rect(&self.clip));
        let mut display = display.translated(self.offset);
        let pt = EPoint::new(position.x, position.y);
        let mut text_builder = MonoTextStyleBuilder::new()
            .font(style.font)
            .text_color(*style.color);
        if style.underline {
            text_builder = text_builder.underline();
        }
        let h_align = match style.halign {
            Align::Start => Alignment::Left,
            Align::Center => Alignment::Center,
            Align::End => Alignment::Right,
        };
        let v_align = match style.valign {
            Align::Start => Baseline::Top,
            Align::Center => Baseline::Middle,
            Align::End => Baseline::Bottom,
        };
        let estyle = text_builder.build();
        let etext = Text {
            position: pt,
            text,
            character_style: estyle,
            text_style: TextStyleBuilder::new()
                .alignment(h_align)
                .baseline(v_align)
                .build(),
        };
        etext.draw(&mut display);
    }
    fn translate(&mut self, offset: &GPoint) {
        self.offset = self.offset.add(EPoint::new(offset.x, offset.y));
    }
}
