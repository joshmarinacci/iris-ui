use crate::geom::{Bounds, Point};
use crate::gfx::DrawingContext;
use crate::input::{InputEvent, InputResult};
use crate::view::{View, ViewId};
use crate::{Callback, DrawEvent, GuiEvent, LayoutEvent, LayoutFn, Theme};
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::{format, vec};
use hashbrown::HashMap;
use log::{info, warn};

/// The top level object of the UI tree
#[derive(Debug)]
pub struct Scene {
    count: u32,
    pub(crate) keys: HashMap<ViewId, View>,
    children: HashMap<ViewId, Vec<ViewId>>,
    parents: HashMap<ViewId, ViewId>,
    pub(crate) dirty: bool,
    pub(crate) bounds: Bounds,
    pub dirty_rect: Bounds,
    root_id: ViewId,
    pub(crate) focused: Option<ViewId>,
    layout_dirty: bool,
    scale: u32,
}

impl Scene {
    /// print a textual representation of the view tree to info. Used for debugging.
    pub fn dump(&self) {
        info!("scene");
        info!(
            " dirty {} {}, focused {:?}",
            self.dirty, self.dirty_rect, self.focused
        );
        self.dump_view(&self.root_id.clone(), "");
    }
    fn dump_view(&self, id: &ViewId, indent: &str) {
        if let Some(view) = self.get_view(&id) {
            info!("{indent}{id} ---");
            // info!("{indent}  padding {}", view.padding);
            info!("{indent}  bounds  {}", view.bounds);
            info!("{indent}  h = {:?} {:?}", view.h_flex, view.h_align);
            info!("{indent}  v = {:?} {:?}", view.v_flex, view.v_align);
        }
        let kids = self.get_children_ids(id);
        for kid in kids {
            self.dump_view(kid, &format!("{indent}    "));
        }
    }
}

impl Scene {
    /// Get the ViewID of the root of the view tree.
    pub fn root_id(&self) -> ViewId {
        self.root_id.clone()
    }
    pub fn next_view_id(&mut self) -> ViewId {
        self.count += 1;
        ViewId::make(format!("view_{}", self.count))
    }
    /// Set the focused view.
    pub fn set_focused(&mut self, name: &ViewId) {
        if let Some(fo) = self.focused.clone() {
            self.mark_dirty_view(&fo);
        }
        self.focused = Some(name.clone());
        self.mark_dirty_view(name);
    }

    /// Get the focused View, if any.
    pub fn get_focused(&self) -> Option<ViewId> {
        self.focused.clone()
    }

    /// Returns if the view is focused or not.
    pub fn is_focused(&self, name: &ViewId) -> bool {
        self.focused.as_ref().is_some_and(|focused| focused == name)
    }

    /// Returns if the view is visible or not.
    pub fn is_visible(&self, name: &ViewId) -> bool {
        if let Some(view) = self.get_view(name) {
            view.visible
        } else {
            false
        }
    }

    /// Makes the view visible and marks scene as dirty.
    pub fn show_view(&mut self, name: &ViewId) {
        if let Some(view) = self.get_view_mut(name) {
            view.visible = true;
        }
        self.mark_dirty_view(name);
    }

    /// Makes the view invisible and marks scene as dirty.
    pub fn hide_view(&mut self, name: &ViewId) {
        if let Some(view) = self.get_view_mut(name) {
            view.visible = false;
        }
        self.mark_dirty_view(name);
    }

    /// Marks the entire scene as dirty.
    pub fn mark_dirty_all(&mut self) {
        self.dirty_rect = self.bounds;
        self.dirty = true;
    }

    /// Marks a specific view as dirty.
    pub fn mark_dirty_view(&mut self, name: &ViewId) {
        if let Some(view) = self.get_view(name) {
            let global_bounds = self.get_view_global_bounds(view);
            self.dirty_rect = self.dirty_rect.union(global_bounds);
            self.dirty = true;
        }
    }

    /// Marks the layout of the scene as dirty.
    pub fn mark_layout_dirty(&mut self) {
        self.layout_dirty = true;
        self.mark_dirty_all();
    }

    /// Get the children of the view.
    pub fn get_children_ids(&self, name: &ViewId) -> &[ViewId] {
        if let Some(children) = self.children.get(name) {
            children.as_slice()
        } else {
            &[]
        }
    }

