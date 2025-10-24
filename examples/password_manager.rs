#[cfg(feature = "std")]
use embedded_graphics::geometry::{Point as EPoint, Size};
use embedded_graphics::pixelcolor::Rgb565;
use iris_ui::geom::{Bounds, Point as GPoint};
use iris_ui::scene::{click_at, draw_scene, event_at_focused, layout_scene, Scene};
use iris_ui::{Theme, BW_THEME};

use embedded_graphics_simulator::sdl2::{Keycode, Mod};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use env_logger::Target;
use iris_ui::button::make_button;
use iris_ui::device::EmbeddedDrawingContext;
use iris_ui::grid::{make_grid_panel, GridLayoutState};
use iris_ui::input::{InputEvent, InputResult, OutputAction, TextAction};
use iris_ui::label::{make_header_label, make_label};
use iris_ui::list_view::make_list_view;
use iris_ui::panel::make_panel;
use iris_ui::text_input::make_text_input;
use iris_ui::view::Align::{End, Start};
use iris_ui::view::Flex::Grow;
use iris_ui::view::ViewId;
use log::{info, LevelFilter};

struct PasswordEntry {
    name: String,
    description: String,
    username: String,
    password: String,
}

const SEARCH_BUTTON: ViewId = ViewId::new("search_button");
const LIST_BUTTON: ViewId = ViewId::new("list_button");
const ENTRIES_LIST: ViewId = ViewId::new("entries_list");
const DETAILS_PANEL: ViewId = ViewId::new("details_panel");

fn make_scene() -> Scene {
    let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

    // button for search screen
    let search_button = make_button(&SEARCH_BUTTON, "Search")
        .position_at(30, 20);
    scene.add_view_to_root(search_button);
    // button for list of all entries
    let list_button = make_button(&LIST_BUTTON, "List")
        .position_at(30, 50);
    scene.add_view_to_root(list_button);
    // list of all entries

    let entries = vec![
        PasswordEntry {
            name: "email".into(),
            description: "personal gmail account".into(),
            username: "me@mydomain.com".into(),
            password: "some_password".into(),
        },
        PasswordEntry {
            name: "Yahoo".into(),
            description: "yahoo account".into(),
            username: "me@mydomain.com".into(),
            password: "some_password".into(),
        },
        PasswordEntry {
            name: "Home Wifi".into(),
            description: "".into(),
            username: "SSID".into(),
            password: "PASSWORD".into(),
        }
    ];
    let entries = vec!["a", "b", "c"];

    let list = make_list_view(&ENTRIES_LIST, entries, 0)
        .with_bounds(Bounds::new(100, 20, 200, 200));
    scene.add_view_to_root(list);

    let mut panel = make_grid_panel(&DETAILS_PANEL)
        .with_bounds(Bounds::new(10, 10, 300, 220));

    {
        let mut grid = GridLayoutState::new_row_column(4, 30, 3, 100);
        grid.debug = false;
        grid.border_visible = true;

        let name_label = make_header_label("name_title", "Name").with_h_align(End);
        grid.place_at_row_column(&name_label.name, 0, 0);
        scene.add_view_to_parent(name_label, &DETAILS_PANEL);

        let name_value = make_label("name_value", "some name").with_h_align(Start);
        grid.place_at_row_column_with_spans(&name_value.name, 0, 1, 1, 2);
        scene.add_view_to_parent(name_value, &DETAILS_PANEL);


        let desc_label = make_header_label("desc_title", "Description").with_h_align(End);
        grid.place_at_row_column(&desc_label.name, 1, 0);
        scene.add_view_to_parent(desc_label, &DETAILS_PANEL);

        let desc_value = make_label("desc_value", "some desc").with_h_align(Start);
        grid.place_at_row_column_with_spans(&desc_value.name, 1, 1, 1, 2);
        scene.add_view_to_parent(desc_value, &DETAILS_PANEL);

        let user_label = make_header_label("user_title", "Username").with_h_align(End);
        grid.place_at_row_column(&user_label.name, 2, 0);
        scene.add_view_to_parent(user_label, &DETAILS_PANEL);

        let user_value = make_label("user_value", "some desc").with_h_align(Start);
        grid.place_at_row_column_with_spans(&user_value.name, 2, 1, 1, 2);
        scene.add_view_to_parent(user_value, &DETAILS_PANEL);

        let pass_label = make_header_label("pass_title", "Password").with_h_align(End);
        grid.place_at_row_column(&pass_label.name, 3, 0);
        scene.add_view_to_parent(pass_label, &DETAILS_PANEL);

        let pass_value = make_text_input("pass_value", "some desc").with_h_align(Start).with_h_flex(Grow);
        grid.place_at_row_column_with_spans(&pass_value.name, 3, 1, 1, 2);
        scene.add_view_to_parent(pass_value, &DETAILS_PANEL);

        let close = make_button(&ViewId::new("close"), "Close");
        grid.place_at_row_column(&close.name, 4, 2);
        scene.add_view_to_parent(close, &DETAILS_PANEL);

        panel.state = Some(Box::new(grid));
    }

    scene.add_view_to_root(panel);


    // buttons to add new entry
    // button to edit the current entry
    // select item in list of entry shows details
    // complete editing or cancel editing

    //mock data for now.
    scene
}
fn main() -> Result<(), std::convert::Infallible> {
    env_logger::Builder::new()
        .target(Target::Stdout) // <-- redirects to stdout
        .filter(None, LevelFilter::Info)
        .init();

    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(320, 240));

    let mut scene = make_scene();
    let mut theme = BW_THEME;

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
        }
        _ => {}
    }
}
