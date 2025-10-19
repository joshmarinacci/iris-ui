use crate::geom::Insets;
use crate::gfx::draw_centered_text;
use crate::input::OutputAction;
use crate::view::{View, ViewId};
use crate::{DrawEvent, GuiEvent, LayoutEvent, util};
use alloc::boxed::Box;

pub fn make_toggle_button(name: &ViewId, title: &str) -> View {
    View {
        name: name.clone(),
        title: title.into(),
        state: Some(Box::new(SelectedState::new())),
        draw: Some(draw_toggle_button),
        layout: Some(layout_toggle_button),
        input: Some(input_toggle_button),
        ..Default::default()
    }
}

pub struct SelectedState {
    pub selected: bool,
}

impl SelectedState {
    pub fn new() -> SelectedState {
        SelectedState { selected: false }
    }
}

fn draw_toggle_button(e: &mut DrawEvent) {
    let style = if let Some(state) = e.view.get_state::<SelectedState>()
        && state.selected
    {
        e.theme.selected
    } else {
        e.theme.standard
    };

    e.ctx.fill_rect(&e.view.bounds, &style.fill);
    e.ctx.stroke_rect(&e.view.bounds, &style.text);
    if let Some(focused) = e.focused {
        let focus_insets = Insets::new_same(2);
        if focused == &e.view.name {
            e.ctx
                .stroke_rect(&((*&e.view.bounds) - focus_insets), &style.text);
        }
    }

    draw_centered_text(
        e.ctx,
        &e.view.title,
        &e.view.bounds,
        &e.theme.font,
        &style.text,
    );
}

fn input_toggle_button(event: &mut GuiEvent) -> Option<OutputAction> {
    if let Some(state) = event.scene.get_view_state::<SelectedState>(event.target) {
        state.selected = !state.selected;
        event.scene.set_focused(event.target);
        event.scene.mark_dirty_view(event.target);
        return Some(OutputAction::Selected("toggle".into(), 0));
    }
    None
}

fn layout_toggle_button(event: &mut LayoutEvent) {
    if let Some(view) = event.scene.get_view_mut(event.target) {
        view.bounds = util::calc_bounds(view.bounds, event.theme.font, &view.title);
    }
}

mod tests {
    use crate::geom::{Bounds, Point};
    use crate::scene::{Scene, click_at, draw_scene, layout_scene};
    use crate::test::MockDrawingContext;
    use crate::toggle_button::{SelectedState, make_toggle_button};
    use crate::view::ViewId;
    use alloc::vec;

    #[test]
    fn test_toggle_button() {
        let theme = MockDrawingContext::make_mock_theme();
        let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));
        {
            let button = make_toggle_button(&ViewId::new("toggle"), "Toggle");
            scene.add_view_to_root(button);
        }
        layout_scene(&mut scene, &theme);

        {
            let mut button = scene.get_view_mut(&ViewId::new("toggle")).unwrap();
            assert_eq!(button.name, ViewId::new("toggle"));
            let ch_size = &theme.font.character_size;
            assert_eq!(
                button.bounds,
                Bounds::new(
                    0,
                    0,
                    (("toggle".len() as u32) * ch_size.width + (ch_size.width) * 2) as i32,
                    (ch_size.height + (ch_size.height / 2) * 2) as i32
                )
            );
            let state = &mut button.get_state::<SelectedState>().unwrap();
            assert_eq!(state.selected, false);
        }

        click_at(&mut scene, &vec![], Point::new(10, 10));

        {
            let state = scene
                .get_view_state::<SelectedState>(&ViewId::new("toggle"))
                .unwrap();
            assert_eq!(state.selected, true);
        }

        let mut ctx: MockDrawingContext = MockDrawingContext::new(&scene);

        assert_eq!(scene.dirty, true);
        draw_scene(&mut scene, &mut ctx, &theme);
        assert_eq!(scene.dirty, false);
    }
}
