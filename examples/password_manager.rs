#[cfg(feature = "std")]
use embedded_graphics::geometry::{Point as EPoint, Size};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics_simulator::sdl2::{Keycode, Mod};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use env_logger::Target;
use iris_ui::button::{ButtonState, as_button, make_full_button};
use iris_ui::device::EmbeddedDrawingContext;
use iris_ui::geom::{Bounds, Insets, Point as GPoint, Point};
use iris_ui::grid::{GridLayoutState, make_grid_panel};
use iris_ui::input::{InputEvent, InputResult, OutputAction, TextAction};
use iris_ui::label::{as_header_label, make_label};
use iris_ui::layouts::{layout_hbox, layout_vbox};
use iris_ui::list_view::{ListState, make_generic_list};
use iris_ui::panel::{PanelState, draw_std_panel, make_panel};
use iris_ui::scene::{Scene, click_at, draw_scene, event_at_focused, layout_scene};
use iris_ui::text_input::{TextInputState, make_text_input};
use iris_ui::view::Align::{Center, End, Start};
use iris_ui::view::Flex::{Fixed, Grow, Shrink};
use iris_ui::view::{Align, View, ViewId};
use iris_ui::{BW_THEME, Theme};
use log::{LevelFilter, info};
use std::cell::RefCell;
use std::cmp::PartialEq;
use std::rc::Rc;

#[derive(Debug, Clone)]
struct PasswordEntry {
    name: String,
    description: String,
    username: String,
    password: String,
}

const ENTRIES_LIST: ViewId = ViewId::new("entries_list");
const DETAILS_PANEL: ViewId = ViewId::new("details_panel");
const DELETE_DIALOG: ViewId = ViewId::new("delete_dialog");

fn render_password_entry(event: &PasswordEntry) -> String {
    format!("{}: {}", event.name, event.username)
}

fn make_scene(database: &mut Rc<RefCell<Vec<PasswordEntry>>>) -> Scene {
    let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

    // button for search screen
    build_button(&mut scene, ButtonState::new_command("search"))
        .with_title("Search").with_position(30,20).add_to_root();
    // button for list of all entries
    build_button(&mut scene, ButtonState::new_command("list"))
        .with_title("List").with_position(30, 50).add_to_root();
    // button to add a new entry
    build_button(&mut scene, ButtonState::new_command("add"))
        .with_position(30, 80).with_title("Add").add_to_root();

    let list = make_generic_list(&ENTRIES_LIST, database.clone(), 0, render_password_entry)
        .with_v_flex(Grow)
        .with_bounds(Bounds::new(100, 20, 200, 150));

    build_button(&mut scene, ButtonState::new_command("scroll-up"))
        .with_title("up")
        .with_position(120,200)
        .add_to_root();
    build_button(&mut scene, ButtonState::new_command("scroll-down"))
        .with_title("down")
        .with_position(170,200)
        .add_to_root();

    scene.set_focused(&list.name);
    scene.add_view_to_root(list);
    scene
}