    /// Get the children of the view filtered by a callback function.
    pub fn get_children_ids_filtered(&self, id: &ViewId, cb: fn(&View) -> bool) -> Vec<ViewId> {
        self.get_children_ids(id)
            .iter()
            .filter_map(|kid| self.get_view(kid))
            .filter(|v| cb(v))
            .map(|v| v.name.clone())
            .collect()
    }

    /// Returns true if the scene contains a specific view.
    pub(crate) fn has_view(&self, name: &ViewId) -> bool {
        self.keys.contains_key(name)
    }

    /// Get the View struct for a ViewID.
    pub fn get_view(&self, name: &ViewId) -> Option<&View> {
        self.keys.get(name)
    }

    /// Mutably get View struct for a ViewID.
    pub fn get_view_mut(&mut self, name: &ViewId) -> Option<&mut View> {
        self.keys.get_mut(name)
    }

    /// Get the state object, if any, for a ViewID.
    pub fn get_view_state<T: 'static>(&mut self, name: &ViewId) -> Option<&mut T> {
        if let Some(view) = self.get_view_mut(name) {
            if let Some(view) = &mut view.state {
                return view.downcast_mut::<T>();
            }
        }
        None
    }
    /// Get the layout function, if any, for a ViewID.
    pub fn get_view_layout(&mut self, name: &ViewId) -> Option<LayoutFn> {
        if let Some(view) = self.get_view_mut(name) {
            return view.layout;
        }
        None
    }

    /// Get the bounds of a ViewID
    pub fn get_view_bounds(&self, name: &ViewId) -> Option<Bounds> {
        if let Some(view) = self.get_view(name) {
            return Some(view.bounds.clone());
        }
        None
    }
    pub(crate) fn viewcount(&self) -> usize {
        self.keys.len()
    }

    /// Remove View from the scene.
    pub fn remove_view(&mut self, name: &ViewId) -> Option<View> {
        self.mark_dirty_view(name);
        self.keys.remove(name)
    }

    /// Get the parent of the View.
    pub fn get_parent_for_view(&self, name: &ViewId) -> Option<&ViewId> {
        self.parents.get(name)
    }

    /// Remove the view from its parent.
    pub fn remove_view_from_parent(&mut self, parent: &ViewId, child: &ViewId) {
        if let Some(children) = self.children.get_mut(parent) {
            if let Some(n) = children.iter().position(|name| name == child) {
                children.remove(n);
            }
        }
        if self.parents.contains_key(child) {
            self.parents.remove(child);
        } else {
            warn!("parent {parent} does not contain child {child}");
        }
    }

    /// Create a new scene with the specified bounds.
    pub fn new_with_bounds(bounds: Bounds) -> Scene {
        let root_id = ViewId::new("root");
        let root = View {
            name: root_id.clone(),
            title: root_id.to_string(),
            bounds,
            visible: true,
            input: None,
            state: None,
            layout: Some(layout_root_panel),
            draw: Some(|e| e.ctx.fill_rect(&e.view.bounds, &e.theme.panel.fill)),
            ..Default::default()
        };
        let mut keys: HashMap<ViewId, View> = HashMap::new();
        keys.insert(root_id.clone(), root);
        Scene {
            bounds,
            keys,
            dirty: true,
            layout_dirty: true,
            root_id,
            focused: None,
            dirty_rect: bounds,
            children: HashMap::new(),
            parents: HashMap::new(),
            count: 0,
            scale: 1,
        }
    }

    /// Create a new scene with the specified bounds and an integer scale factor.
    /// Scale is applied at the rendering boundary — layout and input remain in logical pixels.
    pub fn new_with_scale(bounds: Bounds, scale: u32) -> Scene {
        let mut scene = Self::new_with_bounds(bounds);
        scene.scale = scale;
        scene
    }

    /// Returns the scene's integer scale factor (default 1).
    pub fn scale(&self) -> u32 {
        self.scale
    }

    pub(crate) fn new() -> Scene {
        let bounds = Bounds::new(0, 0, 200, 200);
        Self::new_with_bounds(bounds)
    }

    pub(crate) fn add_view(&mut self, view: View) {
        let name = view.name.clone();
        if self.keys.contains_key(&name) {
            warn!("might be adding duplicate view key {name}");
        }
        self.keys.insert(name.clone(), view);
        self.mark_layout_dirty();
        self.mark_dirty_view(&name);
    }
    /// Add a View to the root of the scene. The scene takes ownership of the View.
    pub fn add_view_to_root(&mut self, view: View) {
        self.add_view_to_parent(view, &self.root_id.clone());
    }
    /// Add a View as a child of a view already in the scene. The scene takes ownership of the View.
    pub fn add_view_to_parent(&mut self, view: View, parent: &ViewId) {
        if !self.children.contains_key(parent) {
            self.children.insert(parent.clone(), vec![]);
        }
        self.parents.insert(view.name.clone(), parent.clone());
        if let Some(children) = self.children.get_mut(parent) {
            children.push(view.name.clone());
        }
        self.add_view(view);
    }
    fn move_view_to_parent(&mut self, child: &ViewId, parent: &ViewId) {
        if !self.children.contains_key(parent) {
            self.children.insert(parent.clone(), vec![]);
        }
        if let Some(children) = self.children.get_mut(parent) {
            children.push(child.clone());
        }
        self.parents.insert(child.clone(), parent.clone());
    }
    /// Remove a view and any children from the scene.
    pub fn remove_parent_and_children(&mut self, name: &ViewId) {
        let kids = self.get_children_ids(name).to_vec();
        for kid in kids {
            self.remove_parent_and_children(&kid);
            self.remove_view_from_parent(name, &kid);
        }
        self.remove_view(name);
    }

    fn get_view_global_bounds(&self, view: &View) -> Bounds {
        let mut current = &view.name;
        let mut offset = Point::zero();
        while let Some(parent) = self.parents.get(current) {
            if let Some(bounds) = self.get_view_bounds(parent) {
                offset = offset + bounds.position;
            }
            current = parent;
        }
        view.bounds + offset
    }
}

