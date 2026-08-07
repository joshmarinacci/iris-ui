use crate::LayoutEvent;
use crate::geom::Size;
use crate::panel::PanelState;
use crate::view::Align::{Center, End, Start};
use crate::view::Flex;
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
    let padding = panel_state.padding;
    let h_flex = parent.h_flex;
    let v_flex = parent.v_flex;
    let parent_w = parent.bounds.size.w;
    let parent_h = parent.bounds.size.h;
    let mut available_space: Size = pass.space - padding;
    if h_flex == Fixed {
        available_space.w = parent_w - padding.left - padding.right;
    }
    if v_flex == Fixed {
        available_space.h = parent_h - padding.top - padding.bottom;
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
        if let Some(view) = pass.scene.get_view_mut(id) {
            view.bounds.size.h + a
        } else {
            a
        }
    });
    let total_children = pass.scene.get_children_ids(&pass.target).len() as i32;
    let total_gap = gap * (total_children - 1).max(0);
    let vert_leftover = available_space.h - kids_sum - total_gap;

    // layout the flex children
    let flex_kids = pass
        .scene
        .get_children_ids_filtered(&pass.target, |v| v.v_flex == Flex::Grow);
    if flex_kids.len() > 0 {
        let flex_space = Size {
            w: available_space.w,
            h: vert_leftover / (flex_kids.len() as i32),
        };
        for kid in flex_kids {
            pass.layout_child(&kid, flex_space);
        }
    }

    // calculate the max width of any child
    let mut max_width = 0;
    let all_kids = pass.scene.get_children_ids(&pass.target).to_vec();
    for kid in &all_kids {
        if let Some(kid) = pass.scene.get_view_mut(kid) {
            max_width = max_width.max(kid.bounds.size.w);
        }
    }
    if h_flex == Shrink {
        available_space.w = max_width;
    }

    // position all the children
    let mut y = padding.top;
    let avail_w = available_space.w;
    for kid in &all_kids {
        if let Some(kid) = pass.scene.get_view_mut(kid) {
            kid.bounds.position.x = match &kid.h_align {
                Start => 0,
                Center => (avail_w - kid.bounds.size.w) / 2,
                End => avail_w - kid.bounds.size.w,
            } + padding.left;
            kid.bounds.position.y = y;
            y += kid.bounds.size.h + gap;
        }
    }
    // content_h: sum of all children heights + gaps between them (strip trailing gap)
    let content_h = if all_kids.is_empty() { 0 } else { y - gap };
    // layout self
    if let Some(view) = pass.scene.get_view_mut(&pass.target) {
        view.bounds.size.w = match &view.h_flex {
            Fixed => view.bounds.size.w,
            Shrink => max_width + padding.left + padding.right,
            Grow => pass.space.w,
        };
        view.bounds.size.h = match &view.v_flex {
            Fixed => view.bounds.size.h,
            Shrink => content_h + padding.bottom,
            Grow => pass.space.h,
        };
    }
}

