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
use iris_ui::scene::{click_at, draw_scene, event_at_focused, layout_scene, Scene};
use iris_ui::toggle_button::make_toggle_button;
use iris_ui::toggle_group::{layout_toggle_group, make_toggle_group, SelectOneOfState};
use iris_ui::{util, Theme, ViewStyle, BW_THEME};
use std::convert::Into;

use embedded_graphics::prelude::*;
use embedded_graphics_simulator::sdl2::{Keycode, Mod};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use env_logger::Target;
use iris_ui::device::EmbeddedDrawingContext;
use iris_ui::grid::{make_grid_panel, GridLayoutState, LayoutConstraint};
use iris_ui::input::{InputEvent, InputResult, OutputAction, TextAction};
use iris_ui::label::{make_header_label, make_label};
use iris_ui::layouts::{layout_hbox, layout_std_panel, layout_vbox};
use iris_ui::list_view::make_list_view;
use iris_ui::panel::{draw_std_panel, make_panel, PanelState};
use iris_ui::tabbed_panel::{make_tabbed_panel, LayoutPanelState};
use iris_ui::text_input::make_text_input;
use iris_ui::util::hex_str_to_rgb565;
use iris_ui::view::Align::{Center, Start};
use iris_ui::view::Flex::{Fixed, Grow, Shrink};
use iris_ui::view::{Align, View, ViewId};
use log::{info, LevelFilter};

