use crate::gfx::TextStyle;
use crate::util;
use crate::view::Flex::Shrink;
use crate::view::{View, ViewId};

pub fn make_label(name: &'static str, title: &str) -> View {
    View {
        name: ViewId::new(name),
        title: title.into(),
        h_flex: Shrink,
        v_flex: Shrink,
        layout: Some(|e| {
            if let Some(view) = e.scene.get_view_mut(e.target) {
                view.bounds.size = util::calc_size(e.theme.font, &view.title);
            }
        }),
        draw: Some(|e| {
            let style = TextStyle::new(e.theme.font, &e.theme.standard.text);
            e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
        }),
        ..Default::default()
    }
}

pub fn make_header_label(name: &'static str, title: &str) -> View {
    View {
        name: ViewId::new(name),
        title: title.into(),
        h_flex: Shrink,
        v_flex: Shrink,
        layout: Some(|e| {
            if let Some(view) = e.scene.get_view_mut(e.target) {
                view.bounds.size = util::calc_size(e.theme.font, &view.title);
            }
        }),
        draw: Some(|e| {
            let style = TextStyle::new(e.theme.bold_font, &e.theme.accented.fill);
            e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
        }),
        ..Default::default()
    }
}