fn make_entry_edit_panel(entry: &PasswordEntry, scene: &mut Scene, mode: EditMode) -> View {
    let mut panel = make_grid_panel(&DETAILS_PANEL).with_bounds(Bounds::new(10, 10, 300, 220));

    {
        let mut grid = GridLayoutState::new_row_column(5, 30, 3, 93);
        grid.padding = Insets::new_same(10);
        grid.debug = false;
        grid.gap = 5;
        grid.border_visible = true;

        fn make_label_or_input(mode: EditMode, id: &'static str, value: &String) -> View {
            if mode == EditMode::View {
                make_label(id, value)
            } else {
                make_text_input(id, value)
            }
            .with_h_align(Start)
            .with_h_flex(Grow)
        }

        {
            ViewBuilder::build_with(scene, as_header_label, ())
                .with_title("Name")
                .with_h_align(End)
                .with(|view| {
                    grid.place_at_row_column(&view.name, 0, 0);
                })
                .add_to_parent(&DETAILS_PANEL);

            let name_value = make_label_or_input(mode, "name_value", &entry.name);
            grid.place_at_row_column_with_spans(&name_value.name, 0, 1, 1, 2);
            scene.add_view_to_parent(name_value, &DETAILS_PANEL);
        }

        {
            let name = build_header_label(scene)
                .with_title("Description")
                .with_h_align(End)
                .add_to_parent(&DETAILS_PANEL);
            grid.place_at_row_column(&name, 1, 0);

            let desc_value = make_label_or_input(mode, "desc_value", &entry.description);
            grid.place_at_row_column_with_spans(&desc_value.name, 1, 1, 1, 2);
            scene.add_view_to_parent(desc_value, &DETAILS_PANEL);
        }

        {
            let name = build_header_label(scene)
                .with_title("Username")
                .with_h_align(End)
                .add_to_parent(&DETAILS_PANEL);
            grid.place_at_row_column(&name, 2, 0);

            let user_value = make_label_or_input(mode, "user_value", &entry.username);
            grid.place_at_row_column_with_spans(&user_value.name, 2, 1, 1, 2);
            scene.add_view_to_parent(user_value, &DETAILS_PANEL);
        }

        {
            ViewBuilder::build_with(scene, as_header_label, ())
                .with_title("Password")
                .with_h_align(End)
                .with(|view| {
                    grid.place_at_row_column(&view.name, 3, 0);
                })
                .add_to_parent(&DETAILS_PANEL);

            let pass_value = make_label_or_input(mode, "pass_value", &entry.password);
            grid.place_at_row_column_with_spans(&pass_value.name, 3, 1, 1, 2);
            scene.add_view_to_parent(pass_value, &DETAILS_PANEL);
        }

        match mode {
            EditMode::View => {
                let delete =
                    make_full_button(&ViewId::new("delete"), "Delete", "delete-entry", false)
                        .with_h_align(Center)
                        .with_v_align(Center);
                grid.place_at_row_column(&delete.name, 4, 0);
                scene.add_view_to_parent(delete, &DETAILS_PANEL);

                let edit = make_full_button(&ViewId::new("edit"), "Edit", "edit-entry", false)
                    .with_h_align(Center)
                    .with_v_align(Center);
                grid.place_at_row_column(&edit.name, 4, 1);
                scene.add_view_to_parent(edit, &DETAILS_PANEL);

                let close = make_full_button(&ViewId::new("close"), "Close", "close-panel", true)
                    .with_h_align(Center)
                    .with_v_align(Center);
                grid.place_at_row_column(&close.name, 4, 2);
                scene.add_view_to_parent(close, &DETAILS_PANEL);
            }
            EditMode::Add => {
                // cancel button
                let cancel =
                    make_full_button(&ViewId::new("cancel"), "Cancel", "close-panel", false)
                        .with_h_align(Center)
                        .with_v_align(Center);
                grid.place_at_row_column(&cancel.name, 4, 1);
                scene.add_view_to_parent(cancel, &DETAILS_PANEL);

                // save button
                let save = make_full_button(&ViewId::new("save"), "Save", "add-entry-save", true)
                    .with_h_align(Center)
                    .with_v_align(Center);
                grid.place_at_row_column(&save.name, 4, 2);
                scene.add_view_to_parent(save, &DETAILS_PANEL);
            }
            EditMode::Edit => {
                let cancel =
                    make_full_button(&ViewId::new("cancel"), "Cancel", "close-panel", false)
                        .with_h_align(Center)
                        .with_v_align(Center);
                grid.place_at_row_column(&cancel.name, 4, 1);
                scene.add_view_to_parent(cancel, &DETAILS_PANEL);

                // save button
                let save = make_full_button(&ViewId::new("save"), "Save", "edit-entry-save", true)
                    .with_h_align(Center)
                    .with_v_align(Center);
                grid.place_at_row_column(&save.name, 4, 2);
                scene.add_view_to_parent(save, &DETAILS_PANEL);
            }
            EditMode::Delete => {}
        }

        panel.state = Some(Box::new(grid));
    }

    return panel;
}

struct ViewBuilder<'a, S> {
    scene: &'a mut Scene,
    view: View,
    pub state: Box<S>,
}

impl<'a, S: 'static> ViewBuilder<'a, S> {
    // pub fn build<F: FnMut(&mut View)>(scene: &'_ mut Scene) -> ViewBuilder<'_> {
    //     let view = scene.make_view();
    //     ViewBuilder {
    //         scene,
    //         view,
    //     }
    // }
    pub fn build_with<F: FnMut(&mut View)>(scene: &'a mut Scene, cb: F, state:S) -> ViewBuilder<'a, S> {
        let view = scene.make_view();
        ViewBuilder { scene, view, state:Box::new(state) }.with(cb)
    }

    pub fn with<F: FnMut(&mut View)>(mut self, mut cb: F) -> ViewBuilder<'a, S> {
        cb(&mut self.view);
        self
    }
    pub fn with_h_align(mut self, align: Align) -> ViewBuilder<'a, S> {
        self.view.h_align = align;
        self
    }
    pub fn mut_state<F: FnMut(&mut S)>(mut self, mut cb: F) -> ViewBuilder<'a, S> {
        cb(&mut self.state);
        self
    }
    pub fn with_title(mut self, title: &'a str) -> ViewBuilder<'a, S> {
        self.view.title = title.into();
        self
    }
    pub fn with_position(mut self, x: i32, y: i32) -> ViewBuilder<'a, S> {
        self.view.bounds.position.x = x;
        self.view.bounds.position.y = y;
        self
    }

    pub fn add_to_parent(mut self, parent: &ViewId) -> ViewId {
        self.view.state = Some(self.state);
        let name = self.view.name.clone();
        self.scene.add_view_to_parent(self.view, parent);
        name
    }
    pub fn add_to_root(mut self) -> ViewId {
        self.view.state = Some(self.state);
        let name = self.view.name.clone();
        self.scene.add_view_to_root(self.view);
        name
    }
}

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

    let mut scene = make_scene(&mut database);
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum EditMode {
    View,
    Add,
    Edit,
    Delete,
}

