use iris_ui::button::ButtonState;
use iris_ui::geom::Insets;
use iris_ui::label::as_header_label;
use iris_ui::layouts::{layout_hbox, layout_vbox, make_vertical_spacer};
use iris_ui::panel::{draw_std_panel, make_panel, PanelState};
use iris_ui::scene::Scene;
use iris_ui::view::Align::Center;
use iris_ui::view::Flex::Shrink;
use iris_ui::view::{View, ViewId};
use iris_ui::view_builder::ViewBuilder;

pub enum OptionDialogChoices {
    DeleteOrCancel,
    YesOrNo,
}

pub enum OptionResponses {
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

pub fn make_option_dialog(
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
            crate::build_button(scene, ButtonState {
                command: OptionResponses::Delete.into(),
                primary: false,
            }).with_title("Delete").add_to_parent(&panel.name);
            crate::build_button(scene, ButtonState {
                command: OptionResponses::Cancel.into(),
                primary: true,
            }).with_title("Cancel").add_to_parent(&panel.name);
        }
        OptionDialogChoices::YesOrNo => {
            crate::build_button(scene, ButtonState {
                command: OptionResponses::Yes.into(),
                primary: false,
            }).with_title("Yes").add_to_parent(&panel.name);
            crate::build_button(scene, ButtonState {
                command: OptionResponses::No.into(),
                primary: true,
            }).with_title("No").add_to_parent(&panel.name);
        }
    }
    scene.add_view_to_parent(panel, &dialog.name);

    dialog
}