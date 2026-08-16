use crate::DrawEvent;
use crate::geom::Insets;
use crate::view::{View, ViewId};
use alloc::boxed::Box;
use embedded_graphics::pixelcolor::PixelColor;

pub struct PanelState {
    pub gap: i32,
    pub border_visible: bool,
    pub padding: Insets,
}

impl Default for PanelState {
    fn default() -> Self {
        PanelState {
            gap: 0,
            border_visible: true,
            padding: Insets::new_same(0),
        }
    }
}

impl PanelState {
    pub fn new() -> PanelState {
        Self::default()
    }
}

pub fn draw_std_panel<C: PixelColor>(e: &mut DrawEvent<C>) {
    let bounds = e.view.bounds;
    e.ctx.fill_rect(&bounds, &e.theme.panel.fill);
    if let Some(state) = e.view.get_state::<PanelState>() {
        if state.border_visible {
            e.ctx.stroke_rect(&bounds, &e.theme.panel.text);
        }
    }
}

pub fn make_panel<C: PixelColor>(name: &ViewId) -> View<C> {
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
