use crate::geom::{Bounds, Insets, Point};
use crate::view::{Align, Flex, View, ViewId};
use crate::{DrawEvent, LayoutEvent};
use alloc::boxed::Box;
use alloc::string::ToString;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use hashbrown::HashMap;

pub struct GridLayoutState {
    constraints: HashMap<ViewId, LayoutConstraint>,
    row_count: usize,
    col_count: usize,
    col_width: usize,
    row_height: usize,
    pub debug: bool,
    pub gap: i32,
    pub border_visible: bool,
    pub padding: Insets,
}

impl GridLayoutState {
    pub fn new_row_column(
        row_count: usize,
        row_height: usize,
        col_count: usize,
        col_width: usize,
    ) -> GridLayoutState {
        GridLayoutState {
            constraints: HashMap::new(),
            col_count,
            row_count,
            col_width,
            row_height,
            debug: false,
            border_visible: true,
            gap: 0,
            padding: Insets::new_same(0),
        }
    }
}

impl GridLayoutState {
    pub fn place_at_row_column(
        &mut self,
        name: &ViewId,
        row: usize,
        col: usize,
    ) -> Option<LayoutConstraint> {
        self.constraints
            .insert(name.clone(), LayoutConstraint::at_row_column(row, col))
    }
    pub fn place_at_row_column_with_spans(
        &mut self,
        name: &ViewId,
        row: usize,
        col: usize,
        row_span: usize,
        col_span: usize,
    ) {
        self.constraints.insert(
            name.clone(),
            LayoutConstraint {
                col,
                row,
                col_span,
                row_span,
            },
        );
    }
}

pub struct LayoutConstraint {
    pub col: usize,
    pub row: usize,
    pub col_span: usize,
    pub row_span: usize,
}

impl LayoutConstraint {
    pub fn at_row_column(row: usize, col: usize) -> LayoutConstraint {
        LayoutConstraint {
            col,
            row,
            col_span: 1,
            row_span: 1,
        }
    }
}

pub fn make_grid_panel(name: &ViewId) -> View {
    View {
        name: name.clone(),
        title: name.to_string(),
        state: Some(Box::new(GridLayoutState {
            constraints: HashMap::new(),
            col_count: 2,
            row_count: 2,
            col_width: 100,
            row_height: 30,
            debug: false,
            gap: 0,
            padding: Insets::new_same(0),
            border_visible: false,
        })),
        layout: Some(layout_grid),
        draw: Some(draw_grid),
        visible: true,
        ..Default::default()
    }
}

fn draw_grid(evt: &mut DrawEvent) {
    let bounds = evt.view.bounds;
    evt.ctx
        .fill_rect(&evt.view.bounds, &evt.theme.panel.fill);
    if let Some(state) = evt.view.get_state::<GridLayoutState>() {
        let padding = state.padding;
        if state.border_visible {
            evt.ctx.stroke_rect(&bounds, &evt.theme.standard.text);
        }
        if state.debug {
            for i in 0..state.col_count + 1 {
                let x = (i * state.col_width) as i32 + bounds.x() + padding.left;
                let y = bounds.y();
                let y2 = bounds.y() + bounds.h();
                evt.ctx
                    .line(&Point::new(x, y), &Point::new(x, y2), &Rgb565::RED);
                evt.ctx
                    .line(&Point::new(x - state.gap, y), &Point::new(x - state.gap, y2), &Rgb565::YELLOW);
                evt.ctx
                    .line(&Point::new(x + state.gap, y), &Point::new(x + state.gap, y2), &Rgb565::YELLOW);
            }
            for j in 0..state.row_count + 1 {
                let y = (j * state.row_height) as i32 + bounds.y() + padding.top;
                let x = bounds.x();
                let x2 = bounds.x() + bounds.w();
                evt.ctx
                    .line(&Point::new(x, y), &Point::new(x2, y), &Rgb565::RED);
                evt.ctx
                    .line(&Point::new(x, y - state.gap), &Point::new(x2, y - state.gap), &Rgb565::YELLOW);
                evt.ctx
                    .line(&Point::new(x, y + state.gap), &Point::new(x2, y + state.gap), &Rgb565::YELLOW);
            }
        }
    }
}