pub fn layout_centered_dialog(pass: &mut LayoutEvent) {
    let target_id = pass.target.clone();
    let available = pass.space;
    layout_vbox(pass);
    if let Some(view) = pass.scene.get_view_mut(&target_id) {
        let dialog_w = view.bounds.size.w;
        let dialog_h = view.bounds.size.h;
        view.bounds.position.x = ((available.w - dialog_w) / 2).max(0);
        view.bounds.position.y = ((available.h - dialog_h) / 2).max(0);
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

    let h_flex = parent.h_flex;
    let v_flex = parent.v_flex;

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
    let total_children = pass.scene.get_children_ids(&pass.target).len() as i32;
    let total_gap = gap * (total_children - 1).max(0);
    let avail_horizontal_space = available_space.w - kids_sum - total_gap;

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
    let all_kids = pass.scene.get_children_ids(&pass.target).to_vec();
    for kid in &all_kids {
        if let Some(kid) = pass.scene.get_view_mut(kid) {
            max_height = max_height.max(kid.bounds.size.h);
        }
    }

    // now position all children
    if v_flex == Shrink {
        available_space.h = max_height;
    }
    let avail_h = available_space.h;
    let mut x = padding.left;
    for kid in &all_kids {
        if let Some(kid) = pass.scene.get_view_mut(kid) {
            kid.bounds.position.x = x;
            x += kid.bounds.size.w;
            x += gap;
            kid.bounds.position.y = match &kid.v_align {
                Start => 0,
                Center => (avail_h - kid.bounds.size.h) / 2,
                End => avail_h - kid.bounds.size.h,
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
    let padding = state.padding;

    if view.v_flex == Grow {
        view.bounds.size.h = pass.space.h;
    }
    if view.h_flex == Grow {
        view.bounds.size.w = pass.space.w;
    }
    let space = view.bounds.size - padding;
    pass.layout_all_children(&pass.target.clone(), space);
}

#[cfg(test)]
#[cfg(any(feature = "std", feature = "headless"))]
pub(crate) mod tests {
    use crate::LayoutEvent;
    use crate::geom::{Bounds, Insets, Point, Size};
    use crate::layouts::{layout_hbox, layout_std_panel, layout_vbox};
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

    pub(crate) fn layout_fill(layout: &mut LayoutEvent) {
        if let Some(view) = layout.scene.get_view_mut(&layout.target) {
            if view.h_flex == Flex::Grow {
                view.bounds.size.w = layout.space.w;
            }
            if view.v_flex == Flex::Grow {
                view.bounds.size.h = layout.space.h;
            }
        }
    }

    #[test]
    fn test_hbox_fixed_width() {
        let mut scene = Scene::new();
        let parent_id: ViewId = "parent".into();
        // fixed 200 px wide parent
        let parent_view = View {
            name: parent_id.clone(),
            state: Some(Box::new(PanelState {
                border_visible: false,
                padding: Insets::new_same(10),
                gap: 0,
            })),
            bounds: Bounds {
                position: Point::new(0, 0),
                size: Size::new(200, 60),
            },
            h_flex: Flex::Fixed,
            v_flex: Flex::Grow,
            layout: Some(layout_hbox),
            ..Default::default()
        };

        let child1_id: ViewId = "child1".into();
        scene.add_view_to_parent(
            View {
                name: child1_id.clone(),
                title: "abc".into(),
                v_align: Start,
                layout: Some(layout_button),
                ..Default::default()
            },
            &parent_id,
        );

        // middle child grows to fill remaining horizontal space
        let child2_id: ViewId = "child2".into();
        scene.add_view_to_parent(
            View {
                name: child2_id.clone(),
                v_align: Start,
                h_flex: Flex::Grow,
                v_flex: Flex::Grow,
                layout: Some(layout_fill),
                ..Default::default()
            },
            &parent_id,
        );

        let child3_id: ViewId = "child3".into();
        scene.add_view_to_parent(
            View {
                name: child3_id.clone(),
                title: "abc".into(),
                v_align: Start,
                layout: Some(layout_button),
                ..Default::default()
            },
            &parent_id,
        );

        scene.add_view_to_parent(parent_view, &scene.root_id());

        let theme = MockDrawingContext::make_mock_theme();
        layout_scene(&mut scene, &theme);

        // panel keeps its fixed width of 200; height grows to fill scene
        assert_eq!(view_bounds(&scene, &parent_id).size, Size::new(200, 200));
        // child1: left fixed child, inset by padding
        assert_eq!(view_bounds(&scene, &child1_id).position, Point::new(10, 10));
        assert_eq!(view_bounds(&scene, &child1_id).size, Size::new(30, 10));
        // child2: fills (200 - 2*10 padding - child1.w - child3.w) = 120 horizontally
        assert_eq!(view_bounds(&scene, &child2_id).position, Point::new(40, 10));
        assert_eq!(view_bounds(&scene, &child2_id).size, Size::new(120, 180));
        // child3: right fixed child, placed after child2
        assert_eq!(
            view_bounds(&scene, &child3_id).position,
            Point::new(160, 10)
        );
        assert_eq!(view_bounds(&scene, &child3_id).size, Size::new(30, 10));
    }

    #[test]
    fn test_vbox_gap() {
        // gap > 0 pushes each child down by (previous_child_height + gap).
        let mut scene = Scene::new();
        let parent_id: ViewId = "parent".into();
        let parent_view = View {
            name: parent_id.clone(),
            state: Some(Box::new(PanelState {
                border_visible: false,
                padding: Insets::new_same(10),
                gap: 5,
            })),
            h_flex: Flex::Grow,
            v_flex: Flex::Grow,
            layout: Some(layout_vbox),
            ..Default::default()
        };
        let child1_id: ViewId = "child1".into();
        scene.add_view_to_parent(
            View {
                name: child1_id.clone(),
                title: "a".into(), // 10px wide, 10px tall
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
                title: "a".into(),
                h_align: Align::Start,
                layout: Some(layout_button),
                ..Default::default()
            },
            &parent_id,
        );
        scene.add_view_to_parent(parent_view, &scene.root_id());

        let theme = MockDrawingContext::make_mock_theme();
        layout_scene(&mut scene, &theme);

        // child1 is placed at top-left padding corner
        assert_eq!(
            view_bounds(&scene, &child1_id).position,
            Point::new(10, 10),
            "child1 must be at (padding.left, padding.top)"
        );
        // child2 is offset by child1.height + gap below child1
        assert_eq!(
            view_bounds(&scene, &child2_id).position,
            Point::new(10, 25), // 10 + 10 + 5
            "child2 must be at (padding.left, padding.top + child1.h + gap)"
        );
    }

    #[test]
    fn test_hbox_gap() {
        // gap > 0 pushes each child right by (previous_child_width + gap).
        let mut scene = Scene::new();
        let parent_id: ViewId = "parent".into();
        let parent_view = View {
            name: parent_id.clone(),
            state: Some(Box::new(PanelState {
                border_visible: false,
                padding: Insets::new_same(8),
                gap: 6,
            })),
            h_flex: Flex::Grow,
            v_flex: Flex::Grow,
            layout: Some(layout_hbox),
            ..Default::default()
        };
        let child1_id: ViewId = "child1".into();
        scene.add_view_to_parent(
            View {
                name: child1_id.clone(),
                title: "a".into(), // 10px wide, 10px tall
                v_align: Align::Start,
                layout: Some(layout_button),
                ..Default::default()
            },
            &parent_id,
        );
        let child2_id: ViewId = "child2".into();
        scene.add_view_to_parent(
            View {
                name: child2_id.clone(),
                title: "a".into(),
                v_align: Align::Start,
                layout: Some(layout_button),
                ..Default::default()
            },
            &parent_id,
        );
        scene.add_view_to_parent(parent_view, &scene.root_id());

        let theme = MockDrawingContext::make_mock_theme();
        layout_scene(&mut scene, &theme);

        // child1 is placed at the left/top padding corner
        assert_eq!(
            view_bounds(&scene, &child1_id).position,
            Point::new(8, 8),
            "child1 must be at (padding.left, padding.top)"
        );
        // child2 is offset right by child1.width + gap
        assert_eq!(
            view_bounds(&scene, &child2_id).position,
            Point::new(24, 8), // 8 + 10 + 6
            "child2 must be at (padding.left + child1.w + gap, padding.top)"
        );
    }

    #[test]
    fn test_vbox_shrinks_width_to_widest_child() {
        // h_flex=Shrink: parent width collapses to max_child_width + 2*padding.
        let mut scene = Scene::new();
        let parent_id: ViewId = "parent".into();
        let parent_view = View {
            name: parent_id.clone(),
            state: Some(Box::new(PanelState {
                border_visible: false,
                padding: Insets::new_same(5),
                gap: 0,
            })),
            h_flex: Flex::Shrink,
            v_flex: Flex::Grow,
            layout: Some(layout_vbox),
            ..Default::default()
        };
        // wide child: "abc" = 30px
        let wide_id: ViewId = "wide".into();
        scene.add_view_to_parent(
            View {
                name: wide_id.clone(),
                title: "abc".into(),
                h_align: Align::Start,
                layout: Some(layout_button),
                ..Default::default()
            },
            &parent_id,
        );
        // narrow child: "a" = 10px
        let narrow_id: ViewId = "narrow".into();
        scene.add_view_to_parent(
            View {
                name: narrow_id.clone(),
                title: "a".into(),
                h_align: Align::Start,
                layout: Some(layout_button),
                ..Default::default()
            },
            &parent_id,
        );
        scene.add_view_to_parent(parent_view, &scene.root_id());

        let theme = MockDrawingContext::make_mock_theme();
        layout_scene(&mut scene, &theme);

        // Parent width = max(30, 10) + 2*5 = 40
        assert_eq!(
            view_bounds(&scene, &parent_id).size.w,
            40,
            "parent width must shrink to widest child + 2*padding"
        );
        assert_eq!(
            view_bounds(&scene, &wide_id).position,
            Point::new(5, 5),
            "wide child must be at (padding.left, padding.top)"
        );
        assert_eq!(
            view_bounds(&scene, &narrow_id).position,
            Point::new(5, 15), // 5 + 10 + 0
            "narrow child must be at (padding.left, padding.top + wide.h)"
        );
    }

    #[test]
    fn test_hbox_shrinks_height_to_tallest_child() {
        // v_flex=Shrink: parent height collapses to max_child_height + 2*padding.
        let mut scene = Scene::new();
        let parent_id: ViewId = "parent".into();
        let parent_view = View {
            name: parent_id.clone(),
            state: Some(Box::new(PanelState {
                border_visible: false,
                padding: Insets::new_same(6),
                gap: 0,
            })),
            h_flex: Flex::Grow,
            v_flex: Flex::Shrink,
            layout: Some(layout_hbox),
            ..Default::default()
        };
        let child1_id: ViewId = "child1".into();
        scene.add_view_to_parent(
            View {
                name: child1_id.clone(),
                title: "a".into(), // 10px wide, 10px tall
                v_align: Align::Start,
                layout: Some(layout_button),
                ..Default::default()
            },
            &parent_id,
        );
        let child2_id: ViewId = "child2".into();
        scene.add_view_to_parent(
            View {
                name: child2_id.clone(),
                title: "a".into(),
                v_align: Align::Start,
                layout: Some(layout_button),
                ..Default::default()
            },
            &parent_id,
        );
        scene.add_view_to_parent(parent_view, &scene.root_id());

        let theme = MockDrawingContext::make_mock_theme();
        layout_scene(&mut scene, &theme);

        // Parent height = max(10, 10) + 2*6 = 22
        assert_eq!(
            view_bounds(&scene, &parent_id).size.h,
            22,
            "parent height must shrink to tallest child + 2*padding"
        );
        assert_eq!(
            view_bounds(&scene, &parent_id).size.w,
            200,
            "parent width must grow to fill scene"
        );
        assert_eq!(
            view_bounds(&scene, &child1_id).position,
            Point::new(6, 6),
            "child1 must be at (padding.left, padding.top)"
        );
        assert_eq!(
            view_bounds(&scene, &child2_id).position,
            Point::new(16, 6), // 6 + 10
            "child2 must be at (padding.left + child1.w, padding.top)"
        );
    }

    #[test]
    fn test_hbox_grow_child_accounts_for_gap() {
        // A Grow child must receive (available_w - shrink_kids_total - gap*(n-1)) not
        // (available_w - shrink_kids_total), otherwise it overflows the container.
        let mut scene = Scene::new(); // 200x200
        let parent_id: ViewId = "parent".into();
        let parent_view = View {
            name: parent_id.clone(),
            state: Some(Box::new(PanelState {
                border_visible: false,
                padding: Insets::new_same(0),
                gap: 10,
            })),
            h_flex: Flex::Grow,
            v_flex: Flex::Grow,
            layout: Some(layout_hbox),
            ..Default::default()
        };
        let fixed_id: ViewId = "fixed".into();
        scene.add_view_to_parent(
            View {
                name: fixed_id.clone(),
                title: "ab".into(), // 20px wide
                layout: Some(layout_button),
                ..Default::default()
            },
            &parent_id,
        );
        let grow_id: ViewId = "grow".into();
        scene.add_view_to_parent(
            View {
                name: grow_id.clone(),
                h_flex: Flex::Grow,
                v_flex: Flex::Grow,
                layout: Some(layout_fill),
                ..Default::default()
            },
            &parent_id,
        );
        scene.add_view_to_parent(parent_view, &scene.root_id());

        let theme = MockDrawingContext::make_mock_theme();
        layout_scene(&mut scene, &theme);

        // fixed(20) + gap(10) + grow = 200  →  grow = 170
        assert_eq!(
            view_bounds(&scene, &grow_id).size.w,
            170,
            "grow child width must not include the gap (was 180 before fix)"
        );
        let right_edge =
            view_bounds(&scene, &grow_id).position.x + view_bounds(&scene, &grow_id).size.w;
        assert_eq!(right_edge, 200, "grow child must not overflow the container");
    }

    #[test]
    fn test_vbox_grow_child_accounts_for_gap() {
        // Same bug in the vertical direction: vert_leftover must subtract gap*(n-1).
        let mut scene = Scene::new(); // 200x200
        let parent_id: ViewId = "parent".into();
        let parent_view = View {
            name: parent_id.clone(),
            state: Some(Box::new(PanelState {
                border_visible: false,
                padding: Insets::new_same(0),
                gap: 10,
            })),
            h_flex: Flex::Grow,
            v_flex: Flex::Grow,
            layout: Some(layout_vbox),
            ..Default::default()
        };
        let fixed_id: ViewId = "fixed".into();
        scene.add_view_to_parent(
            View {
                name: fixed_id.clone(),
                title: "a".into(), // 10px tall
                h_align: Align::Start,
                layout: Some(layout_button),
                ..Default::default()
            },
            &parent_id,
        );
        let grow_id: ViewId = "grow".into();
        scene.add_view_to_parent(
            View {
                name: grow_id.clone(),
                h_align: Align::Start,
                h_flex: Flex::Grow,
                v_flex: Flex::Grow,
                layout: Some(layout_fill),
                ..Default::default()
            },
            &parent_id,
        );
        scene.add_view_to_parent(parent_view, &scene.root_id());

        let theme = MockDrawingContext::make_mock_theme();
        layout_scene(&mut scene, &theme);

        // fixed(10) + gap(10) + grow = 200  →  grow = 180
        assert_eq!(
            view_bounds(&scene, &grow_id).size.h,
            180,
            "grow child height must not include the gap (was 190 before fix)"
        );
        let bottom_edge =
            view_bounds(&scene, &grow_id).position.y + view_bounds(&scene, &grow_id).size.h;
        assert_eq!(bottom_edge, 200, "grow child must not overflow the container");
    }

    #[test]
    fn test_vbox_fixed_height() {
        let mut scene = Scene::new();
        let parent_id: ViewId = "parent".into();
        // fixed 120 px height parent
        let parent_view = View {
            name: parent_id.clone(),
            state: Some(Box::new(PanelState {
                border_visible: false,
                padding: Insets::new_same(10),
                gap: 0,
            })),
            bounds: Bounds {
                position: Point::new(0, 0),
                size: Size::new(200, 120),
            },
            h_flex: Flex::Grow,
            v_flex: Flex::Fixed,
            layout: Some(layout_vbox),
            ..Default::default()
        };

        let child1_id: ViewId = "child1".into();
        scene.add_view_to_parent(
            View {
                name: child1_id.clone(),
                title: "abc".into(),
                h_align: Start,
                layout: Some(layout_button),
                ..Default::default()
            },
            &parent_id,
        );

        // middle child grows to fill remaining vertical space
        let child2_id: ViewId = "child2".into();
        scene.add_view_to_parent(
            View {
                name: child2_id.clone(),
                h_align: Start,
                h_flex: Flex::Grow,
                v_flex: Flex::Grow,
                layout: Some(layout_fill),
                ..Default::default()
            },
            &parent_id,
        );

        let child3_id: ViewId = "child3".into();
        scene.add_view_to_parent(
            View {
                name: child3_id.clone(),
                title: "abc".into(),
                h_align: Start,
                layout: Some(layout_button),
                ..Default::default()
            },
            &parent_id,
        );

        scene.add_view_to_parent(parent_view, &scene.root_id());

        let theme = MockDrawingContext::make_mock_theme();
        layout_scene(&mut scene, &theme);

        // panel keeps its fixed height of 120; width grows to fill scene
        assert_eq!(view_bounds(&scene, &parent_id).size, Size::new(200, 120));
        // child1: top fixed child, inset by padding
        assert_eq!(view_bounds(&scene, &child1_id).position, Point::new(10, 10));
        assert_eq!(view_bounds(&scene, &child1_id).size, Size::new(30, 10));
        // child2: fills (120 - 2*padding - child1.h - child3.h) = 80 vertically
        assert_eq!(view_bounds(&scene, &child2_id).position, Point::new(10, 20));
        assert_eq!(view_bounds(&scene, &child2_id).size, Size::new(180, 80));
        // child3: bottom fixed child, pushed below child2
        assert_eq!(
            view_bounds(&scene, &child3_id).position,
            Point::new(10, 100)
        );
        assert_eq!(view_bounds(&scene, &child3_id).size, Size::new(30, 10));
    }
}
