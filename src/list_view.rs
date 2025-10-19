use crate::geom::Bounds;
use crate::gfx::draw_centered_text;
use crate::input::{InputEvent, OutputAction, TextAction};
use crate::view::{View, ViewId};
use crate::{DrawEvent, GuiEvent, LayoutEvent};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;
use core::option::Option::Some;
use log::info;

pub fn make_list_view(name: &ViewId, data: Vec<&str>, selected: usize) -> View {
    View {
        name: name.clone(),
        title: name.to_string(),
        state: Some(ListState::new_with(data, selected)),
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
        if self.selected < self.items.len() - 1 {
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
    pub fn new_with(items: Vec<&str>, selected: usize) -> Box<dyn Any> {
        Box::new(ListState {
            items: items.iter().map(|s| s.to_string()).collect(),
            selected,
        })
    }
}

fn input_list(e: &mut GuiEvent) -> Option<OutputAction> {
    match &e.event_type {
        InputEvent::Tap(pt) => {
            e.scene.mark_dirty_view(e.target);
            e.scene.set_focused(e.target);
            if let Some(view) = e.scene.get_view_mut(e.target) {
                let bounds = view.bounds;
                if let Some(state) = view.get_state::<ListState>() {
                    let cell_height = bounds.h() / (state.items.len() as i32);
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
                        info!("firmly selecting the item");
                        return Some(OutputAction::Command(
                            state.items[state.selected as usize].clone(),
                        ));
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
        let cell_height = bounds.h() / (state.items.len() as i32);
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
            draw_centered_text(e.ctx, item, &bds, &e.theme.font, &style.text);
        }
    }
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.standard.text);
}

fn layout_list(e: &mut LayoutEvent) {
    if let Some(state) = e.scene.get_view_state::<ListState>(e.target) {
        let ch = e.theme.font.character_size;
        let height = state.items.len() as u32 * ch.height * 2;
        if let Some(view) = e.scene.get_view_mut(e.target) {
            view.bounds.size.h = height as i32;
        }
    }
}
mod tests {
    use crate::geom::{Bounds, Point};
    use crate::list_view::{ListState, make_list_view};
    use crate::scene::{Scene, click_at, draw_scene, layout_scene};
    use crate::test::MockDrawingContext;
    use crate::view::ViewId;
    use alloc::vec;

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
            let mut group = scene.get_view_mut(&listview).unwrap();
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
}