const POPUP_MENU: &'static ViewId = &ViewId::new("popup-menu");
fn make_scene() -> Scene {
    let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));
    const TABBED_PANEL: &'static ViewId = &ViewId::new("tabbed-panel");

    let tab_names = vec!["buttons", "layouts", "lists", "inputs", "themes"];
    let mut tabbed_panel: View = make_tabbed_panel(&TABBED_PANEL, tab_names, 0, &mut scene);
    const BUTTONS_PANEL: &'static ViewId = &ViewId::new("buttons");
    const LAYOUT_PANEL: &'static ViewId = &ViewId::new("layout-panel");
    const LISTS_PANEL: &'static ViewId = &ViewId::new("lists-panel");
    const INPUTS_PANEL: &'static ViewId = &ViewId::new("input-panel");
    const THEMES_PANEL: &'static ViewId = &ViewId::new("themes-panel");

    if let Some(state) = tabbed_panel.get_state::<LayoutPanelState>() {
        state.register_panel("buttons", BUTTONS_PANEL);
        state.register_panel("layouts", LAYOUT_PANEL);
        state.register_panel("lists", LISTS_PANEL);
        state.register_panel("inputs", INPUTS_PANEL);
        state.register_panel("themes", THEMES_PANEL);
    }

    {
        let mut grid = make_grid_panel(BUTTONS_PANEL).with_flex(Grow, Grow);
        let mut grid_layout = GridLayoutState::new_row_column(4, 30, 3, 100);
        grid_layout.debug = false;
        grid_layout.border_visible = false;

        let header1 = make_header_label("header1", "Header");
        grid_layout.place_at_row_column(&header1.name, 0, 0);
        scene.add_view_to_parent(header1, &grid.name);

        let label1 = make_label("label1", "A Label");
        grid_layout.place_at_row_column(&label1.name, 0, 1);
        scene.add_view_to_parent(label1, &grid.name);

        let button1 = make_button(&ViewId::new("button1"), "Button");
        grid_layout.place_at_row_column(&button1.name, 1, 0);
        scene.add_view_to_parent(button1, &grid.name);

        let button3 = make_full_button(&ViewId::new("button3"), "Primary", "primary".into(), true);
        grid_layout.place_at_row_column(&button3.name, 2, 0);
        scene.add_view_to_parent(button3, &grid.name);

        let toggle1 = make_toggle_button(&ViewId::new("toggle1"), "Toggle");
        grid_layout.place_at_row_column(&toggle1.name, 1, 1);
        scene.add_view_to_parent(toggle1, &grid.name);

        let mut button3 =
            make_toggle_group(&ViewId::new("toggle2"), vec!["Apple", "Ball", "Car"], 1)
                .with_h_flex(Shrink)
                .with_h_align(Center);
        grid_layout.place_at_row_column_with_spans(&button3.name, 3, 0, 2, 1);
        scene.add_view_to_parent(button3, &grid.name);

        grid.state = Some(Box::new(grid_layout));
        scene.add_view_to_parent(grid, &tabbed_panel.name);
    }
    {
        let wrapper = make_panel(LAYOUT_PANEL)
            .with_layout(Some(layout_hbox))
            .with_h_flex(Grow)
            .with_visible(false);

        {
            let col1 = make_column("col1").with_v_flex(Grow).with_h_flex(Grow);
            scene.add_view_to_parent(make_label("vbox-label", "vbox layout"), &col1.name);
            let vbox = make_panel(&ViewId::new("vbox_1")).with_layout(Some(layout_vbox));
            scene.add_view_to_parent(make_button(&ViewId::new("vbox-button1"), "A"), &vbox.name);
            scene.add_view_to_parent(make_button(&ViewId::new("vbox-button2"), "B"), &vbox.name);
            scene.add_view_to_parent(make_button(&ViewId::new("vbox-button3"), "C"), &vbox.name);
            scene.add_view_to_parent(vbox, &col1.name);
            scene.add_view_to_parent(col1, &wrapper.name);
        }

        {
            let col2 = make_column("col2").with_v_align(Start).with_h_flex(Grow);
            scene.add_view_to_parent(make_label("hbox-label", "hbox layout"), &col2.name);
            let hbox = make_panel(&ViewId::new("hbox_1")).with_layout(Some(layout_hbox));
            scene.add_view_to_parent(make_button(&ViewId::new("hbox-button1"), "A"), &hbox.name);
            scene.add_view_to_parent(make_button(&ViewId::new("hbox-button2"), "B"), &hbox.name);
            scene.add_view_to_parent(make_button(&ViewId::new("hbox-button3"), "C"), &hbox.name);
            scene.add_view_to_parent(hbox, &col2.name);
            scene.add_view_to_parent(col2, &wrapper.name);
        }

        scene.add_view_to_parent(wrapper, &tabbed_panel.name);
    }
    {
        let mut wrapper = make_panel(&LISTS_PANEL)
            .with_visible(false)
            .with_flex(Grow, Grow)
            .with_state(Some(Box::new(PanelState {
                gap: 0,
                border_visible: false,
                padding: Insets::new_same(0),
            })))
            .with_layout(Some(layout_hbox));

        let col1 = make_column("lists-col1");
        scene.add_view_to_parent(make_label("lists-label", "Lists"), &col1.name);
        let button = make_full_button(&scene.next_view_id(), "Open Popup", "open-popup", false);
        scene.add_view_to_parent(button, &col1.name);
        scene.add_view_to_parent(col1, &wrapper.name);
        let list = make_list_view(
            &ViewId::new("list-view"),
            vec!["First", "Second", "Third", "Fourth", "Fifth"],
            1,
        );
        scene.add_view_to_parent(list, &wrapper.name);
        scene.add_view_to_parent(wrapper, &tabbed_panel.name);
    }
    {
        let panel = make_panel(INPUTS_PANEL)
            .with_layout(Some(layout_std_panel))
            .with_state(Some(Box::new(PanelState {
                gap: 0,
                border_visible: false,
                padding: Insets::new_same(0),
            })))
            .with_flex(Grow, Grow)
            .with_visible(false);
        scene.add_view_to_parent(
            make_text_input("text input", "input").position_at(10, 10),
            &panel.name,
        );
        scene.add_view_to_parent(panel, &tabbed_panel.name);
    }
    {
        let panel = make_column(THEMES_PANEL.as_str())
            .with_visible(false)
            .with_state(Some(Box::new(PanelState {
                border_visible: false,
                gap: 0,
                padding: Insets::new_same(10),
            })))
            .with_flex(Grow, Grow);
        let themes_list_id = ViewId::new("themes-list");
        let themes = make_list_view(
            &themes_list_id,
            vec!["Light", "Dark", "Ice Cream", "Minty Fresh", "Amber"],
            0,
        );
        scene.add_view_to_parent(themes, &panel.name);
        scene.add_view_to_parent(panel, &tabbed_panel.name);
    }
    scene.add_view_to_root(tabbed_panel);

    {
        let font_buttons_name = scene.next_view_id();
        let font_buttons = make_panel(&font_buttons_name)
            .with_bounds(Bounds::new(30, 200, 200, 30))
            .with_layout(Some(layout_hbox))
            .with_state(Some(Box::new(PanelState {
                border_visible: true,
                gap: 5,
                padding: Insets::new_same(5),
            })));
        let small_button = make_full_button(&scene.next_view_id(), "Small", "font-small", false);
        scene.add_view_to_parent(small_button, &font_buttons_name);
        let med_button = make_full_button(&scene.next_view_id(), "Medium", "font-medium", false);
        scene.add_view_to_parent(med_button, &font_buttons_name);
        let large_button = make_full_button(&scene.next_view_id(), "Large", "font-large", false);
        scene.add_view_to_parent(large_button, &font_buttons_name);
        scene.add_view_to_root(font_buttons);
    }

    if let Some(state) = scene.get_view_state::<SelectOneOfState>(TABBED_PANEL) {
        state.selected = 2;
    }

    scene
}
fn make_column(name: &'static str) -> View {
    make_panel(&ViewId::new(name))
        .with_state(Some(Box::new(PanelState::new())))
        .with_layout(Some(layout_vbox))
}

