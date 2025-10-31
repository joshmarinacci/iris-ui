pub mod option_dialog;

use crate::option_dialog::OptionDialogChoices;
use iris_ui::button::{as_button, make_full_button, ButtonState};
use iris_ui::geom::{Bounds, Insets};
use iris_ui::grid::{make_grid_panel, GridLayoutState};
use iris_ui::input::{InputResult, OutputAction};
use iris_ui::label::{as_header_label, make_label};
use iris_ui::list_view::{make_generic_list, ListState};
use iris_ui::scene::Scene;
use iris_ui::text_input::{make_text_input, TextInputState};
use iris_ui::view::Align::{Center, End, Start};
use iris_ui::view::Flex::Grow;
use iris_ui::view::{View, ViewId};
use iris_ui::view_builder::ViewBuilder;
use iris_ui::Theme;
use log::info;
use std::cell::RefCell;
use std::rc::Rc;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[derive(Debug, Clone)]
pub struct PasswordEntry {
    pub name: String,
    pub description: String,
    pub username: String,
    pub password: String,
}

pub const ENTRIES_LIST: ViewId = ViewId::new("entries_list");
pub const DETAILS_PANEL: ViewId = ViewId::new("details_panel");
pub const DELETE_DIALOG: ViewId = ViewId::new("delete_dialog");

fn render_password_entry(event: &PasswordEntry) -> String {
    format!("{}: {}", event.name, event.username)
}

pub fn make_scene(database: &mut Rc<RefCell<Vec<PasswordEntry>>>) -> Scene {
    let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

    // button for search screen
    build_button(&mut scene, ButtonState::new_command("search"))
        .with_title("Search").with_position(30, 20).add_to_root();
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
        .with_position(120, 200)
        .add_to_root();
    build_button(&mut scene, ButtonState::new_command("scroll-down"))
        .with_title("down")
        .with_position(170, 200)
        .add_to_root();

    scene.set_focused(&list.name);
    scene.add_view_to_root(list);
    scene
}

pub fn make_entry_edit_panel(entry: &PasswordEntry, scene: &mut Scene, mode: EditMode) -> View {
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditMode {
    View,
    Add,
    Edit,
    Delete,
}

fn build_button(scene: &mut Scene, state: ButtonState) -> ViewBuilder<ButtonState> {
    ViewBuilder::build_with(scene, as_button, state)
}

fn build_header_label(scene: &mut Scene) -> ViewBuilder<()> {
    ViewBuilder::build_with(scene, as_header_label, ())
}

pub fn handle_events(
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
                        let dialog = crate::option_dialog::make_option_dialog(
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