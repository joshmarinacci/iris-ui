use crate::geom::Bounds;
use crate::gfx::draw_centered_text;
use crate::input::{InputEvent, OutputAction, TextAction};
use crate::view::{View, ViewId};
use crate::{DrawEvent, GuiEvent, LayoutEvent};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::option::Option::Some;
use log::info;

pub fn make_list_view(name: &ViewId, data: Vec<&str>, selected: usize) -> View {
    View {
        name: name.clone(),
        title: name.to_string(),
        state: Some(Box::new(ListState::new_with(data, selected))),
        input: Some(input_list),
        layout: Some(layout_list),
        draw: Some(draw_list),
        ..Default::default()
    }
}

pub struct ListState {
    pub items: Vec<String>,
    pub selected: usize,
}

impl ListState {
    pub fn select_next(&mut self) {
        if !self.items.is_empty() && self.selected < self.items.len() - 1 {
            self.selected += 1;
        }
    }
    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }
}

impl ListState {
    pub fn new_with(items: Vec<&str>, selected: usize) -> Self {
        ListState {
            items: items.iter().map(|s| s.to_string()).collect(),
            selected,
        }
    }
}

fn input_list(e: &mut GuiEvent) -> Option<OutputAction> {
    match &e.event_type {
        InputEvent::PointerUp(pt) => {
            e.scene.mark_dirty_view(e.target);
            e.scene.set_focused(e.target);
            if let Some(view) = e.scene.get_view_mut(e.target) {
                let bounds = view.bounds;
                if let Some(state) = view.get_state::<ListState>() {
                    if state.items.is_empty() {
                        return None;
                    }
                    let cell_height = bounds.h() / (state.items.len() as i32);
                    if cell_height <= 0 {
                        return None;
                    }
                    let y = pt.y - bounds.y();
                    let n = y / cell_height;
                    if n >= 0 && n < state.items.len() as i32 {
                        state.selected = n as usize;
                        return Some(OutputAction::Command(state.items[state.selected].clone()));
                    }
                }
            }
        }
        InputEvent::Scroll(delta) => {
            e.scene.mark_dirty_view(e.target);
            if let Some(state) = e.scene.get_view_state::<ListState>(e.target) {
                if delta.y > 0 {
                    state.select_next();
                }
                if delta.y < 0 {
                    state.select_prev();
                }
            }
        }
        InputEvent::Text(action) => {
            e.scene.mark_dirty_view(e.target);
            if let Some(state) = e.scene.get_view_state::<ListState>(e.target) {
                match action {
                    TextAction::Up => state.select_prev(),
                    TextAction::Down => state.select_next(),
                    TextAction::Enter => {
                        if let Some(item) = state.items.get(state.selected) {
                            info!("firmly selecting the item");
                            return Some(OutputAction::Command(item.clone()));
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    None
}

fn draw_list(e: &mut DrawEvent) {
    let bounds = e.view.bounds;
    e.ctx.fill_rect(&e.view.bounds, &e.theme.standard.fill);
    let name = e.view.name.clone();
    if let Some(state) = e.view.get_state::<ListState>() {
        if state.items.is_empty() {
            return;
        }
        let cell_height = bounds.h() / (state.items.len() as i32);
        if cell_height <= 0 {
            return;
        }
        for (i, item) in state.items.iter().enumerate() {
            let style = if i == state.selected {
                &e.theme.selected
            } else {
                &e.theme.standard
            };
            let bds = Bounds::new(
                bounds.x(),
                bounds.y() + (i as i32) * cell_height + 1,
                bounds.w(),
                cell_height - 1,
            );
            // draw background only if selected
            if i == state.selected {
                e.ctx.fill_rect(&bds, &style.fill);
                if let Some(focused) = e.focused {
                    if focused == &name {
                        e.ctx.stroke_rect(&bds.contract(2), &style.text);
                    }
                }
            }

            // draw text
            draw_centered_text(e.ctx, item, &bds, e.theme.font, &style.text);
        }
    }
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.standard.text);
}

fn layout_list(e: &mut LayoutEvent) {
    if let Some(state) = e.scene.get_view_state::<ListState>(e.target) {
        let ch = e.theme.font.char_height();
        let content_height = state.items.len() as i32 * ch * 2;
        if let Some(view) = e.scene.get_view_mut(e.target) {
            view.bounds.size.h = if view.v_flex == crate::view::Flex::Grow {
                e.space.h
            } else {
                content_height
            };
            if view.h_flex == crate::view::Flex::Grow {
                view.bounds.size.w = e.space.w;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::geom::{Bounds, Point};
    use crate::layouts::layout_vbox;
    use crate::list_view::{ListState, make_list_view};
    use crate::panel::PanelState;
    use crate::scene::{Scene, click_at, draw_scene, layout_scene};
    use crate::test::MockDrawingContext;
    use crate::view::{Flex, View, ViewId};
    use alloc::boxed::Box;
    use alloc::vec;

    #[test]
    fn test_list_view_enter_with_stale_selected_no_panic() {
        use crate::input::{InputEvent, TextAction};
        use crate::scene::event_at_focused;
        let theme = MockDrawingContext::make_mock_theme();
        let mut scene: Scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));
        let listview = ViewId::new("listview");
        {
            let list = make_list_view(&listview, vec!["A", "B", "C"], 2);
            scene.add_view_to_root(list);
        }
        layout_scene(&mut scene, &theme);
        scene.set_focused(&listview);
        // Shrink items so selected index (2) is now out of bounds
        {
            let view = scene.get_view_mut(&listview).unwrap();
            let state = view.get_state::<ListState>().unwrap();
            state.items.truncate(1);
        }
        // Enter should not panic
        event_at_focused(&mut scene, &InputEvent::Text(TextAction::Enter));
    }

    #[test]
    fn test_list_view() {
        let theme = MockDrawingContext::make_mock_theme();
        let mut scene: Scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

        let listview = ViewId::new("listview");
        {
            let list = make_list_view(&listview, vec!["A", "BB", "CCC"], 0);
            scene.add_view_to_root(list);
        }
        layout_scene(&mut scene, &theme);

        {
            let group = scene.get_view_mut(&listview).unwrap();
            let state = &mut group.get_state::<ListState>().unwrap();
            assert_eq!(state.selected, 0);
        }

        click_at(&mut scene, &vec![], Point::new(50, 30));

        {
            let state = &mut scene
                .get_view_state::<ListState>(&ViewId::new("listview"))
                .unwrap();
            assert_eq!(state.selected, 1);
        }

        let mut ctx = MockDrawingContext::new(&scene);
        assert_eq!(scene.dirty, true);
        draw_scene(&mut scene, &mut ctx, &theme);
        assert_eq!(scene.dirty, false);
    }

    #[test]
    fn test_list_view_grows_to_fill_parent_width() {
        let theme = MockDrawingContext::make_mock_theme();
        // 320×240 scene; parent vbox fills it entirely
        let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

        let parent_id: ViewId = "parent".into();
        let parent = View {
            name: parent_id.clone(),
            h_flex: Flex::Grow,
            v_flex: Flex::Grow,
            layout: Some(layout_vbox),
            state: Some(Box::new(PanelState::default())),
            ..Default::default()
        };
        scene.add_view_to_parent(parent, &scene.root_id());

        let list_id: ViewId = "list".into();
        let mut list = make_list_view(&list_id, vec!["A", "B", "C"], 0);
        list.h_flex = Flex::Grow;
        scene.add_view_to_parent(list, &parent_id);

        layout_scene(&mut scene, &theme);

        let list_w = scene.get_view(&list_id).unwrap().bounds.size.w;
        assert_eq!(list_w, 320, "list width should fill the 320px parent width");
    }
}