fn main() -> Result<(), std::convert::Infallible> {
    env_logger::Builder::new()
        .target(Target::Stdout) // <-- redirects to stdout
        .filter(None, LevelFilter::Info)
        .init();

    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(320, 240));

    let mut scene = make_scene();
    let mut theme = BW_THEME;
    copy_theme_colors(&mut theme, &LIGHT_THEME);

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Simulator Test", &output_settings);
    'running: loop {
        let mut ctx = EmbeddedDrawingContext::new(&mut display);
        ctx.clip = scene.dirty_rect.clone();
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
                    if let Some(result) =
                        click_at(&mut scene, &vec![], GPoint::new(point.x, point.y))
                    {
                        handle_events(result, &mut scene, &mut theme);
                    }
                }
                SimulatorEvent::MouseButtonDown { mouse_btn, point } => {
                    println!("mouse down");
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

fn handle_events(result: InputResult, scene: &mut Scene, theme: &mut Theme) {
    println!("result of event {:?} from {}", result.input, result.source);
    match &result.action {
        Some(OutputAction::Command(cmd)) => {
            info!("got a command {cmd}");
            match cmd.as_str() {
                "font-small" => {
                    theme.font = FONT_5X7;
                    theme.bold_font = FONT_5X7;
                    scene.mark_layout_dirty();
                }
                "font-medium" => {
                    theme.font = FONT_6X10;
                    theme.bold_font = FONT_6X10;
                    scene.mark_layout_dirty();
                }
                "font-large" => {
                    theme.font = FONT_7X13;
                    theme.bold_font = FONT_7X13_BOLD;
                    scene.mark_layout_dirty();
                }
                "open-popup" => {
                    let menu = make_list_view(POPUP_MENU, vec!["Item 1", "Item 2", "Item 3"], 0)
                        .position_at(50, 50);
                    scene.set_focused(&menu.name);
                    scene.add_view_to_root(menu);
                }
                _ => {}
            }
        }
        _ => {}
    }
    if result.source.as_str() == "themes-list" {
        match &result.action {
            Some(OutputAction::Command(cmd)) => {
                match cmd.as_str() {
                    "Dark" => copy_theme_colors(theme, &DARK_THEME),
                    "Light" => copy_theme_colors(theme, &LIGHT_THEME),
                    "Ice Cream" => copy_theme_colors(theme, &ICE_CREAM_THEME),
                    "Minty Fresh" => copy_theme_colors(theme, &MINTY_FRESH),
                    "Amber" => copy_theme_colors(theme, &AMBER),
                    _ => {}
                }
                scene.mark_dirty_all();
            }
            _ => {}
        }
    }
    if result.source == *POPUP_MENU {
        scene.remove_view(POPUP_MENU);
    }
}

const LIGHT_THEME: Theme = Theme {
    font: FONT_7X13,
    bold_font: FONT_7X13_BOLD,
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
const DARK_THEME: Theme = Theme {
    font: FONT_7X13,
    bold_font: FONT_7X13_BOLD,
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

//https://lospec.com/palette-list/ice-cream-gb
const ICE_CREAM_THEME: Theme = Theme {
    font: FONT_7X13,
    bold_font: FONT_7X13_BOLD,
    standard: ViewStyle {
        fill: hex_str_to_rgb565("fff6d3"),
        text: hex_str_to_rgb565("#7c3f58"),
    },
    panel: ViewStyle {
        fill: hex_str_to_rgb565("fff6d3"),
        text: hex_str_to_rgb565("#7c3f58"),
    },
    selected: ViewStyle {
        fill: hex_str_to_rgb565("#f9a875"),
        text: hex_str_to_rgb565("#7c3f58"),
    },
    accented: ViewStyle {
        fill: hex_str_to_rgb565("#eb6b6f"),
        text: hex_str_to_rgb565("#fff6d3"),
    },
};
//https://lospec.com/palette-list/minty-fresh
const MINTY_FRESH: Theme = Theme {
    font: FONT_7X13,
    bold_font: FONT_7X13_BOLD,
    standard: ViewStyle {
        fill: hex_str_to_rgb565("#856d52"),
        text: hex_str_to_rgb565("#fbffe0"),
    },
    panel: ViewStyle {
        fill: hex_str_to_rgb565("#40332f"),
        text: hex_str_to_rgb565("#fbffe0"),
    },
    selected: ViewStyle {
        fill: hex_str_to_rgb565("#fbffe0"),
        text: hex_str_to_rgb565("#40332f"),
    },
    accented: ViewStyle {
        fill: hex_str_to_rgb565("#95c798"),
        text: hex_str_to_rgb565("#fbffe0"),
    },
};
//https://lospec.com/palette-list/amber-crtgb
const AMBER: Theme = Theme {
    font: FONT_7X13,
    bold_font: FONT_7X13_BOLD,
    standard: ViewStyle {
        fill: hex_str_to_rgb565("#0d0405"),
        text: hex_str_to_rgb565("#d35600"),
    },
    panel: ViewStyle {
        fill: hex_str_to_rgb565("#0d0405"),
        text: hex_str_to_rgb565("#d35600"),
    },
    selected: ViewStyle {
        fill: hex_str_to_rgb565("#fed018"),
        text: hex_str_to_rgb565("#5e1210"),
    },
    accented: ViewStyle {
        fill: Rgb565::RED,
        text: Rgb565::WHITE,
    },
};

fn copy_theme_colors(theme: &mut Theme, new: &Theme) {
    theme.standard = new.standard.clone();
    theme.panel = new.panel.clone();
    theme.selected = new.selected.clone();
    theme.accented = new.accented.clone();
}
