use crate::gfx::draw_centered_text;
use crate::input::{InputEvent, OutputAction};
use crate::util;
use crate::view::Flex::Shrink;
use crate::view::{View, ViewId};
use alloc::boxed::Box;
use alloc::string::{String, ToString};

pub struct ButtonState {
    command: String,
    primary: bool,
    pressed: bool,
}

pub fn make_button(name: &ViewId, title: &str) -> View {
    make_full_button(name, title, title.into(), false)
}
pub fn make_full_button(name: &ViewId, title: &str, command: &str, primary: bool) -> View {
    View {
        name: name.clone(),
        title: title.to_string(),
        h_flex: Shrink,
        v_flex: Shrink,
        state: Some(Box::new(ButtonState {
            primary,
            command: command.into(),
            pressed: false,
        })),
        input: Some(|e| {
            match &e.event_type {
                InputEvent::PointerDown(_pt) => {
                    e.scene.set_focused(e.target);
                    if let Some(state) = e.scene.get_view_state::<ButtonState>(e.target) {
                        state.pressed = true;
                    }
                    e.scene.mark_dirty_view(e.target);
                }
                InputEvent::PointerUp(_pt) => {
                    let was_pressed = e
                        .scene
                        .get_view_state::<ButtonState>(e.target)
                        .is_some_and(|state| core::mem::replace(&mut state.pressed, false));
                    e.scene.mark_dirty_view(e.target);
                    if was_pressed {
                        if let Some(state) = e.scene.get_view_state::<ButtonState>(e.target) {
                            return Some(OutputAction::Command(state.command.clone()));
                        }
                    }
                }
                _ => {}
            }
            None
        }),
        layout: Some(|e| {
            if let Some(view) = e.scene.get_view_mut(&e.target) {
                view.bounds.size = util::calc_size(e.theme.bold_font, &view.title);
            }
        }),
        draw: Some(|e| {
            let Some(state) = e.view.get_state::<ButtonState>() else {
                return;
            };
            let pressed = state.pressed;
            if state.primary {
                let (fill, text) = if pressed {
                    (e.theme.accented.text, e.theme.accented.fill)
                } else {
                    (e.theme.accented.fill, e.theme.accented.text)
                };
                e.ctx.fill_rect(&e.view.bounds, &fill);
                e.ctx.stroke_rect(&e.view.bounds, &e.theme.standard.text);
                if e.focused == &Some(e.view.name.clone()) {
                    e.ctx.stroke_rect(&e.view.bounds.contract(2), &text);
                }
                draw_centered_text(e.ctx, &e.view.title, &e.view.bounds, e.theme.bold_font, &text);
            } else {
                let (fill, text) = if pressed {
                    (e.theme.standard.text, e.theme.standard.fill)
                } else {
                    (e.theme.standard.fill, e.theme.standard.text)
                };
                e.ctx.fill_rect(&e.view.bounds, &fill);
                e.ctx.stroke_rect(&e.view.bounds, &e.theme.standard.text);
                if e.focused == &Some(e.view.name.clone()) {
                    e.ctx.stroke_rect(&e.view.bounds.contract(2), &text);
                }
                draw_centered_text(e.ctx, &e.view.title, &e.view.bounds, e.theme.bold_font, &text);
            }
        }),
        ..Default::default()
    }
}

#[cfg(test)]
#[cfg(any(feature = "std", feature = "headless"))]
mod tests {
    use super::*;
    use crate::geom::{Bounds, Point};
    use crate::input::OutputAction;
    use crate::scene::{Scene, click_at, draw_scene, pointer_down_at, pointer_up_at};
    use crate::test::MockDrawingContext;
    use alloc::vec;

    fn repaint(scene: &mut Scene) {
        let theme = MockDrawingContext::make_mock_theme();
        let mut ctx = MockDrawingContext::new(scene);
        draw_scene(scene, &mut ctx, &theme);
        scene.dirty_rect = Bounds::new_empty();
    }

    fn make_test_scene() -> Scene {
        let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 200, 200));
        let button = make_button(&"button".into(), "Button").position_at(20, 20);
        scene.add_view_to_root(button);
        repaint(&mut scene);
        scene
    }

    #[test]
    fn pointer_down_sets_pressed_and_focus_but_no_command() {
        let mut scene = make_test_scene();
        let result = pointer_down_at(&mut scene, &vec![], Point::new(30, 30));
        assert!(result.is_none(), "PointerDown alone must not fire a command");
        assert_eq!(scene.get_focused(), Some("button".into()));
        assert!(
            scene
                .get_view_state::<ButtonState>(&"button".into())
                .unwrap()
                .pressed
        );
    }

    #[test]
    fn pointer_up_after_down_fires_command_and_clears_pressed() {
        let mut scene = make_test_scene();
        pointer_down_at(&mut scene, &vec![], Point::new(30, 30));
        let result = pointer_up_at(&mut scene, &vec![], Point::new(30, 30));
        match result.and_then(|r| r.action) {
            Some(OutputAction::Command(cmd)) => assert_eq!(cmd, "Button"),
            other => panic!("expected a Command action, got {other:?}"),
        }
        assert!(
            !scene
                .get_view_state::<ButtonState>(&"button".into())
                .unwrap()
                .pressed,
            "button must not remain visually pressed after release"
        );
    }

    #[test]
    fn pointer_up_without_prior_down_does_not_fire_command() {
        let mut scene = make_test_scene();
        let result = pointer_up_at(&mut scene, &vec![], Point::new(30, 30));
        assert!(
            result.is_none(),
            "a release with no matching press must not trigger the button's action"
        );
    }

    #[test]
    fn click_at_still_performs_a_full_click() {
        let mut scene = make_test_scene();
        let result = click_at(&mut scene, &vec![], Point::new(30, 30));
        match result.and_then(|r| r.action) {
            Some(OutputAction::Command(cmd)) => assert_eq!(cmd, "Button"),
            other => panic!("expected a Command action, got {other:?}"),
        }
    }
}
