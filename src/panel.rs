use crate::DrawEvent;
use crate::geom::Insets;
use crate::view::{View, ViewId};
use alloc::boxed::Box;

pub struct PanelState {
    pub gap: i32,
    pub border_visible: bool,
    pub padding: Insets,
}

impl PanelState {
    pub fn new() -> PanelState {
        PanelState {
            gap: 0,
            border_visible: true,
            padding: Insets::new_same(0),
        }
    }
}

pub fn draw_std_panel(e: &mut DrawEvent) {
    let bounds = e.view.bounds;
    e.ctx.fill_rect(&bounds, &e.theme.panel.fill);
    if let Some(state) = e.view.get_state::<PanelState>() {
        if state.border_visible {
            e.ctx.stroke_rect(&bounds, &e.theme.panel.text);
        }
    }
}

pub fn make_panel(name: &ViewId) -> View {
    View {
        name: name.clone(),
        title: "title".into(),
        state: Some(Box::new(PanelState {
            gap: 0,
            border_visible: true,
            padding: Insets::new_same(0),
        })),
        draw: Some(draw_std_panel),
        ..Default::default()
    }
}