impl Scene {
    // resize the scene
    pub fn resize(&mut self, bounds: Bounds) {
        self.bounds = bounds;
        self.mark_layout_dirty();
        self.mark_dirty_all();
    }
}

fn layout_root_panel(pass: &mut LayoutEvent) {
    if let Some(view) = pass.scene.get_view_mut(&pass.target) {
        view.bounds.size.w = pass.space.w;
        view.bounds.size.h = pass.space.h;
    }
    let kids = pass.scene.get_children_ids(&pass.target).to_vec();
    for kid in &kids {
        pass.layout_child(kid, pass.space);
    }
}

/// send a click event to the scene
pub fn click_at(scene: &mut Scene, handlers: &[Callback], pt: Point) -> Option<InputResult> {
    let targets = pick_at(scene, &pt);
    if let Some((target, pt)) = targets.last() {
        let mut event: GuiEvent = GuiEvent {
            scene,
            target,
            event_type: InputEvent::Tap(pt.clone()),
            action: None,
        };
        if let Some(view) = event.scene.get_view(target) {
            if let Some(input) = view.input {
                event.action = input(&mut event);
            }
        }
        for cb in handlers {
            cb(&mut event);
        }
        if let Some(action) = event.action {
            return Some(InputResult {
                source: target.clone(),
                input: event.event_type,
                action: Some(action),
            });
        }
    }
    None
}

/// send a event to the focused element of the scene
pub fn event_at_focused(scene: &mut Scene, event_type: &InputEvent) -> Option<InputResult> {
    if let Some(focused) = scene.focused.clone() {
        let mut event: GuiEvent = GuiEvent {
            scene,
            target: &focused,
            event_type: event_type.clone(),
            action: None,
        };
        if let Some(view) = event.scene.get_view(&focused) {
            if let Some(input) = view.input {
                event.action = input(&mut event);
                return Some(InputResult {
                    source: focused.clone(),
                    input: event.event_type,
                    action: event.action,
                });
            }
        }
    }
    None
}

type Pick = (ViewId, Point);

/// Get a list of views which contain the point.
pub fn pick_at(scene: &mut Scene, pt: &Point) -> Vec<Pick> {
    pick_at_view(scene, pt, &scene.root_id)
}

fn pick_at_view(scene: &Scene, pt: &Point, name: &ViewId) -> Vec<Pick> {
    let mut coll: Vec<Pick> = vec![];
    if let Some(view) = scene.keys.get(name) {
        if view.bounds.contains(pt) && view.visible {
            coll.push((view.name.clone(), pt.clone()));
            let pt2 = pt.subtract(&view.bounds.position);
            for kid in scene.get_children_ids(&view.name) {
                let mut coll2 = pick_at_view(scene, &pt2, kid);
                coll.append(&mut coll2);
            }
        }
    }
    coll
}