enum OptionDialogChoices {
    DeleteOrCancel,
    YesOrNo,
}
enum OptionResponses {
    Delete,
    Cancel,
    Save,
    Yes,
    No,
}
impl Into<String> for OptionResponses {
    fn into(self) -> String {
        match self {
            OptionResponses::Delete => "delete".to_string(),
            OptionResponses::Cancel => "cancel".to_string(),
            OptionResponses::Save => "save".to_string(),
            OptionResponses::Yes => "yes".to_string(),
            OptionResponses::No => "no".to_string(),
        }
    }
}
impl OptionResponses {
    pub fn to_string(&self) -> &'static str {
        match self {
            OptionResponses::Delete => "delete",
            OptionResponses::Cancel => "cancel",
            OptionResponses::Save => "save",
            OptionResponses::Yes => "yes",
            OptionResponses::No => "no",
        }
    }
}

fn build_button(scene:&mut Scene, state:ButtonState) -> ViewBuilder<ButtonState> {
    ViewBuilder::build_with(scene, as_button, state)
}
fn build_header_label(scene:&mut Scene) -> ViewBuilder<()> {
    ViewBuilder::build_with(scene, as_header_label, ())
}

fn make_vertical_spacer(height: i32) -> View {
    let spacer = View {
        v_flex: Fixed,
        bounds: Bounds::new(0, 0, 0, 20),
        ..Default::default()
    };
    spacer
}
fn make_option_dialog(
    name: &ViewId,
    text: String,
    variant: OptionDialogChoices,
    scene: &mut Scene,
) -> View {
    let dialog = View {
        name: name.clone(),
        title: "Option Dialog".into(),
        state: Some(Box::new(PanelState {
            gap: 5,
            border_visible: true,
            padding: Insets::new_same(10),
        })),
        layout: Some(layout_vbox),
        draw: Some(draw_std_panel),
        h_align: Center,
        v_align: Center,
        h_flex: Shrink,
        v_flex: Shrink,
        ..Default::default()
    }
    .position_at(50, 20);

    ViewBuilder::build_with(scene, as_header_label, ())
        .with_title(&text)
        .add_to_parent(&dialog.name);

    scene.add_view_to_parent(make_vertical_spacer(20), &dialog.name);

    let panel = make_panel(&scene.next_view_id())
        .with_layout(Some(layout_hbox))
        .with_state(Some(Box::new(PanelState {
            gap: 10,
            border_visible: false,
            padding: Insets::new_same(0),
        })));

    match variant {
        OptionDialogChoices::DeleteOrCancel => {
            build_button(scene, ButtonState {
                command: OptionResponses::Delete.into(),
                primary: false,
            }).with_title("Delete").add_to_parent(&panel.name);
            build_button(scene, ButtonState {
                command: OptionResponses::Cancel.into(),
                primary: true,
            }).with_title("Cancel").add_to_parent(&panel.name);
        }
        OptionDialogChoices::YesOrNo => {
            build_button(scene, ButtonState {
                command: OptionResponses::Yes.into(),
                primary: false,
            }).with_title("Yes").add_to_parent(&panel.name);
            build_button(scene, ButtonState {
                command: OptionResponses::No.into(),
                primary: true,
            }).with_title("No").add_to_parent(&panel.name);
        }
    }
    scene.add_view_to_parent(panel, &dialog.name);

    dialog
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
                    let panel = make_entry_edit_panel(&entry, scene, EditMode::Add);
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
                        let panel = make_entry_edit_panel(item, scene, EditMode::Edit);
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
                        let dialog = make_option_dialog(
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
            let panel = make_entry_edit_panel(&database.borrow()[*index], scene, EditMode::View);
            scene.add_view_to_root(panel);
        }
        _ => {}
    }
}
