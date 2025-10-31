use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics_simulator::sdl2::{Keycode, Mod};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use env_logger::Target;
use iris_ui::device::EmbeddedDrawingContext;
use iris_ui::geom::Point as GPoint;
use iris_ui::input::{InputEvent, InputResult, OutputAction, TextAction};
use iris_ui::list_view::ListState;
use iris_ui::scene::{click_at, draw_scene, event_at_focused, layout_scene, Scene};
use iris_ui::text_input::TextInputState;
use iris_ui::view::ViewId;
use iris_ui::{Theme, BW_THEME};
use log::{info, LevelFilter};
use password_manager::option_dialog::OptionDialogChoices;
use password_manager::{EditMode, PasswordEntry, DELETE_DIALOG, DETAILS_PANEL, ENTRIES_LIST};
use std::cell::RefCell;
use std::rc::Rc;

fn main() -> Result<(), std::convert::Infallible> {
    env_logger::Builder::new()
        .target(Target::Stdout) // <-- redirects to stdout
        .filter(None, LevelFilter::Info)
        .init();

    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(320, 240));

    let mut database: Rc<RefCell<Vec<PasswordEntry>>> = Rc::new(RefCell::new(vec![]));
    database.borrow_mut().push(PasswordEntry {
        name: "email".into(),
        description: "personal gmail account".into(),
        username: "me@mydomain.com".into(),
        password: "some_password".into(),
    });
    database.borrow_mut().push(PasswordEntry {
        name: "Yahoo".into(),
        description: "yahoo account".into(),
        username: "me@mydomain.com".into(),
        password: "some_password".into(),
    });
    database.borrow_mut().push(PasswordEntry {
        name: "Home Wifi".into(),
        description: "".into(),
        username: "SSID".into(),
        password: "PASSWORD".into(),
    });
    for i in 1..20 {
        database.borrow_mut().push(PasswordEntry {
            name: format!("item {i}"),
            description: "".into(),
            username: "".into(),
            password: "".into(),
        });
    }

    let mut scene = password_manager::make_scene(&mut database);
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
                        handle_events(result, &mut scene, &mut theme, &mut database);
                    }
                }
                SimulatorEvent::MouseButtonDown { mouse_btn, point } => {
                    println!("mouse down");
                }
                SimulatorEvent::MouseWheel {
                    scroll_delta,
                    direction,
                } => {
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
            TextAction::Unknown
        }
    }
}

fn handle_events(
    result: InputResult,
    scene: &mut Scene,
    theme: &mut Theme,
    database: &mut Rc<RefCell<Vec<PasswordEntry>>>,
) {
    println!("result of event {:?} from {}", result.input, result.source);
    match &result.action {
        Some(OutputAction::Command(cmd)) => {
            info!("got a command {cmd} from {}", result.source);
            match cmd.as_str() {
                "add" => {
                    let entry = PasswordEntry {
                        name: "".into(),
                        description: "".into(),
                        username: "".into(),
                        password: "".into(),
                    };
                    let panel = password_manager::make_entry_edit_panel(&entry, scene, EditMode::Add);
                    scene.add_view_to_root(panel);
                }
                "close-panel" => {
                    scene.remove_parent_and_children(&DETAILS_PANEL);
                    scene.set_focused(&ENTRIES_LIST);
                }
                "add-entry-save" => {
                    let mut entry = PasswordEntry {
                        name: "".into(),
                        description: "".into(),
                        username: "".into(),
                        password: "".into(),
                    };
                    if let Some(state) =
                        scene.get_view_state::<TextInputState>(&ViewId::new("name_value"))
                    {
                        entry.name = state.text.clone();
                    }
                    if let Some(state) =
                        scene.get_view_state::<TextInputState>(&ViewId::new("desc_value"))
                    {
                        entry.description = state.text.clone();
                    }
                    if let Some(state) =
                        scene.get_view_state::<TextInputState>(&ViewId::new("user_value"))
                    {
                        entry.username = state.text.clone();
                    }
                    if let Some(state) =
                        scene.get_view_state::<TextInputState>(&ViewId::new("pass_value"))
                    {
                        entry.password = state.text.clone();
                    }
                    scene.remove_parent_and_children(&DETAILS_PANEL);
                    database.borrow_mut().push(entry);
                }
                "edit-entry" => {
                    scene.remove_parent_and_children(&DETAILS_PANEL);
                    if let Some(state) =
                        scene.get_view_state::<ListState<PasswordEntry>>(&ENTRIES_LIST)
                    {
                        let item = &database.borrow()[state.selected];
                        let panel = password_manager::make_entry_edit_panel(item, scene, EditMode::Edit);
                        scene.add_view_to_root(panel);
                    }
                }
                "edit-entry-save" => {
                    if let Some(state) =
                        scene.get_view_state::<ListState<PasswordEntry>>(&ENTRIES_LIST)
                    {
                        let entry = &mut database.borrow_mut()[state.selected];
                        if let Some(state) =
                            scene.get_view_state::<TextInputState>(&ViewId::new("name_value"))
                        {
                            entry.name = state.text.clone();
                        }
                        if let Some(state) =
                            scene.get_view_state::<TextInputState>(&ViewId::new("desc_value"))
                        {
                            entry.description = state.text.clone();
                        }
                        if let Some(state) =
                            scene.get_view_state::<TextInputState>(&ViewId::new("user_value"))
                        {
                            entry.username = state.text.clone();
                        }
                        if let Some(state) =
                            scene.get_view_state::<TextInputState>(&ViewId::new("pass_value"))
                        {
                            entry.password = state.text.clone();
                        }
                    }
                    scene.remove_parent_and_children(&DETAILS_PANEL);
                }
                "delete-entry" => {
                    if let Some(state) =
                        scene.get_view_state::<ListState<PasswordEntry>>(&ENTRIES_LIST)
                    {
                        let entry = &mut database.borrow_mut()[state.selected];
                        let dialog = password_manager::option_dialog::make_option_dialog(
                            &DELETE_DIALOG,
                            format!("Really delete '{}'?", entry.name),
                            OptionDialogChoices::DeleteOrCancel,
                            scene,
                        );
                        scene.add_view_to_root(dialog);
                    }
                }
                "delete" => {
                    if let Some(state) =
                        scene.get_view_state::<ListState<PasswordEntry>>(&ENTRIES_LIST)
                    {
                        let entry = database.borrow_mut().remove(state.selected);
                        scene.remove_parent_and_children(&DETAILS_PANEL);
                        scene.remove_parent_and_children(&DELETE_DIALOG);
                    }
                }
                "cancel" => {
                    scene.remove_parent_and_children(&DELETE_DIALOG);
                }
                "scroll-up" => {
                    if let Some(state) = scene.get_view_state::<ListState<PasswordEntry>>(&ENTRIES_LIST) {
                        state.scroll_up();
                    }
                    scene.mark_dirty_view(&ENTRIES_LIST);
                }
                "scroll-down" => {
                    if let Some(state) = scene.get_view_state::<ListState<PasswordEntry>>(&ENTRIES_LIST) {
                        state.scroll_down();
                    }
                    scene.mark_dirty_view(&ENTRIES_LIST);
                }
                _ => {
                    info!("unhandled command {cmd}")
                }
            }
        }
        Some(OutputAction::Selected(name, index)) => {
            info!("selected {name} at {index}");
            let panel = password_manager::make_entry_edit_panel(&database.borrow()[*index], scene, EditMode::View);
            scene.add_view_to_root(panel);
        }
        _ => {}
    }
}
