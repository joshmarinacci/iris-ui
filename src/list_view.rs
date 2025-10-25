use crate::geom::Bounds;
use crate::gfx::TextStyle;
use crate::input::{InputEvent, OutputAction, TextAction};
use crate::view::Align::Start;
use crate::view::{View, ViewId};
use crate::{DrawEvent, GuiEvent, LayoutEvent};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;
use core::option::Option::Some;
use log::info;

pub fn make_list_view(name: &ViewId, data: Vec<&str>, selected: usize) -> View {
    let data = data.iter().map(|s| s.to_string()).collect();
    make_generic_list::<String>(name, data, selected, render_string)
}
pub fn make_generic_list<T: 'static>(name: &ViewId, items: Vec<T>, selected: usize, renderer: GenericRenderer<T>) -> View {
    View {
        name: name.clone(),
        title: name.to_string(),
        state: Some(Box::new(ListState {
            items,
            renderer,
            selected,
        })),
        input: Some(input_list::<T>),
        layout: Some(layout_list::<T>),
        draw: Some(draw_list::<T>),
        ..Default::default()
    }
}

pub type GenericRenderer<T> = fn(event: &T) -> String;
pub struct ListState<T> {
    items: Vec<T>,
    renderer: GenericRenderer<T>,
    pub selected: usize,
}

impl<T> ListState<T> {
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

fn input_list<T: 'static>(e: &mut GuiEvent) -> Option<OutputAction> {
    match &e.event_type {
        InputEvent::Tap(pt) => {
            e.scene.mark_dirty_view(e.target);
            e.scene.set_focused(e.target);
            if let Some(view) = e.scene.get_view_mut(e.target) {
                let bounds = view.bounds;
                if let Some(state) = view.get_state::<ListState<T>>() {
                    let cell_height = bounds.h() / (state.items.len() as i32);
                    let y = pt.y - bounds.y();
                    let n = y / cell_height;
                    if n >= 0 && n < state.items.len() as i32 {
                        state.selected = n as usize;
                        let text = (state.renderer)(&state.items[state.selected]);
                        return Some(OutputAction::Command(text));
                    }
                }
            }
        }
        InputEvent::Scroll(delta) => {
            e.scene.mark_dirty_view(e.target);
            if let Some(state) = e.scene.get_view_state::<ListState<T>>(e.target) {
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
            if let Some(state) = e.scene.get_view_state::<ListState<T>>(e.target) {
                match action {
                    TextAction::Up => state.select_prev(),
                    TextAction::Down => state.select_next(),
                    TextAction::Enter => {
                        info!("firmly selecting the item");
                        let text = (state.renderer)(&state.items[state.selected]);
                        return Some(OutputAction::Command(text));
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    None
}

fn draw_list<T: 'static>(e: &mut DrawEvent) {
    let bounds = e.view.bounds;
    e.ctx.fill_rect(&e.view.bounds, &e.theme.standard.fill);
    let name = e.view.name.clone();
    if let Some(state) = e.view.get_state::<ListState<T>>() {
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
            let text = (state.renderer)(item);
            e.ctx.fill_text(&bds, &text, &TextStyle {
                font: &e.theme.font,
                color: &style.text,
                halign: Start,
                valign: Start,
                underline: false,
            });
        }
    }
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.standard.text);
}

fn layout_list<T: 'static>(e: &mut LayoutEvent) {
    if let Some(state) = e.scene.get_view_state::<ListState<T>>(e.target) {
        let ch = e.theme.font.character_size;
        let height = state.items.len() as u32 * ch.height * 2;
        if let Some(view) = e.scene.get_view_mut(e.target) {
            view.bounds.size.h = height as i32;
        }
    }
}

fn render_string<T: ToString>(event: &T) -> String {
    event.to_string()
}

mod tests {
    use crate::geom::{Bounds, Point};
    use crate::list_view::{make_list_view, ListState};
    use crate::scene::{click_at, draw_scene, layout_scene, Scene};
    use crate::test::MockDrawingContext;
    use crate::view::ViewId;
    use alloc::string::String;
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
            let state = &mut group.get_state::<ListState<String>>().unwrap();
            assert_eq!(state.selected, 0);
        }

        click_at(&mut scene, &vec![], Point::new(50, 30));

        {
            let state = &mut scene
                .get_view_state::<ListState<String>>(&ViewId::new("listview"))
                .unwrap();
            assert_eq!(state.selected, 1);
        }

        let mut ctx = MockDrawingContext::new(&scene);
        assert_eq!(scene.dirty, true);
        draw_scene(&mut scene, &mut ctx, &theme);
        assert_eq!(scene.dirty, false);
    }
}
