use crate::LayoutEvent;
use crate::geom::{Bounds, Size};
use crate::panel::PanelState;
use crate::view::Align::{Center, End, Start};
use crate::view::{Flex, View};
use crate::view::Flex::Grow;
use Flex::{Fixed, Shrink};
use log::info;

pub fn layout_vbox(pass: &mut LayoutEvent) {
    let Some(parent) = pass.scene.get_view_mut(&pass.target) else {
        return;
    };
    let Some(panel_state) = parent.get_state::<PanelState>() else {
        return;
    };
    let gap = panel_state.gap;
    let padding = panel_state.padding.clone();
    let h_flex = parent.h_flex.clone();
    let mut available_space: Size = pass.space - padding;
    if h_flex == Fixed {
        available_space.w = parent.bounds.size.w;
    }

    // get the intrinsic children
    let fixed_kids = pass
        .scene
        .get_children_ids_filtered(&pass.target, |v| v.v_flex == Shrink);
    // lay out the intrinsic children
    for kid in &fixed_kids {
        pass.layout_child(kid, available_space);
    }

    // calculate total used height
    let kids_sum = fixed_kids.iter().fold(0, |a, id| {
        return if let Some(view) = pass.scene.get_view_mut(id) {
            view.bounds.size.h + a
        } else {
            a
        };
    });
    let vert_leftover = (pass.space - padding).h - kids_sum;

    // layout the flex children
    let flex_kids = pass
        .scene
        .get_children_ids_filtered(&pass.target, |v| v.v_flex == Flex::Grow);
    if flex_kids.len() > 0 {
        let flex_space = Size {
            w: pass.space.w - padding.left - padding.right,
            h: vert_leftover / (flex_kids.len() as i32),
        };
        for kid in flex_kids {
            pass.layout_child(&kid, flex_space);
        }
    }

    // calculate the max width of any child
    let mut max_width = 0;
    for kid in pass.scene.get_children_ids(&pass.target) {
        if let Some(kid) = pass.scene.get_view_mut(&kid) {
            max_width = max_width.max(kid.bounds.size.w);
        }
    }
    if h_flex == Shrink {
        available_space.w = max_width;
    }

    // position all the children
    let mut y = padding.top;
    let all_kids = pass.scene.get_children_ids(&pass.target);
    let avail_w = available_space.w;
    for kid in all_kids {
        if let Some(kid) = pass.scene.get_view_mut(&kid) {
            kid.bounds.position.x = match &kid.h_align {
                Start => 0,
                Center => (avail_w - kid.bounds.size.w) / 2,
                End => (avail_w - kid.bounds.size.w),
            } + padding.left;
            kid.bounds.position.y = y;
            y += kid.bounds.size.h + gap;
        }
    }
    // layout self
    if let Some(view) = pass.scene.get_view_mut(&pass.target) {
        view.bounds.size.w = match &view.h_flex {
            Fixed => view.bounds.size.w,
            Shrink => max_width + padding.left + padding.right,
            Grow => pass.space.w,
        };
        view.bounds.size.h = match &view.v_flex {
            Fixed => view.bounds.size.h,
            Shrink => view.bounds.size.h,
            Grow => pass.space.h,
        };
    }
}

