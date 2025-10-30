use crate::gfx::TextStyle;
use crate::view::Flex::Shrink;
use crate::view::{View, ViewId};
use crate::{util, DrawEvent, LayoutEvent};

fn layout_label(e:&mut LayoutEvent) {
    if let Some(view) = e.scene.get_view_mut(e.target) {
        view.bounds.size = util::calc_size(e.theme.font, &view.title);
    }
}

fn draw_label(e:&mut DrawEvent) {
    let style = TextStyle::new(&e.theme.font, &e.theme.accented.fill);
    e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
}

pub fn make_label(name: &'static str, title: &str) -> View {
    View {
        name: ViewId::new(name),
        title: title.into(),
        h_flex: Shrink,
        v_flex: Shrink,
        layout: Some(layout_label),
        draw: Some(draw_label),
        ..Default::default()
    }
}

fn layout_header_label(e:&mut LayoutEvent) {
    if let Some(view) = e.scene.get_view_mut(e.target) {
        view.bounds.size = util::calc_size(e.theme.bold_font, &view.title);
    }
}

fn draw_header_label(e:&mut DrawEvent) {
    let style = TextStyle::new(&e.theme.bold_font, &e.theme.accented.fill);
    e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
}

pub fn make_header_label(name: &'static str, title: &str) -> View {
    View {
        name: ViewId::new(name),
        title: title.into(),
        h_flex: Shrink,
        v_flex: Shrink,
        layout: Some(layout_header_label),
        draw: Some(draw_header_label),
        ..Default::default()
    }
}

pub fn header_label(view:View) -> View {
    view.with_h_flex(Shrink)
        .with_v_flex(Shrink)
        .with_layout(Some(layout_header_label))
        .with_draw(Some(draw_header_label))
}

pub fn as_header_label(view:&mut View) {
    view.h_flex = Shrink;
    view.v_flex = Shrink;
    view.layout = Some(layout_header_label);
    view.draw = Some(draw_header_label);
}