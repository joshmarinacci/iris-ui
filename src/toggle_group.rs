use crate::geom::{Bounds, Point};
use crate::gfx::draw_centered_text;
use crate::input::{InputEvent, OutputAction};
use crate::view::Flex::{Grow, Shrink};
use crate::view::{View, ViewId};
use crate::{DrawEvent, GuiEvent, LayoutEvent};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::option::Option::Some;
use embedded_graphics::pixelcolor::PixelColor;

pub fn make_toggle_group<C: PixelColor>(name: &ViewId, data: Vec<&str>, selected: usize) -> View<C> {
    View {
        name: name.clone(),
        title: name.to_string(),
        bounds: Bounds::new(0, 0, (data.len() * 60) as i32, 30),
        state: Some(Box::new(SelectOneOfState::new_with(data, selected))),
        input: Some(input_toggle_group),
        layout: Some(layout_toggle_group),
        draw: Some(draw_toggle_group),
        visible: true,
        h_flex: Grow,
        v_flex: Shrink,
        ..Default::default()
    }
}

pub struct SelectOneOfState {
    pub items: Vec<String>,
    pub selected: usize,
    pressed: Option<usize>,
}

impl SelectOneOfState {
    pub fn new_with(items: Vec<&str>, selected: usize) -> Self {
        SelectOneOfState {
            items: items.iter().map(|s| s.to_string()).collect(),
            selected,
            pressed: None,
        }
    }
}

/// Which segment (if any) a point falls over, given the group's bounds and item count.
fn cell_index_at(bounds: Bounds, item_count: usize, pt: Point) -> Option<usize> {
    if item_count == 0 {
        return None;
    }
    let cell_width = bounds.size.w / (item_count as i32);
    let x = pt.x - bounds.x();
    let n = x / cell_width;
    if n >= 0 && n < item_count as i32 {
        Some(n as usize)
    } else {
        None
    }
}

pub fn input_toggle_group<C: PixelColor>(e: &mut GuiEvent<C>) -> Option<OutputAction> {
    match &e.event_type {
        InputEvent::PointerDown(pt) => {
            let pt = *pt;
            e.scene.set_focused(e.target);
            if let Some(view) = e.scene.get_view_mut(e.target) {
                let bounds = view.bounds;
                if let Some(state) = view.get_state::<SelectOneOfState>() {
                    state.pressed = cell_index_at(bounds, state.items.len(), pt);
                }
            }
            e.scene.mark_dirty_view(e.target);
        }
        InputEvent::PointerUp(pt) => {
            let pt = *pt;
            e.scene.mark_dirty_view(e.target);
            if let Some(view) = e.scene.get_view_mut(e.target) {
                let bounds = view.bounds;
                if let Some(state) = view.get_state::<SelectOneOfState>() {
                    let pressed = state.pressed.take();
                    if state.items.is_empty() {
                        return None;
                    }
                    // Only commit if released over the same segment that was pressed,
                    // so dragging off the group before lifting cancels the selection.
                    if let Some(n) = cell_index_at(bounds, state.items.len(), pt) {
                        if pressed == Some(n) {
                            state.selected = n;
                            return Some(OutputAction::Command(state.items[n].clone()));
                        }
                    }
                }
            }
        }
        _ => {}
    }
    None
}

fn draw_toggle_group<C: PixelColor>(e: &mut DrawEvent<C>) {
    let bounds = e.view.bounds;
    e.ctx.fill_rect(&e.view.bounds, &e.theme.standard.fill);
    let name = e.view.name.clone();
    if let Some(state) = e.view.get_state::<SelectOneOfState>() {
        if state.items.is_empty() {
            return;
        }
        let cell_width = bounds.size.w / (state.items.len() as i32);
        for (i, item) in state.items.iter().enumerate() {
            let is_selected = i == state.selected;
            let is_pressed = state.pressed == Some(i);
            let style = if is_selected {
                e.theme.selected
            } else {
                e.theme.standard
            };
            // While pressed, invert the segment's fill/text so the touch is visible
            // before release commits the selection (mirrors button.rs's pressed state).
            let (fill, text) = if is_pressed {
                (style.text, style.fill)
            } else {
                (style.fill, style.text)
            };
            let bds = Bounds::new(
                bounds.position.x + (i as i32) * cell_width + 1,
                bounds.y(),
                cell_width - 1,
                bounds.h(),
            );
            // draw background only if selected or currently pressed
            if is_selected || is_pressed {
                e.ctx.fill_rect(&bds, &fill);
                if let Some(focused) = e.focused {
                    if focused == &name {
                        e.ctx.stroke_rect(&bds.contract(2), &text);
                    }
                }
            }

            // draw text
            draw_centered_text(e.ctx, item, &bds, e.theme.font, &text);

            // draw left edge except for the first one
            if i != 0 {
                let x = bounds.position.x + (i as i32) * cell_width;
                e.ctx.line(
                    &Point::new(x, bounds.y()),
                    &Point::new(x, bounds.position.y + bounds.size.h - 1),
                    &e.theme.standard.text,
                );
            }
        }
    }
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.standard.text);
}