/// Draw the scene to the drawing context with the provided theme.
pub fn draw_scene(scene: &mut Scene, ctx: &mut dyn DrawingContext, theme: &Theme) {
    if scene.dirty {
        ctx.fill_rect(&scene.bounds, &theme.standard.fill);
        let name = scene.root_id.clone();
        draw_view(scene, ctx, theme, &name, Point::zero());
        scene.dirty = false;
        scene.dirty_rect = Bounds::new_empty();
    }
}

fn draw_view(
    scene: &mut Scene,
    ctx: &mut dyn DrawingContext,
    theme: &Theme,
    name: &ViewId,
    offset: Point,
) {
    let dirty_rect = scene.dirty_rect;

    // Read visibility and local bounds before any mutable borrow.
    let (visible, local_bounds) = match scene.get_view(name) {
        Some(view) => (view.visible, view.bounds),
        None => return,
    };

    if !visible {
        return;
    }

    // Skip the entire subtree when it lies outside the dirty region.
    if !dirty_rect.is_empty() && !(local_bounds + offset).intersects(&dirty_rect) {
        return;
    }

    // Draw this view.
    let focused = scene.focused.clone();
    let scene_bounds = scene.bounds;
    if let Some(view) = scene.get_view_mut(name) {
        if let Some(draw) = view.draw {
            let mut de: DrawEvent = DrawEvent {
                theme,
                view,
                ctx,
                focused: &focused,
                bounds: &scene_bounds,
            };
            draw(&mut de);
        }
    }

    // Draw children, accumulating the coordinate offset for dirty-rect checks.
    let child_offset = offset + local_bounds.position;
    ctx.translate(&local_bounds.position);
    let kids: Vec<ViewId> = scene.get_children_ids(name).to_vec();
    for kid in &kids {
        draw_view(scene, ctx, theme, kid, child_offset);
    }
    ctx.translate(&local_bounds.position.negate());
}

/// Layout the scene with the provided theme
pub fn layout_scene(scene: &mut Scene, theme: &Theme) {
    if scene.layout_dirty {
        let mut pass = LayoutEvent {
            target: &scene.root_id(),
            space: scene.bounds.size,
            scene,
            theme,
        };
        if let Some(layout) = pass.scene.get_view_layout(&pass.scene.root_id()) {
            layout(&mut pass);
        }
        scene.layout_dirty = false;
    }
}

#[cfg(test)]
#[cfg(any(feature = "std", feature = "headless"))]
mod tests {
    use crate::geom::Bounds;
    use crate::scene::Scene;
    use crate::view::ViewId;

    #[test]
    fn remove_parent_and_children_cleans_grandchildren() {
        let mut scene: Scene = Scene::new();
        let parent_id: ViewId = "parent".into();
        let child_id: ViewId = "child".into();
        let grandchild_id: ViewId = "grandchild".into();

        let parent_view = crate::tests::make_simple_view(&parent_id);
        scene.add_view_to_parent(parent_view, &scene.root_id());
        let child_view = crate::tests::make_simple_view(&child_id);
        scene.add_view_to_parent(child_view, &parent_id);
        let grandchild_view = crate::tests::make_simple_view(&grandchild_id);
        scene.add_view_to_parent(grandchild_view, &child_id);

        assert_eq!(scene.viewcount(), 4); // root + parent + child + grandchild

        scene.remove_parent_and_children(&parent_id);

        assert_eq!(scene.viewcount(), 1); // only root remains
        assert!(scene.get_view(&parent_id).is_none());
        assert!(scene.get_view(&child_id).is_none());
        assert!(scene.get_view(&grandchild_id).is_none());
        assert!(scene.get_parent_for_view(&grandchild_id).is_none());
        assert!(scene.get_parent_for_view(&child_id).is_none());
    }