fn layout_grid(pass: &mut LayoutEvent) {
    let padding = if let Some(state) = pass.scene.get_view_state::<GridLayoutState>(pass.target) {
        state.padding
    } else {
        Insets::new_same(0)
    };
    let Some(view) = pass.scene.get_view_mut(pass.target) else {
        return;
    };
    // let padding = state.padding.clone();
    if view.h_flex == Flex::Grow {
        view.bounds.size.w = pass.space.w;
    }
    if view.h_flex == Flex::Shrink {}
    if view.v_flex == Flex::Grow {
        view.bounds.size.h = pass.space.h;
    }
    if view.v_flex == Flex::Shrink {}

    let parent_bounds = view.bounds.clone();
    let kids = pass.scene.get_children_ids(pass.target);
    let space = parent_bounds.size.clone() - padding;
    for kid in kids {
        pass.layout_child(&kid, space);
        if let Some(state) = pass.scene.get_view_state::<GridLayoutState>(pass.target) {
            let gap = state.gap;
            let cell_bounds = if let Some(cons) = &state.constraints.get(&kid) {
                let x = (cons.col * state.col_width) as i32 + padding.left + gap;
                let y = (cons.row * state.row_height) as i32 + padding.top + gap;
                let w = (cons.col_span * state.col_width) as i32 - gap * 2;
                let h = (cons.row_span * state.row_height) as i32 - gap * 2;
                Bounds::new(x, y, w, h)
            } else {
                Bounds::new(0, 0, 0, 0)
            };

            // pass.layout_child(&kid, space);
            if let Some(view) = pass.scene.get_view_mut(&kid) {
                view.bounds.size.w = match &view.h_flex {
                    Flex::Grow => cell_bounds.w(),
                    Flex::Fixed => view.bounds.size.w,
                    Flex::Shrink => view.bounds.size.w,
                };
                view.bounds.size.h = match &view.v_flex {
                    Flex::Grow => cell_bounds.h(),
                    Flex::Fixed => view.bounds.size.h,
                    Flex::Shrink => view.bounds.size.h,
                };
                view.bounds.position.x = match &view.h_align {
                    Align::Start => cell_bounds.x(),
                    Align::Center => cell_bounds.x() + (cell_bounds.w() - view.bounds.w()) / 2,
                    Align::End => cell_bounds.x() + cell_bounds.w() - view.bounds.w(),
                };
                view.bounds.position.y = match &view.v_align {
                    Align::Start => cell_bounds.y(),
                    Align::Center => cell_bounds.y() + (cell_bounds.h() - view.bounds.h()) / 2,
                    Align::End => cell_bounds.y() + cell_bounds.h() - view.bounds.h(),
                };
            }
        }
    }
}

impl Into<ViewId> for &'static str {
    fn into(self) -> ViewId {
        ViewId::new(self)
    }
}

mod tests {
    use crate::button::make_button;
    use crate::geom::Bounds;
    use crate::grid::{make_grid_panel, GridLayoutState, LayoutConstraint};
    use crate::label::make_label;
    use crate::scene::{draw_scene, layout_scene, Scene};
    use crate::test::MockDrawingContext;
    use crate::view::Align::Start;
    use crate::view::ViewId;
    use alloc::boxed::Box;

    #[test]
    fn test_grid_layout() {
        let theme = MockDrawingContext::make_mock_theme();

        let mut grid = make_grid_panel(&ViewId::new("grid"));
        grid.bounds = Bounds::new(40, 40, 200, 200);
        let mut grid_layout = GridLayoutState::new_row_column(2, 30, 2, 100);

        let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

        let mut label1 = make_label("label1", "Label 1");
        label1.h_align = Start;
        label1.v_align = Start;
        grid_layout.place_at_row_column(&label1.name, 0, 0);
        scene.add_view_to_parent(label1, &grid.name);

        let mut label2 = make_label("label2", "Label 2");
        label2.h_align = Start;
        label2.v_align = Start;
        grid_layout.place_at_row_column(&label2.name, 0, 1);
        scene.add_view_to_parent(label2, &grid.name);

        let mut label3 = make_label("label3", "Label 3");
        label3.h_align = Start;
        label3.v_align = Start;
        grid_layout.place_at_row_column(&label3.name, 1, 0);
        scene.add_view_to_parent(label3, &grid.name);

        grid.state = Some(Box::new(grid_layout));
        scene.add_view_to_root(grid);

        layout_scene(&mut scene, &theme);

        {
            let label1 = scene.get_view(&ViewId::new("label1")).unwrap();
            assert_eq!(label1.name, ViewId::new("label1"));
            assert_eq!(label1.bounds, Bounds::new(0, 0, 54, 20));

            let label2 = scene.get_view(&ViewId::new("label2")).unwrap();
            assert_eq!(label2.name, ViewId::new("label2"));
            assert_eq!(label2.bounds, Bounds::new(100, 0, 54, 20));

            let label3 = scene.get_view(&ViewId::new("label3")).unwrap();
            assert_eq!(label3.name, ViewId::new("label3"));
            assert_eq!(label3.bounds, Bounds::new(0, 30, 54, 20));
        }

        let mut ctx = MockDrawingContext::new(&scene);

        assert_eq!(scene.dirty, true);
        draw_scene(&mut scene, &mut ctx, &theme);
        assert_eq!(scene.dirty, false);
    }

    #[test]
    fn col_span() {
        let theme = MockDrawingContext::make_mock_theme();
        let mut grid = make_grid_panel(&ViewId::new("grid"))
            .position_at(0, 0)
            .with_size(200, 200);
        let mut layout = GridLayoutState::new_row_column(2, 30, 2, 100);
        layout.debug = true;
        let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

        let button = make_button(&"b1".into(), "b1");
        layout.constraints.insert(
            button.name.clone(),
            LayoutConstraint {
                col: 0,
                row: 0,
                col_span: 2,
                row_span: 1,
            },
        );

        grid.state = Some(Box::new(layout));
        scene.add_view_to_parent(button, &grid.name);
        scene.add_view_to_root(grid);
        layout_scene(&mut scene, &theme);

        if let Some(view) = scene.get_view(&ViewId::new("b1")) {
            assert_eq!(view.bounds, Bounds::new(86, 2, 28, 25));
        }
    }
}