pub fn layout_toggle_group<C: PixelColor>(pass: &mut LayoutEvent<C>) {
    if let Some(view) = pass.scene.get_view_mut(pass.target) {
        let cw = pass.theme.font.char_width();
        let ch = pass.theme.font.char_height();
        let height = ch + (ch / 2) * 2;
        if view.h_flex == Grow {
            view.bounds.size.w = pass.space.w;
        }
        if view.h_flex == Shrink {
            if let Some(state) = view.get_state::<SelectOneOfState>() {
                let mut width = 0;
                for item in &state.items {
                    width += cw;
                    width += pass.theme.font.str_width(item);
                    width += cw;
                }
                view.bounds.size.w = width;
            }
        }
        if view.v_flex == Grow {
            view.bounds.size.h = pass.space.h;
        }
        if view.v_flex == Shrink {
            view.bounds.size.h = height;
        }
    }
    pass.layout_all_children(&pass.target.clone(), pass.space);
}

#[cfg(test)]
mod tests {
    use crate::geom::{Bounds, Point};
    use crate::scene::{Scene, click_at, draw_scene, layout_scene};
    use crate::test::MockDrawingContext;
    use crate::toggle_group::{SelectOneOfState, make_toggle_group};
    use crate::view::ViewId;
    use alloc::vec;
    use embedded_graphics::pixelcolor::Rgb565;

    #[test]
    fn test_toggle_group_empty_no_panic() {
        let theme = MockDrawingContext::make_mock_theme();
        let mut scene: Scene<Rgb565> = Scene::new_with_bounds(Bounds::new(0, 0, 100, 240));
        {
            let group = make_toggle_group(&ViewId::new("group"), vec![], 0);
            scene.add_view_to_root(group);
        }
        layout_scene(&mut scene, &theme);
        // tap should not panic
        click_at(&mut scene, &vec![], Point::new(50, 10));
        // draw should not panic
        let mut ctx = MockDrawingContext::new(&scene);
        draw_scene(&mut scene, &mut ctx, &theme);
    }

    #[test]
    fn test_toggle_group() {
        let theme = MockDrawingContext::make_mock_theme();
        let mut scene: Scene<Rgb565> = Scene::new_with_bounds(Bounds::new(0, 0, 100, 240));
        {
            let group = make_toggle_group(&ViewId::new("group"), vec!["A", "BB", "CCC"], 0);
            scene.add_view_to_root(group);
        }
        layout_scene(&mut scene, &theme);

        {
            let group = scene.get_view_mut(&ViewId::new("group")).unwrap();
            assert_eq!(group.name.as_str(), "group");
            assert_eq!(group.bounds, Bounds::new(0, 0, 100, 13 + 7));
            let state = &mut group.get_state::<SelectOneOfState>().unwrap();
            assert_eq!(state.items, vec!["A", "BB", "CCC"]);
            assert_eq!(state.selected, 0);
        }

        click_at(&mut scene, &vec![], Point::new(50, 10));

        {
            let state = &mut scene
                .get_view_state::<SelectOneOfState>(&ViewId::new("group"))
                .unwrap();
            assert_eq!(state.items, vec!["A", "BB", "CCC"]);
            assert_eq!(state.selected, 1);
        }

        let mut ctx = MockDrawingContext::new(&scene);
        assert_eq!(scene.dirty, true);
        draw_scene(&mut scene, &mut ctx, &theme);
        assert_eq!(scene.dirty, false);
    }
}