    #[test]
    fn basic_add_remove() {
        let mut scene: Scene = Scene::new_with_bounds(Bounds::new(0, 0, 100, 30));
        assert_eq!(scene.viewcount(), 1);
        let view = crate::tests::make_simple_view(&"foo".into());
        assert_eq!(scene.viewcount(), 1);
        scene.add_view(view);
        assert_eq!(scene.viewcount(), 2);
        assert!(scene.get_view(&"foo".into()).is_some());
        let res = scene.remove_view(&"foo".into());
        assert_eq!(res.is_some(), true);
        assert_eq!(scene.viewcount(), 1);
        let res2 = scene.remove_view(&"bar".into());
        assert_eq!(res2.is_some(), false);
    }
    #[test]
    fn parent_child() {
        let mut scene: Scene = Scene::new();
        let parent_id: ViewId = "parent".into();
        let child_id: ViewId = "child".into();
        let parent_view = crate::tests::make_simple_view(&parent_id);
        scene.add_view(parent_view);

        let child_view = crate::tests::make_simple_view(&child_id);
        assert_eq!(scene.get_children_ids(&parent_id).len(), 0);
        assert_eq!(scene.viewcount(), 2);
        scene.add_view_to_parent(child_view, &parent_id);
        assert_eq!(scene.get_children_ids(&parent_id).len(), 1);
        assert_eq!(scene.get_parent_for_view(&child_id).unwrap(), &parent_id);
        scene.remove_view_from_parent(&parent_id, &child_id);
        assert_eq!(scene.get_children_ids(&parent_id).len(), 0);
        assert!(scene.get_parent_for_view(&child_id).is_none());

        scene.move_view_to_parent(&child_id, &parent_id);
        assert_eq!(scene.get_children_ids(&parent_id).len(), 1);
        assert_eq!(scene.get_parent_for_view(&child_id), Some(&parent_id));
        let child2 = crate::tests::make_simple_view(&"child2".into());
        scene.add_view_to_parent(child2, &parent_id);
        assert_eq!(scene.get_children_ids(&parent_id).len(), 2);
        assert_eq!(scene.viewcount(), 4);

        scene.remove_parent_and_children(&parent_id);
        assert_eq!(scene.get_children_ids(&parent_id).len(), 0);
        assert_eq!(scene.viewcount(), 1);
    }
}

#[cfg(test)]
#[cfg(any(feature = "std", feature = "headless"))]
mod dirty_rect_tests {
    use crate::geom::Bounds;
    use crate::scene::{Scene, draw_scene};
    use crate::test::MockDrawingContext;
    use crate::view::{View, ViewId};
    use crate::DrawEvent;
    use alloc::boxed::Box;

    struct DrawCounter {
        count: i32,
    }

    fn counting_draw(e: &mut DrawEvent) {
        if let Some(state) = e.view.get_state::<DrawCounter>() {
            state.count += 1;
        }
    }

    #[test]
    fn test_dirty_rect_culls_non_overlapping_view() {
        let theme = MockDrawingContext::make_mock_theme();
        let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 200, 200));

        let a_id = ViewId::new("a");
        scene.add_view_to_root(View {
            name: a_id.clone(),
            bounds: Bounds::new(0, 0, 100, 200),
            draw: Some(counting_draw),
            state: Some(Box::new(DrawCounter { count: 0 })),
            visible: true,
            ..Default::default()
        });

        let b_id = ViewId::new("b");
        scene.add_view_to_root(View {
            name: b_id.clone(),
            bounds: Bounds::new(100, 0, 100, 200),
            draw: Some(counting_draw),
            state: Some(Box::new(DrawCounter { count: 0 })),
            visible: true,
            ..Default::default()
        });

        // Mark only the left half as dirty — view B should be skipped.
        scene.dirty_rect = Bounds::new(0, 0, 100, 200);
        scene.dirty = true;

        let mut ctx = MockDrawingContext::new(&scene);
        draw_scene(&mut scene, &mut ctx, &theme);

        assert_eq!(
            scene.get_view_state::<DrawCounter>(&a_id).unwrap().count,
            1,
            "view A overlaps dirty_rect and must be drawn"
        );
        assert_eq!(
            scene.get_view_state::<DrawCounter>(&b_id).unwrap().count,
            0,
            "view B is outside dirty_rect and must be skipped"
        );
    }

    #[test]
    fn test_empty_dirty_rect_draws_all_views() {
        let theme = MockDrawingContext::make_mock_theme();
        let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 200, 200));

        let a_id = ViewId::new("a");
        scene.add_view_to_root(View {
            name: a_id.clone(),
            bounds: Bounds::new(50, 50, 100, 100),
            draw: Some(counting_draw),
            state: Some(Box::new(DrawCounter { count: 0 })),
            visible: true,
            ..Default::default()
        });

        // Empty dirty_rect means "no partial-dirty region — redraw everything".
        scene.dirty_rect = Bounds::new_empty();
        scene.dirty = true;

        let mut ctx = MockDrawingContext::new(&scene);
        draw_scene(&mut scene, &mut ctx, &theme);

        assert_eq!(
            scene.get_view_state::<DrawCounter>(&a_id).unwrap().count,
            1,
            "with empty dirty_rect all views must be drawn"
        );
    }
}
