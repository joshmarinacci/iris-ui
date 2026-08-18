#[cfg(feature = "std")]
use embedded_graphics::geometry::{Point as EPoint, Size};
use embedded_graphics::mono_font::ascii::{
    FONT_5X7, FONT_6X10, FONT_7X13_BOLD, FONT_9X15, FONT_9X15_BOLD,
};
use embedded_graphics::mono_font::iso_8859_9::FONT_7X13;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::prelude::WebColors;
use iris_ui::button::{make_button, make_full_button};
use iris_ui::geom::{Bounds, Insets, Point as GPoint};
use iris_ui::scene::{
    Scene, draw_scene, event_at_focused, layout_scene, pointer_down_at, pointer_up_at,
};
use iris_ui::toggle_button::make_toggle_button;
use iris_ui::toggle_group::{SelectOneOfState, layout_toggle_group, make_toggle_group};
use iris_ui::{BW_THEME, FontKind, Theme, ViewStyle, util};
use std::convert::Into;

use embedded_graphics::pixelcolor::{BinaryColor, Rgb888};
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::sdl2::{Keycode, Mod};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettings, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent,
    Window,
};
use env_logger::Target;
use iris_ui::device::{EmbeddedDrawingContext, FromColor};
use iris_ui::grid::{GridLayoutState, LayoutConstraint, make_grid_panel};
use iris_ui::input::{InputEvent, InputResult, OutputAction, TextAction};
use iris_ui::label::{make_header_label, make_label};
use iris_ui::layouts::{layout_hbox, layout_std_panel, layout_vbox};
use iris_ui::list_view::make_list_view;
use iris_ui::panel::{PanelState, draw_std_panel, make_panel};
use iris_ui::tabbed_panel::{LayoutPanelState, make_tabbed_panel};
use iris_ui::text_input::make_text_input;
use iris_ui::util::hex_str_to_rgb565;
use iris_ui::view::Align::{Center, Start};
use iris_ui::view::Flex::{Fixed, Grow, Shrink};
use iris_ui::view::{Align, Flex, View, ViewId};
use log::{LevelFilter, info};


fn make_scene(scale: u32) -> Scene<Rgb565> {
    let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));
    // {
    //     let button_id = ViewId::new("ok");
    //     let button = make_button(&button_id, "Greetings, Earthling");
    //     scene.add_view_to_root(button);
    // }

    // {
    //     let toolbar_id = ViewId::new("toolbar");
    //     let toolbar = make_panel(&toolbar_id)
    //         .with_layout(Some(layout_hbox));
    //     scene.add_view_to_root(toolbar);
    //
    //     let button1_id = ViewId::new("button1");
    //     let button1 = make_button(&button1_id, "Apple");
    //     scene.add_view_to_parent(button1, &toolbar_id);
    //
    //     let button2_id = ViewId::new("button2");
    //     let button2 = make_button(&button2_id, "Bear");
    //     scene.add_view_to_parent(button2, &toolbar_id);
    //
    //     let button3_id = ViewId::new("button3");
    //     let button3 = make_button(&button3_id, "Cat");
    //     scene.add_view_to_parent(button3, &toolbar_id);
    // }

    // {
    //     let toolbar_id = ViewId::new("toolbar");
    //
    //     scene.add_view_to_root(
    //         make_panel(&toolbar_id)
    //             .with_layout(Some(layout_hbox))
    //     );
    //
    //     scene.add_view_to_parent(make_button(&ViewId::new("button1"), "Apple"), &toolbar_id);
    //     scene.add_view_to_parent(make_button(&ViewId::new("button2"), "Bar"), &toolbar_id);
    //     scene.add_view_to_parent(make_button(&ViewId::new("button3"), "Cat"), &toolbar_id);
    //
    // }

    {
        let toolbar_id = ViewId::new("toolbar");

        scene.add_view_to_root(
            make_panel(&toolbar_id)
                .with_layout(Some(layout_hbox))
                .with_h_align(Align::Center)
                .with_v_align(Align::Center)
                .with_h_flex(Flex::Shrink)
                .with_v_flex(Flex::Shrink)
        );

        scene.add_view_to_parent(make_button(&ViewId::new("button1"), "Apple"), &toolbar_id);
        scene.add_view_to_parent(make_button(&ViewId::new("button2"), "Bar"), &toolbar_id);
        scene.add_view_to_parent(make_button(&ViewId::new("button3"), "Cat"), &toolbar_id);
    }

    scene

}

