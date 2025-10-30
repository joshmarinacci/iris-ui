use crate::gfx::draw_centered_text;
use crate::input::{InputEvent, OutputAction};
use crate::view::Flex::Shrink;
use crate::view::{View, ViewId};
use crate::{util, DrawEvent, GuiEvent, LayoutEvent};
use alloc::boxed::Box;
use alloc::string::{String, ToString};

pub struct ButtonState {
    pub command: String,
    pub primary: bool,
}

fn button_layout(e: &mut LayoutEvent) {
    if let Some(view) = e.scene.get_view_mut(&e.target) {
        view.bounds.size = util::calc_size(e.theme.bold_font, &view.title);
    }
}

fn button_draw(e: &mut DrawEvent) {
    let Some(state) = e.view.get_state::<ButtonState>() else {
        return;
    };
    if state.primary {
        e.ctx.fill_rect(&e.view.bounds, &e.theme.accented.fill);
        e.ctx.stroke_rect(&e.view.bounds, &e.theme.standard.text);
        if e.focused == &Some(e.view.name.clone()) {
            e.ctx
                .stroke_rect(&e.view.bounds.contract(2), &e.theme.accented.text);
        }
        draw_centered_text(
            e.ctx,
            &e.view.title,
            &e.view.bounds,
            &e.theme.bold_font,
            &e.theme.accented.text,
        );
    } else {
        e.ctx.fill_rect(&e.view.bounds, &e.theme.standard.fill);
        e.ctx.stroke_rect(&e.view.bounds, &e.theme.standard.text);
        if e.focused == &Some(e.view.name.clone()) {
            e.ctx
                .stroke_rect(&e.view.bounds.contract(2), &e.theme.standard.text);
        }
        draw_centered_text(
            e.ctx,
            &e.view.title,
            &e.view.bounds,
            &e.theme.bold_font,
            &e.theme.standard.text,
        );
    }
}

fn button_input(e: &mut GuiEvent) -> Option<OutputAction> {
    if let InputEvent::Tap(_pt) = &e.event_type {
        e.scene.set_focused(e.target);
        if let Some(state) = e.scene.get_view_state::<ButtonState>(e.target) {
            return Some(OutputAction::Command(state.command.clone()));
        }
    }
    None
}

pub fn make_button(name: &ViewId, title: &str) -> View {
    make_full_button(name, title, title.into(), false)
}
pub fn make_full_button(name: &ViewId, title: &str, command: &str, primary: bool) -> View {
    let mut view = View {
        ..Default::default()
    };
    as_button(&mut view);
    view.name = name.clone();
    view.title = title.to_string();
    view.state = Some(Box::new(ButtonState {
        primary,
        command: command.into(),
    }));
    view
}

pub fn as_button(view: &mut View) {
    view.h_flex = Shrink;
    view.v_flex = Shrink;
    view.state = Some(Box::new(ButtonState {
        primary: false,
        command: "".into(),
    }));
    view.input = Some(button_input);
    view.layout = Some(button_layout);
    view.draw = Some(button_draw);
}
