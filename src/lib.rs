#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate core;

use crate::geom::Size;
use crate::input::{InputEvent, OutputAction};
use crate::scene::Scene;
use crate::view::ViewId;
use alloc::string::String;
use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::mono_font::ascii::{FONT_7X13, FONT_7X13_BOLD};
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use geom::{Bounds, Point};
use gfx::DrawingContext;
use view::View;

pub mod button;
pub mod device;
pub mod geom;
pub mod gfx;
pub mod grid;
pub mod input;
pub mod label;
pub mod layouts;
pub mod list_view;
pub mod panel;
pub mod scene;
pub mod tabbed_panel;
pub mod test;
pub mod text_input;
pub mod toggle_button;
pub mod toggle_group;
pub mod util;
pub mod view;

pub struct DrawEvent<'a> {
    pub ctx: &'a mut dyn DrawingContext,
    pub theme: &'a Theme,
    pub focused: &'a Option<ViewId>,
    pub view: &'a mut View,
    pub bounds: &'a Bounds,
}

pub type DrawFn = fn(event: &mut DrawEvent);
pub type LayoutFn = fn(layout: &mut LayoutEvent);
pub type InputFn = fn(event: &mut GuiEvent) -> Option<OutputAction>;

#[derive(Debug)]
pub struct Theme {
    pub font: MonoFont<'static>,
    pub bold_font: MonoFont<'static>,
    pub standard: ViewStyle,
    pub accented: ViewStyle,
    pub selected: ViewStyle,
    pub panel: ViewStyle,
}

#[derive(Debug, Clone, Copy)]
pub struct ViewStyle {
    pub fill: Rgb565,
    pub text: Rgb565,
}

pub type Callback = fn(event: &mut GuiEvent);

#[derive(Debug)]
pub struct GuiEvent<'a> {
    pub scene: &'a mut Scene,
    pub target: &'a ViewId,
    pub event_type: InputEvent,
    pub action: Option<OutputAction>,
}

#[derive(Debug)]
pub struct LayoutEvent<'a> {
    pub scene: &'a mut Scene,
    pub target: &'a ViewId,
    pub space: Size,
    pub theme: &'a Theme,
}

impl<'a> LayoutEvent<'a> {
    pub(crate) fn layout_all_children(&mut self, name: &ViewId, space: Size) {
        let fixed_kids = self.scene.get_children_ids(name);
        for kid in &fixed_kids {
            self.layout_child(kid, space);
        }
    }
    pub(crate) fn layout_child(&mut self, kid: &ViewId, available_space: Size) {
        if let Some(view) = self.scene.get_view_mut(kid) {
            if let Some(layout) = view.layout {
                let mut pass = LayoutEvent {
                    target: kid,
                    space: available_space,
                    scene: self.scene,
                    theme: self.theme,
                };
                layout(&mut pass);
            }
        }
    }
}

