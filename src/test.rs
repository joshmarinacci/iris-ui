use crate::font::FontKind;
use crate::geom::{Bounds, Point};
use crate::gfx::{DrawingContext, TextStyle};
use crate::scene::Scene;
use crate::{Theme, ViewStyle, util};
use embedded_graphics::Drawable;
use embedded_graphics::geometry::Point as EPoint;
use embedded_graphics::mock_display::MockDisplay;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_7X13_BOLD;
use embedded_graphics::mono_font::iso_8859_9::FONT_6X10;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor, WebColors};
use embedded_graphics::primitives::{Line, Primitive, PrimitiveStyle};
use embedded_graphics::text::Text;

pub struct MockDrawingContext {
    pub clip_rect: Bounds,
    pub display: MockDisplay<Rgb565>,
    offset: Point,
}

impl MockDrawingContext {
    pub fn new(scene: &Scene<Rgb565>) -> MockDrawingContext {
        let mut ctx: MockDrawingContext = MockDrawingContext {
            clip_rect: scene.dirty_rect,
            display: MockDisplay::new(),
            offset: Point::new(0, 0),
        };
        ctx.display.set_allow_out_of_bounds_drawing(true);
        ctx.display.set_allow_overdraw(true);
        ctx
    }
    pub fn make_mock_theme() -> Theme<Rgb565> {
        Theme {
            font: FontKind::Bitmap(FONT_6X10),
            bold_font: FontKind::Bitmap(FONT_7X13_BOLD),
            standard: ViewStyle {
                fill: Rgb565::WHITE,
                text: Rgb565::BLACK,
            },
            panel: ViewStyle {
                fill: Rgb565::CSS_GRAY,
                text: Rgb565::BLACK,
            },
            selected: ViewStyle {
                fill: Rgb565::WHITE,
                text: Rgb565::BLACK,
            },
            accented: ViewStyle {
                fill: Rgb565::RED,
                text: Rgb565::WHITE,
            },
        }
    }
}
impl DrawingContext<Rgb565> for MockDrawingContext {
    fn fill_rect(&mut self, bounds: &Bounds, color: &Rgb565) {
        util::bounds_to_rect(bounds)
            .intersection(&util::bounds_to_rect(&self.clip_rect))
            .into_styled(PrimitiveStyle::with_fill(*color))
            .draw(&mut self.display)
            .unwrap();
    }

    fn stroke_rect(&mut self, bounds: &Bounds, color: &Rgb565) {
        util::bounds_to_rect(bounds)
            .intersection(&util::bounds_to_rect(&self.clip_rect))
            .into_styled(PrimitiveStyle::with_stroke(*color, 1))
            .draw(&mut self.display)
            .unwrap();
    }

    fn line(&mut self, start: &Point, end: &Point, color: &Rgb565) {
        let line = Line::new(EPoint::new(start.x, start.y), EPoint::new(end.x, end.y));
        line.into_styled(PrimitiveStyle::with_stroke(*color, 1))
            .draw(&mut self.display)
            .unwrap();
    }

    fn fill_text(&mut self, bounds: &Bounds, text: &str, style: &TextStyle<Rgb565>) {
        match style.font {
            FontKind::Bitmap(mono_font) => {
                let mono_style = MonoTextStyle::new(&mono_font, *style.color);
                let mut pt = EPoint::new(bounds.position.x, bounds.position.y);
                pt.y += bounds.size.h / 2;
                pt.y += (mono_font.baseline as i32) / 2;
                let w = mono_font.character_size.width as i32 * text.len() as i32;
                pt.x += (bounds.size.w - w) / 2;
                Text::new(text, pt, mono_style)
                    .draw(&mut self.display)
                    .unwrap();
            }
            #[cfg(feature = "ttf")]
            FontKind::TrueType { .. } => {
                // No-op in mock context — tests don't require pixel-accurate TTF output
            }
        }
    }

    fn text(&mut self, _text: &str, _position: &Point, _style: &TextStyle<Rgb565>) {}

    fn translate(&mut self, offset: &Point) {
        self.offset = self.offset + *offset;
    }
}
