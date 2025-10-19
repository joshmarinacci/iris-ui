use embedded_graphics::geometry::Size as ESize;
use embedded_graphics::mono_font::ascii::FONT_7X13_BOLD;
use embedded_graphics::mono_font::iso_8859_9::FONT_7X13;
use embedded_graphics::pixelcolor::{Rgb565, WebColors};
use embedded_graphics::prelude::RgbColor;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use env_logger::Target;
use iris_ui::device::EmbeddedDrawingContext;
use iris_ui::geom::{Bounds, Size};
use iris_ui::scene::{Scene, draw_scene, layout_scene};
use iris_ui::view::{View, ViewId};
use iris_ui::{Theme, ViewStyle};
use log::LevelFilter;
use std::thread::sleep;
use std::time::Duration;

// struct for the state of the progress bar
struct ProgressState {
    value: f32,
}

fn make_progress_bar(name: &ViewId) -> View {
    View {
        name: name.clone(),
        title: "progress".into(),

        // set the state
        state: Some(Box::new(ProgressState { value: 0.0 })),

        // no input
        input: None,

        // fixed size layout
        layout: Some(|e| {
            if let Some(view) = e.scene.get_view_mut(e.target) {
                view.bounds.size = Size::new(100, 20);
            }
        }),

        // draw progress bar
        draw: Some(|e| {
            e.ctx.fill_rect(&e.view.bounds, &e.theme.standard.fill);
            let full = e.view.bounds.size;
            // get the state to calculate the fill width
            if let Some(state) = e.view.get_state::<ProgressState>() {
                let w = (full.w as f32 * state.value) as i32;
                let bd2 = Bounds::new_from(e.view.bounds.position, Size::new(w, full.h));
                e.ctx.fill_rect(&bd2, &e.theme.accented.fill);
            }
            e.ctx.stroke_rect(&e.view.bounds, &e.theme.standard.text);
        }),

        ..Default::default()
    }
}
fn main() -> Result<(), std::convert::Infallible> {
    env_logger::Builder::new()
        .target(Target::Stdout) // <-- redirects to stdout
        .filter(None, LevelFilter::Info)
        .init();

    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(ESize::new(320, 240));

    // reusable ID for the progress bar
    let progress_id = ViewId::new("progress_bar");

    let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));
    scene.add_view_to_root(make_progress_bar(&progress_id));

    let theme = Theme {
        font: FONT_7X13,
        bold_font: FONT_7X13_BOLD,
        standard: ViewStyle {
            fill: Rgb565::WHITE,
            text: Rgb565::BLACK,
        },
        selected: ViewStyle {
            fill: Rgb565::BLUE,
            text: Rgb565::WHITE,
        },
        accented: ViewStyle {
            fill: Rgb565::BLUE,
            text: Rgb565::WHITE,
        },
        panel: ViewStyle {
            fill: Rgb565::CSS_LIGHT_GRAY,
            text: Rgb565::BLACK,
        },
    };

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Simulator Test", &output_settings);
    'running: loop {
        let mut ctx = EmbeddedDrawingContext::new(&mut display);
        ctx.clip = scene.dirty_rect.clone();
        layout_scene(&mut scene, &theme);
        draw_scene(&mut scene, &mut ctx, &theme);
        window.update(&display);

        // for event in window.events() {
        //     match event {
        //         SimulatorEvent::Quit => break 'running,
        //         SimulatorEvent::KeyUp {
        //             keycode,
        //             keymod: _keymod,
        //             repeat: _repeat,
        //         } => {
        //             info!("key is {keycode}");
        //         }
        //         _ => {}
        //     }
        // }

        // update the progress bar every 100 msec
        if let Some(state) = scene.get_view_state::<ProgressState>(&progress_id) {
            state.value += 0.01;
            if state.value > 1.0 {
                state.value = 0.0;
            }
            scene.mark_dirty_view(&progress_id);
            sleep(Duration::from_millis(100));
        }
    }
    Ok(())
}