pub const BW_THEME: Theme = Theme {
    standard: ViewStyle {
        fill: Rgb565::WHITE,
        text: Rgb565::BLACK,
    },
    panel: ViewStyle {
        fill: Rgb565::WHITE,
        text: Rgb565::BLACK,
    },
    selected: ViewStyle {
        fill: Rgb565::BLACK,
        text: Rgb565::WHITE,
    },
    accented: ViewStyle {
        fill: Rgb565::BLACK,
        text: Rgb565::WHITE,
    },
    font: FONT_7X13,
    bold_font: FONT_7X13_BOLD,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::button::make_button;
    use crate::gfx::TextStyle;
    use crate::input::TextAction;
    use crate::scene::{click_at, draw_scene, event_at_focused, pick_at};
    use crate::test::MockDrawingContext;
    use crate::view::Align;
    use alloc::boxed::Box;
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;
    use log::{LevelFilter, info};
    use std::sync::Once;
    use test_log::test;

    extern crate std;

    pub fn make_simple_view(name: &ViewId) -> View {
        View {
            name: name.clone(),
            title: name.to_string(),
            bounds: Bounds::new(0, 0, 10, 10),
            visible: true,
            draw: Some(|e| e.ctx.fill_rect(&e.view.bounds, &e.theme.standard.fill)),
            input: None,
            state: None,
            layout: None,
            ..Default::default()
        }
    }
    fn layout_vbox(evt: &mut LayoutEvent) {
        if let Some(parent) = evt.scene.get_view_mut(evt.target) {
            let mut y = 0;
            let bounds = parent.bounds;
            let kids = evt.scene.get_children_ids(evt.target);
            for kid in kids {
                if let Some(ch) = evt.scene.get_view_mut(&kid) {
                    ch.bounds.position.x = 0;
                    ch.bounds.position.y = y;
                    ch.bounds.size.w = bounds.w();
                    y += ch.bounds.h();
                }
            }
        }
    }
    fn make_vbox(name: &ViewId, bounds: Bounds) -> View {
        View {
            name: name.clone(),
            title: name.to_string(),
            bounds,
            visible: true,
            draw: Some(|e| {
                e.ctx.fill_rect(&e.view.bounds, &e.theme.panel.fill);
            }),
            input: None,
            state: None,
            layout: Some(layout_vbox),
            ..Default::default()
        }
    }
    struct TestButtonState {
        drawn: bool,
        got_input: bool,
    }
    fn make_test_button(name: &ViewId) -> View {
        View {
            name: name.clone(),
            title: name.to_string(),
            bounds: Bounds::new(0, 0, 20, 20),
            visible: true,
            draw: Some(|e| {
                if let Some(state) = &mut e.view.state {
                    if let Some(state) = state.downcast_mut::<TestButtonState>() {
                        state.drawn = true;
                    }
                }
            }),
            input: Some(|e| {
                if let Some(view) = e.scene.get_view_mut(e.target) {
                    if let Some(state) = &mut view.state {
                        if let Some(state) = state.downcast_mut::<TestButtonState>() {
                            state.got_input = true;
                        }
                    }
                }
                None
            }),
            state: Some(Box::new(TestButtonState {
                drawn: false,
                got_input: false,
            })),
            layout: None,
            ..Default::default()
        }
    }
    fn make_text_box(name: &ViewId, title: &str) -> View {
        View {
            name: name.clone(),
            title: title.into(),
            bounds: Bounds::new(0, 0, 100, 30),
            visible: true,
            state: None,
            draw: None,
            layout: None,
            input: Some(|e| {
                match e.event_type {
                    InputEvent::Text(act) => match act {
                        TextAction::TypedAscii(key) => {
                            info!("got a keyboard event {}", key);
                            if let Some(view) = e.scene.get_view_mut(e.target) {
                                view.title.push(key as char)
                            }
                        }
                        _ => {}
                    },
                    _ => info!("ignoring other event"),
                };
                None
            }),
            ..Default::default()
        }
    }
    fn draw_label_view(e: &mut DrawEvent) {
        e.ctx.fill_text(
            &e.view.bounds,
            &e.view.title,
            &TextStyle::new(&e.theme.font, &e.theme.standard.text),
        );
    }
    fn make_label(name: &ViewId) -> View {
        View {
            name: name.clone(),
            title: name.to_string(),
            bounds: Bounds::new(0, 0, 30, 20),
            visible: true,
            draw: Some(draw_label_view),
            input: None,
            state: None,
            layout: None,
            ..Default::default()
        }
    }
    fn was_button_clicked(scene: &mut Scene, name: &ViewId) -> bool {
        scene
            .get_view(name)
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .downcast_ref::<TestButtonState>()
            .unwrap()
            .got_input
    }
    fn was_button_drawn(scene: &mut Scene, name: &ViewId) -> bool {
        scene
            .get_view(name)
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .downcast_ref::<TestButtonState>()
            .unwrap()
            .drawn
    }

    fn repaint(scene: &mut Scene) {
        let theme = MockDrawingContext::make_mock_theme();
        let mut ctx = MockDrawingContext::new(scene);
        draw_scene(scene, &mut ctx, &theme);
        scene.dirty_rect = Bounds::new_empty();
    }

    #[test]
    fn test_pick_at() {
        let mut scene: Scene = Scene::new();
        let vbox = make_vbox(&"parent".into(), Bounds::new(10, 10, 100, 100));

        let mut button = make_test_button(&ViewId::new("child"));
        button.bounds = Bounds::new(10, 10, 10, 10);

        scene.add_view_to_parent(button, &vbox.name);
        scene.add_view_to_root(vbox);
        assert_eq!(pick_at(&mut scene, &Point { x: 5, y: 5 }).len(), 1);
        assert_eq!(pick_at(&mut scene, &Point { x: 15, y: 15 }).len(), 2);
        assert_eq!(pick_at(&mut scene, &Point { x: 25, y: 25 }).len(), 3);
    }
    #[test]
    fn test_layout() {
        let parent: ViewId = "parent".into();
        let theme = MockDrawingContext::make_mock_theme();
        let mut scene: Scene = Scene::new();
        // add panel
        scene.add_view(make_vbox(&parent, Bounds::new(10, 10, 100, 100)));
        // add button 1
        scene.add_view_to_parent(make_test_button(&ViewId::new("button1")), &parent);
        // add button 2
        scene.add_view_to_parent(make_label(&"button2".into()), &parent);
        // layout
        let space = scene.bounds.size.clone();
        layout_vbox(&mut LayoutEvent {
            scene: &mut scene,
            target: &"parent".into(),
            theme: &theme,
            space: space,
        });
        assert_eq!(
            scene.get_view_bounds(&"parent".into()),
            Some(Bounds::new(10, 10, 100, 100))
        );
        assert_eq!(
            scene.get_view_bounds(&"button1".into()),
            Some(Bounds::new(0, 0, 100, 20)),
        );
        assert_eq!(
            scene.get_view_bounds(&"button2".into()),
            Some(Bounds::new(0, 20, 100, 20))
        );
    }
    #[test]
    fn test_repaint() {
        let mut scene = Scene::new();
        // add panel
        scene.add_view(make_vbox(&"parent".into(), Bounds::new(10, 10, 100, 100)));
        // add button 1
        scene.add_view(make_test_button(&ViewId::new("button1")));
        // add button 2
        scene.add_view(make_test_button(&ViewId::new("button2")));

        assert_eq!(scene.dirty, true);
        repaint(&mut scene);
        assert_eq!(scene.dirty, false);
    }
    #[test]
    fn test_events() {
        let mut scene: Scene = Scene::new();
        let mut handlers: Vec<Callback> = vec![];
        handlers.push(|event| {
            info!("got an event {:?}", event);
            if let Some(view) = event.scene.get_view_mut(event.target) {
                view.visible = false;
            }
            event.scene.dirty = true;
        });
        handlers.push(|event| {
            info!("got another event {:?}", event);
            if let Some(view) = event.scene.get_view_mut(event.target) {
                view.visible = false;
            }
            event.scene.dirty = true;
            info!("the action is {:?}", event.action);
        });
        assert_eq!(scene.get_view(&"root".into()).unwrap().visible, true);
        click_at(&mut scene, &handlers, Point::new(5, 5));
        assert_eq!(scene.get_view(&"root".into()).unwrap().visible, false);
    }
    fn handle_toggle_button_input(event: &mut GuiEvent) -> Option<OutputAction> {
        // info!("view clicked {:?}", event.event_type);
        if let Some(view) = event.scene.get_view_mut(event.target) {
            view.state.insert(Box::new(String::from("enabled")));
        }
        None
    }
    #[test]
    fn test_toggle_button() {
        let mut scene = Scene::new();
        // add toggle button
        let button = View {
            name: ViewId::new("toggle"),
            title: String::from("Off"),
            visible: true,
            bounds: Bounds::new(10, 10, 20, 20),
            draw: Some(|e| {
                if let Some(state) = &e.view.state {
                    if let Some(state) = state.downcast_ref::<String>() {
                        if state == "enabled" {
                            e.ctx.fill_rect(&e.view.bounds, &e.theme.standard.text);
                            e.ctx.stroke_rect(&e.view.bounds, &e.theme.standard.fill);
                            let style = TextStyle::new(&e.theme.font, &e.theme.standard.fill)
                                .with_halign(Align::Center);
                            e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
                        } else {
                            e.ctx.fill_rect(&e.view.bounds, &e.theme.standard.fill);
                            e.ctx.stroke_rect(&e.view.bounds, &e.theme.standard.text);
                            let style = TextStyle::new(&e.theme.font, &e.theme.standard.text)
                                .with_halign(Align::Center);
                            e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
                        }
                    }
                }
            }),
            input: Some(handle_toggle_button_input),
            state: Some(Box::new(String::from("disabled"))),
            layout: None,
            ..Default::default()
        };
        scene.add_view_to_root(button);
        // repaint
        repaint(&mut scene);
        assert_eq!(scene.get_view(&"toggle".into()).unwrap().visible, true);
        assert_eq!(
            &scene
                .get_view(&"toggle".into())
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .downcast_ref::<String>()
                .unwrap(),
            &"disabled"
        );
        // click at
        let handlers = vec![];
        click_at(&mut scene, &handlers, Point::new(15, 15));
        // confirm toggle button state has changed to enabled
        assert_eq!(
            &scene
                .get_view(&"toggle".into())
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .downcast_ref::<String>()
                .unwrap(),
            &"enabled"
        );
    }
    #[test]
    fn test_make_visible() {
        // create scene
        let mut scene = Scene::new();

        // create button 1
        let mut button1 = make_test_button(&ViewId::new("button1"));
        button1.visible = true;
        scene.add_view_to_root(button1);

        // create button 2
        let mut button2 = make_test_button(&ViewId::new("button2"));
        button2.bounds.position.x = 100;
        // make button 2 invisible
        button2.visible = false;
        scene.add_view_to_root(button2);

        assert_eq!(was_button_clicked(&mut scene, &"button1".into()), false);
        assert_eq!(was_button_drawn(&mut scene, &"button1".into()), false);
        assert_eq!(was_button_drawn(&mut scene, &"button2".into()), false);

        // repaint. only button 1 should get drawn
        repaint(&mut scene);
        assert_eq!(scene.dirty, false);
        assert_eq!(was_button_drawn(&mut scene, &"button1".into()), true);
        assert_eq!(was_button_drawn(&mut scene, &"button2".into()), false);

        let mut handlers: Vec<Callback> = vec![];
        handlers.push(|e| {
            info!("clicked on {}", e.target);
            if let Some(view) = e.scene.get_view_mut(&"button2".into()) {
                view.visible = true;
                e.scene.dirty = true;
            }
        });

        // tap button 1
        assert_eq!(scene.dirty, false);
        click_at(&mut scene, &handlers, Point::new(15, 15));
        assert_eq!(was_button_clicked(&mut scene, &"button1".into()), true);
        // confirm dirty
        assert_eq!(scene.dirty, true);

        // this time both buttons should be drawn
        repaint(&mut scene);
        assert_eq!(scene.dirty, false);
        assert_eq!(was_button_drawn(&mut scene, &"button1".into()), true);
        assert_eq!(was_button_drawn(&mut scene, &"button2".into()), true);
    }
    #[test]
    fn test_keyboard_events() {
        // make scene
        let mut scene: Scene = Scene::new();

        // make text box
        let text_box = make_text_box(&ViewId::new("textbox1"), "foo");
        scene.add_view_to_root(text_box);
        // confirm text is correct
        assert_eq!(get_view_title(&scene, ViewId::new("textbox1")), "foo");
        // set text box as focused
        scene.focused = Some("textbox1".into());

        // send keyboard event
        event_at_focused(&mut scene, &InputEvent::Text(TextAction::TypedAscii(b'X')));
        // confirm text is updated
        assert_eq!(get_view_title(&scene, ViewId::new("textbox1")), "fooX");
    }

    #[test]
    fn test_draw2() {
        let mut scene = Scene::new();
        let view = View {
            name: "view".into(),
            title: "view".into(),
            bounds: Bounds::new(0, 0, 10, 10),
            visible: true,
            draw: Some(|e| {
                let mut color = &e.theme.standard.text;
                if e.focused.is_some() && e.view.name.eq(e.focused.as_ref().unwrap()) {
                    color = &e.theme.standard.fill;
                }
                e.ctx.fill_rect(&e.view.bounds, color);
            }),
            state: None,
            input: None,
            layout: None,
            ..Default::default()
        };

        scene.add_view_to_root(view);
        repaint(&mut scene);
    }

    #[test]
    fn test_cliprect() {
        // make scene
        let mut scene = Scene::new();
        // add button
        let button = make_button(&"button".into(), "Button").position_at(20, 20);
        scene.add_view_to_root(button);
        assert_eq!(scene.dirty, true);
        // check that dirty area is same as bounds
        assert_eq!(scene.dirty_rect, scene.bounds);
        assert_eq!(scene.dirty_rect.is_empty(), false);
        // draw
        repaint(&mut scene);
        // check that dirty area is empty
        assert_eq!(scene.dirty, false);
        assert_eq!(scene.dirty_rect.is_empty(), true);
        // send tap to button
        click_at(&mut scene, &vec![], Point::new(30, 30));
        // check that dirty area is just for the button
        assert_eq!(scene.dirty, true);
        assert_eq!(
            scene.dirty_rect,
            scene.get_view(&"button".into()).unwrap().bounds
        );
        // draw
        repaint(&mut scene);
        assert_eq!(scene.dirty, false);
        assert_eq!(scene.dirty_rect.is_empty(), true);
        // check that button was redrawn
    }
    #[test]
    fn test_cliprect_nested() {
        let mut scene = Scene::new();
        let panel1 = View {
            name: "panel1".into(),
            bounds: Bounds::new(10, 10, 100, 100),
            ..Default::default()
        };
        scene.add_view_to_root(panel1);
        let panel2 = View {
            name: "panel2".into(),
            bounds: Bounds::new(10, 10, 100, 100),
            ..Default::default()
        };
        scene.add_view_to_parent(panel2, &("panel1".into()));
        let button_id = ViewId::new("button");
        let button = make_button(&button_id, "Button").position_at(20, 20);
        scene.add_view_to_parent(button, &("panel2".into()));

        // draw
        repaint(&mut scene);
        // check that dirty area is empty
        assert_eq!(scene.dirty, false);
        assert_eq!(scene.dirty_rect.is_empty(), true);
        // nothing should be focused yet
        assert!(scene.focused.is_none());

        click_at(&mut scene, &vec![], Point::new(45, 45));
        scene.dump();
        // now the button should be focused
        assert!(scene.focused.is_some());
        assert!(scene.focused.is_some_and(|id| id == button_id));
        assert_eq!(scene.dirty, true);
        assert_eq!(scene.dirty_rect, Bounds::new(40, 40, 100, 100));
    }

    fn get_view_title(scene: &Scene, name: ViewId) -> String {
        scene.get_view(&name).unwrap().title.clone()
    }
}