fn run_loop<C>(
    mut display: SimulatorDisplay<C>,
    output_settings: &OutputSettings,
    mut scene: Scene<Rgb565>,
    mut theme: Theme<Rgb565>,
) where
    C: PixelColor + FromColor<Rgb565> + Into<Rgb888> + From<Rgb888>,
{
    let scale = scene.scale();
    let mut window = Window::new("Simulator Test", output_settings);
    'running: loop {
        let mut ctx = EmbeddedDrawingContext::new_with_scale(&mut display, scale);
        ctx.clip = scene.dirty_rect.scaled(scale);
        layout_scene(&mut scene, &theme);
        draw_scene(&mut scene, &mut ctx, &theme);
        window.update(&display);
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown {
                    keycode, keymod, ..
                } => {
                    let act: TextAction = keydown_to_char(keycode, keymod);
                    let evt = InputEvent::Text(act);
                    if let Some(result) = event_at_focused(&mut scene, &evt) {
                        println!("got input from {:?}", result);
                    }
                }
                SimulatorEvent::MouseButtonUp { point, .. } => {
                    println!("mouse button up {}", point);
                    let lpt = GPoint::new(point.x / scale as i32, point.y / scale as i32);
                    if let Some(result) = pointer_up_at(&mut scene, &vec![], lpt) {
                        handle_events(result, &mut scene, &mut theme);
                    }
                }
                SimulatorEvent::MouseButtonDown { point, .. } => {
                    println!("mouse button down {}", point);
                    let lpt = GPoint::new(point.x / scale as i32, point.y / scale as i32);
                    pointer_down_at(&mut scene, &vec![], lpt);
                }
                SimulatorEvent::MouseWheel {
                    scroll_delta,
                    direction,
                } => {
                    info!("mouse wheel {scroll_delta:?} {direction:?}");
                    if let Some(result) = event_at_focused(
                        &mut scene,
                        &InputEvent::Scroll(GPoint::new(scroll_delta.x, scroll_delta.y)),
                    ) {
                        println!("got input from {:?}", result);
                    }
                }
                _ => {}
            }
        }
    }
}

fn main() -> Result<(), std::convert::Infallible> {
    env_logger::Builder::new()
        .target(Target::Stdout)
        .filter(None, LevelFilter::Info)
        .init();

    let scale: u32 = std::env::args()
        .find(|a| a.starts_with("--scale="))
        .and_then(|a| a["--scale=".len()..].parse().ok())
        .unwrap_or(1);
    let epaper = std::env::args().any(|a| a == "--epaper");

    let scene = make_scene(scale);
    let scale = scene.scale();

    let mut theme = BW_THEME;
    copy_theme_colors(&mut theme, &LIGHT_THEME);

        let display: SimulatorDisplay<Rgb565> =
            SimulatorDisplay::new(Size::new(320 * scale, 240 * scale));
        let output_settings = OutputSettingsBuilder::new().scale(scale).build();
        run_loop(display, &output_settings, scene, theme);
    Ok(())
}

fn keydown_to_char(keycode: Keycode, keymod: Mod) -> TextAction {
    println!("keycode as number {}", keycode.into_i32());
    let ch = keycode.into_i32();
    if ch <= 0 {
        return TextAction::Unknown;
    }
    let shifted = keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD);
    let controlled = keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::RCTRLMOD);

    if let Some(ch) = char::from_u32(ch as u32) {
        if ch == 'd' && controlled {
            return TextAction::ForwardDelete;
        }
        if ch.is_alphabetic() {
            return if shifted {
                TextAction::TypedAscii(ch.to_ascii_uppercase() as u8)
            } else {
                TextAction::TypedAscii(ch.to_ascii_lowercase() as u8)
            };
        }
        if ch.is_ascii_graphic() {
            return TextAction::TypedAscii(ch as u8);
        }
    }
    match keycode {
        Keycode::Backspace => TextAction::BackDelete,
        Keycode::LEFT => TextAction::Left,
        Keycode::RIGHT => TextAction::Right,
        Keycode::UP => TextAction::Up,
        Keycode::DOWN => TextAction::Down,
        Keycode::SPACE => TextAction::TypedAscii(b' '),
        _ => {
            println!("not supported: {keycode}");
            return TextAction::Unknown;
        }
    }
}

fn handle_events(result: InputResult, scene: &mut Scene<Rgb565>, theme: &mut Theme<Rgb565>) {
    println!("result of event {:?} from {}", result.input, result.source);
}

const LIGHT_THEME: Theme<Rgb565> = Theme {
    font: FontKind::Bitmap(FONT_7X13),
    bold_font: FontKind::Bitmap(FONT_7X13_BOLD),
    standard: ViewStyle {
        fill: Rgb565::WHITE,
        text: Rgb565::BLACK,
    },
    panel: ViewStyle {
        fill: Rgb565::CSS_LIGHT_GRAY,
        text: Rgb565::BLACK,
    },
    selected: ViewStyle {
        fill: hex_str_to_rgb565("#444444"),
        text: Rgb565::WHITE,
    },
    accented: ViewStyle {
        fill: hex_str_to_rgb565("#6688dd"),
        text: Rgb565::WHITE,
    },
};
const DARK_THEME: Theme<Rgb565> = Theme {
    font: FontKind::Bitmap(FONT_7X13),
    bold_font: FontKind::Bitmap(FONT_7X13_BOLD),
    standard: ViewStyle {
        fill: hex_str_to_rgb565("#222222"),
        text: hex_str_to_rgb565("#999999"),
    },
    panel: ViewStyle {
        fill: Rgb565::BLACK,
        text: Rgb565::WHITE,
    },
    selected: ViewStyle {
        fill: hex_str_to_rgb565("#000088"),
        text: hex_str_to_rgb565("#3366ff"),
    },
    accented: ViewStyle {
        fill: Rgb565::RED,
        text: Rgb565::WHITE,
    },
};

fn copy_theme_colors(theme: &mut Theme<Rgb565>, new: &Theme<Rgb565>) {
    theme.standard = new.standard.clone();
    theme.panel = new.panel.clone();
    theme.selected = new.selected.clone();
    theme.accented = new.accented.clone();
}