pub fn layout_hbox(pass: &mut LayoutEvent) {
    let Some(parent) = pass.scene.get_view_mut(&pass.target) else {
        return;
    };
    let Some(state) = parent.get_state::<PanelState>() else {
        return;
    };
    let gap = state.gap;
    let padding = state.padding;

    let h_flex = parent.h_flex.clone();
    let v_flex = parent.v_flex.clone();

    // layout self
    if v_flex == Grow {
        parent.bounds.size.h = pass.space.h
    }
    if h_flex == Grow {
        parent.bounds.size.w = pass.space.w
    }

    let mut available_space = pass.space - padding;

    // get the fixed children
    let fixed_kids = pass
        .scene
        .get_children_ids_filtered(&pass.target, |v| v.h_flex == Shrink);

    // layout the fixed width children
    for kid in &fixed_kids {
        pass.layout_child(kid, available_space);
    }

    // calc the total width of the fixed kids
    let kids_sum: i32 = fixed_kids
        .iter()
        .map(|id| pass.scene.get_view(id))
        .flatten()
        .fold(0, |a, v| v.bounds.size.w + a);
    let avail_horizontal_space = (available_space - padding).h - kids_sum;

    // get the flex children
    let flex_kids = pass
        .scene
        .get_children_ids_filtered(&pass.target, |v| v.h_flex == Flex::Grow);
    // if there are any flex children
    if flex_kids.len() > 0 {
        // split the leftover space
        let flex_space = Size {
            w: avail_horizontal_space / (flex_kids.len() as i32),
            h: pass.space.h - padding.top - padding.bottom,
        };
        // layout the flex children
        for kid in &flex_kids {
            pass.layout_child(kid, flex_space);
        }
    }

    // calculate the max height of any child
    let mut max_height = 0;
    for kid in pass.scene.get_children_ids(&pass.target) {
        if let Some(kid) = pass.scene.get_view_mut(&kid) {
            max_height = max_height.max(kid.bounds.size.h);
        }
    }

    // now position all children
    if v_flex == Shrink {
        available_space.h = max_height;
    }
    let avail_h = available_space.h;
    let mut x = padding.left;
    for kid in pass.scene.get_children_ids(&pass.target) {
        if let Some(kid) = pass.scene.get_view_mut(&kid) {
            kid.bounds.position.x = x;
            x += kid.bounds.size.w;
            x += gap;
            kid.bounds.position.y = match &kid.v_align {
                Start => 0,
                Center => (avail_h - kid.bounds.size.h) / 2,
                End => (avail_h - kid.bounds.size.h),
            } + padding.top;
        }
    }
    if let Some(parent) = pass.scene.get_view_mut(pass.target) {
        if parent.v_flex == Shrink {
            parent.bounds.size.h = available_space.h + padding.top + padding.bottom;
        }
        if parent.h_flex == Shrink {
            parent.bounds.size.w = x;
        }
    }
}

pub fn layout_std_panel(pass: &mut LayoutEvent) {
    let Some(view) = pass.scene.get_view_mut(&pass.target) else {
        info!("view not found!");
        return;
    };
    let Some(state) = view.get_state::<PanelState>() else {
        return;
    };
    let padding = state.padding.clone();

    if view.v_flex == Grow {
        view.bounds.size.h = pass.space.h;
    }
    if view.h_flex == Grow {
        view.bounds.size.w = pass.space.w;
    }
    let space = view.bounds.size.clone() - padding;
    pass.layout_all_children(&pass.target.clone(), space);
}

