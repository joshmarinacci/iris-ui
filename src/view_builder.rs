use crate::geom::Bounds;
use crate::scene::Scene;
use crate::view::{Align, Flex, View, ViewId};
use crate::{DrawFn, LayoutFn};
use alloc::boxed::Box;

pub struct ViewBuilder<'a, S> {
    scene: &'a mut Scene,
    view: View,
    pub state: Box<S>,
}

impl<'a, S: 'static> ViewBuilder<'a, S> {
    pub fn build_with_state(scene: &'a mut Scene, state: S) -> ViewBuilder<'a, S> {
        let view = scene.make_view();
        ViewBuilder {
            scene,
            view,
            state: Box::new(state),
        }
    }
    pub fn build_with<F: FnMut(&mut View)>(scene: &'a mut Scene, cb: F, state: S) -> ViewBuilder<'a, S> {
        let view = scene.make_view();
        ViewBuilder { scene, view, state: Box::new(state) }.with(cb)
    }

    pub fn with_bounds(mut self, bounds: Bounds) -> ViewBuilder<'a, S> {
        self.view.bounds = bounds;
        self
    }
    pub fn with<F: FnMut(&mut View)>(mut self, mut cb: F) -> ViewBuilder<'a, S> {
        cb(&mut self.view);
        self
    }
    pub fn with_h_align(mut self, align: Align) -> ViewBuilder<'a, S> {
        self.view.h_align = align;
        self
    }
    pub fn with_h_flex(mut self, h_flex: Flex) -> ViewBuilder<'a, S> {
        self.view.h_flex = h_flex;
        self
    }

    pub fn mut_state<F: FnMut(&mut S)>(mut self, mut cb: F) -> ViewBuilder<'a, S> {
        cb(&mut self.state);
        self
    }
    pub fn with_title(mut self, title: &'a str) -> ViewBuilder<'a, S> {
        self.view.title = title.into();
        self
    }
    pub fn with_position(mut self, x: i32, y: i32) -> ViewBuilder<'a, S> {
        self.view.bounds.position.x = x;
        self.view.bounds.position.y = y;
        self
    }
    pub fn with_layout(mut self, layout: LayoutFn) -> ViewBuilder<'a, S> {
        self.view.layout = Some(layout);
        self
    }
    pub fn with_draw(mut self, draw: DrawFn) -> ViewBuilder<'a, S> {
        self.view.draw = Some(draw);
        self
    }


    pub fn add_to_parent(mut self, parent: &ViewId) -> ViewId {
        self.view.state = Some(self.state);
        let name = self.view.name.clone();
        self.scene.add_view_to_parent(self.view, parent);
        name
    }
    pub fn add_to_root(mut self) -> ViewId {
        self.view.state = Some(self.state);
        let name = self.view.name.clone();
        self.scene.add_view_to_root(self.view);
        name
    }
}
