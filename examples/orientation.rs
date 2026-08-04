/// Demonstrates display orientation rotation.
///
/// Press R to cycle through 0 / 90 / 180 / 270 degree rotations.
/// The red square always marks the logical origin (top-left corner),
/// so you can clearly see how each rotation re-maps the canvas.
///
/// NOTE: `embedded-graphics-transform 0.1.0` targets embedded-graphics-core 0.3
/// (embedded-graphics 0.7.x) and is incompatible with this project's 0.8.x.
/// Rotation is implemented inline below using the same coordinate transforms.
///
/// Run with:
///   cargo run --example orientation --features std
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rotation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl Rotation {
    fn label(self) -> &'static str {
        match self {
            Rotation::Rotate0 => "0 deg   (press R)",
            Rotation::Rotate90 => "90 deg  (press R)",
            Rotation::Rotate180 => "180 deg (press R)",
            Rotation::Rotate270 => "270 deg (press R)",
        }
    }

    fn next(self) -> Self {
        match self {
            Rotation::Rotate0 => Rotation::Rotate90,
            Rotation::Rotate90 => Rotation::Rotate180,
            Rotation::Rotate180 => Rotation::Rotate270,
            Rotation::Rotate270 => Rotation::Rotate0,
        }
    }
}

/// Wraps a `DrawTarget` and remaps each pixel's coordinates according to `rotation`.
///
/// Coordinate transforms (physical display is `phys_w × phys_h`):
/// - Rotate0:   logical (x, y) → physical (x, y),          logical size = phys_w × phys_h
/// - Rotate90:  logical (x, y) → physical (phys_w-1-y, x), logical size = phys_h × phys_w
/// - Rotate180: logical (x, y) → physical (phys_w-1-x, phys_h-1-y), logical size = phys_w × phys_h
/// - Rotate270: logical (x, y) → physical (y, phys_h-1-x), logical size = phys_h × phys_w
struct RotatedDisplay<'a, D> {
    inner: &'a mut D,
    rotation: Rotation,
    phys_w: i32,
    phys_h: i32,
}

impl<'a, D: Dimensions> RotatedDisplay<'a, D> {
    fn new(display: &'a mut D, rotation: Rotation) -> Self {
        let bb = display.bounding_box();
        RotatedDisplay {
            inner: display,
            rotation,
            phys_w: bb.size.width as i32,
            phys_h: bb.size.height as i32,
        }
    }

    fn transform(&self, p: Point) -> Point {
        match self.rotation {
            Rotation::Rotate0 => p,
            Rotation::Rotate90 => Point::new(self.phys_w - 1 - p.y, p.x),
            Rotation::Rotate180 => Point::new(self.phys_w - 1 - p.x, self.phys_h - 1 - p.y),
            Rotation::Rotate270 => Point::new(p.y, self.phys_h - 1 - p.x),
        }
    }
}

impl<'a, D: Dimensions> Dimensions for RotatedDisplay<'a, D> {
    fn bounding_box(&self) -> Rectangle {
        let (lw, lh) = match self.rotation {
            Rotation::Rotate0 | Rotation::Rotate180 => (self.phys_w as u32, self.phys_h as u32),
            Rotation::Rotate90 | Rotation::Rotate270 => (self.phys_h as u32, self.phys_w as u32),
        };
        Rectangle::new(Point::zero(), Size::new(lw, lh))
    }
}

impl<'a, D: DrawTarget<Color = Rgb565>> DrawTarget for RotatedDisplay<'a, D> {
    type Color = Rgb565;
    type Error = D::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let rotation = self.rotation;
        let phys_w = self.phys_w;
        let phys_h = self.phys_h;
        self.inner
            .draw_iter(pixels.into_iter().map(move |Pixel(p, c)| {
                let q = match rotation {
                    Rotation::Rotate0 => p,
                    Rotation::Rotate90 => Point::new(phys_w - 1 - p.y, p.x),
                    Rotation::Rotate180 => Point::new(phys_w - 1 - p.x, phys_h - 1 - p.y),
                    Rotation::Rotate270 => Point::new(p.y, phys_h - 1 - p.x),
                };
                Pixel(q, c)
            }))
    }
}

/// Draw the scene into whatever canvas `display` exposes.
/// Uses `display.bounding_box()` so content adapts when the canvas is
/// portrait (90°/270°) vs landscape (0°/180°).
fn draw_content(
    display: &mut impl DrawTarget<Color = Rgb565, Error = core::convert::Infallible>,
    rotation: Rotation,
) {
    display.clear(Rgb565::WHITE).unwrap();

    let bb = display.bounding_box();
    let w = bb.size.width as i32;
    let h = bb.size.height as i32;

    // Outer border
    Rectangle::new(
        Point::new(2, 2),
        Size::new(bb.size.width - 4, bb.size.height - 4),
    )
    .into_styled(PrimitiveStyle::with_stroke(Rgb565::BLACK, 2))
    .draw(display)
    .unwrap();

    // Red square in the top-left — always marks the logical origin
    Rectangle::new(Point::new(5, 5), Size::new(40, 40))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
        .draw(display)
        .unwrap();

    // Blue circle in the centre
    Circle::new(Point::new(w / 2 - 30, h / 2 - 30), 60)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLUE))
        .draw(display)
        .unwrap();

    // Green line from bottom-left to top-right (asymmetric diagonal)
    Line::new(Point::new(0, h - 1), Point::new(w - 1, 0))
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 3))
        .draw(display)
        .unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(Rgb565::BLACK)
        .build();
    Text::new(rotation.label(), Point::new(50, 26), text_style)
        .draw(display)
        .unwrap();
}

fn main() {
    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Orientation Demo", &output_settings);
    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(320, 240));
    let mut rotation = Rotation::Rotate0;

    'running: loop {
        // Wrap display in the chosen rotation for this frame, draw, then drop
        // the wrapper so we can call window.update(&display) below.
        {
            let mut rotated = RotatedDisplay::new(&mut display, rotation);
            draw_content(&mut rotated, rotation);
        }
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown {
                    keycode: Keycode::R,
                    ..
                } => {
                    rotation = rotation.next();
                }
                _ => {}
            }
        }
    }
}