pub fn make_vertical_spacer(h: i32) -> View {
    let spacer = View {
        v_flex: Fixed,
        bounds: Bounds::new(0, 0, 0, h),
        ..Default::default()
    };
    spacer
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::LayoutEvent;
    use crate::geom::{Bounds, Insets, Point, Size};
    use crate::layouts::{layout_std_panel, layout_vbox};
    use crate::panel::PanelState;
    use crate::scene::{Scene, layout_scene};
    use crate::test::MockDrawingContext;
    use crate::view::Align::Start;
    use crate::view::{Align, Flex, View, ViewId};
    use alloc::boxed::Box;
    use test_log::test;

    pub(crate) fn layout_button(layout: &mut LayoutEvent) {
        if let Some(view) = layout.scene.get_view_mut(&layout.target) {
            view.bounds.size = Size::new((view.title.len() * 10) as i32, 10);
        }
    }
    #[test]
    fn test_button() {
        let button = View {
            name: "button1".into(),
            title: "abc".into(),
            layout: Some(layout_button),
            ..Default::default()
        };

        let theme = MockDrawingContext::make_mock_theme();
        let mut scene = Scene::new();
        scene.add_view_to_parent(button, &scene.root_id());
        layout_scene(&mut scene, &theme);
        // size = 3 letters x 10x10 font + 10px padding
        assert_eq!(
            view_bounds(&scene, &"button1".into()).size,
            Size::new(3 * 10, 10),
            "button size is wrong"
        );
    }

    fn view_bounds(scene: &Scene, name: &ViewId) -> Bounds {
        if let Some(view) = scene.get_view(name) {
            view.bounds
        } else {
            Bounds::new(-99, -99, -99, -99)
        }
    }

    #[test]
    fn test_vbox() {
        let mut scene = Scene::new();
        let parent_id: ViewId = "parent".into();
        let parent_view = View {
            name: parent_id.clone(),
            title: "parent".into(),
            state: Some(Box::new(PanelState {
                border_visible: true,
                padding: Insets::new_same(10),
                gap: 0,
            })),
            bounds: Bounds {
                position: Point::new(-99, -99),
                size: Size::new(100, 100),
            },
            h_flex: Flex::Grow,
            v_flex: Flex::Grow,
            h_align: Start,
            v_align: Start,
            layout: Some(layout_vbox),
            ..Default::default()
        };

        let child1_id: ViewId = "child1".into();
        scene.add_view_to_parent(
            View {
                name: child1_id.clone(),
                title: "ch1".into(),
                h_align: Align::Start,
                layout: Some(layout_button),
                ..Default::default()
            },
            &parent_id,
        );

        let child2_id: ViewId = "child2".into();
        scene.add_view_to_parent(
            View {
                name: child2_id.clone(),
                title: "ch2".into(),
                h_align: Align::Center,
                layout: Some(layout_button),
                ..Default::default()
            },
            &parent_id,
        );

        let child3_id: ViewId = "child3".into();
        scene.add_view_to_parent(
            View {
                name: child3_id.clone(),
                title: "ch3".into(),
                h_align: Align::End,
                layout: Some(layout_button),
                ..Default::default()
            },
            &parent_id,
        );

        let child4_id: ViewId = "child4".into();
        scene.add_view_to_parent(
            View {
                name: child4_id.clone(),
                title: "ch4".into(),
                h_flex: Flex::Grow,
                v_flex: Flex::Grow,
                layout: Some(layout_std_panel),
                ..Default::default()
            },
            &parent_id,
        );

        scene.add_view_to_parent(parent_view, &scene.root_id());

        let theme = MockDrawingContext::make_mock_theme();
        layout_scene(&mut scene, &theme);
        scene.dump();
        if let Some(view) = scene.get_view_mut(&parent_id) {
            assert_eq!(view.name, parent_id);
            // confirm position wasn't modified at all
            assert_eq!(view.bounds.position, Point::new(-99, -99));
            // size = scene size of 200x200
            assert_eq!(view.bounds.size, Size::new(200, 200));
            // left align
            if let Some(view) = scene.get_view(&child1_id) {
                assert_eq!(view.bounds.position, Point::new(10, 10));
                assert_eq!(view.bounds.size, Size::new(30, 10));
            }
            // center align
            if let Some(view) = scene.get_view(&child2_id) {
                assert_eq!(view.bounds.position, Point::new(0 + (180 - 10) / 2, 20));
                assert_eq!(view.bounds.size, Size::new(30, 10));
            }
            // right align
            if let Some(view) = scene.get_view(&child3_id) {
                assert_eq!(view.bounds.position, Point::new(10 + (180 - 30), 30));
                assert_eq!(view.bounds.size, Size::new(30, 10));
            }
            // should fill rest of the space
            assert!(scene.has_view(&child4_id));
            if let Some(view) = scene.get_view(&child4_id) {
                assert_eq!(view.bounds.position, Point::new(50, 40));
                assert_eq!(view.bounds.size, Size::new(100, 180 - 80));
            }
        }
    }

    pub fn make_standard_view(name: &ViewId) -> View {
        View {
            name: name.clone(),
            ..Default::default()
        }
    }
}
