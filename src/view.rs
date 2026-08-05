use crate::geom::Bounds;
use crate::{DrawFn, InputFn, LayoutFn};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::any::Any;
use core::fmt::{Display, Formatter};

/// The ID of a View. Should be unique for the lifetime of the application.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ViewId(Cow<'static, str>);

impl ViewId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ViewId {
    pub const fn new(id: &'static str) -> Self {
        ViewId(Cow::Borrowed(id))
    }
    pub fn make(id: String) -> Self {
        ViewId(Cow::Owned(id))
    }
}
impl Display for ViewId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<&'static str> for ViewId {
    fn from(s: &'static str) -> Self {
        ViewId::new(s)
    }
}

/// Indicates if the view grow, shrink, or maintain a fixed size in the specified direction.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Flex {
    Shrink,
    Grow,
    Fixed,
}

/// Indicates if the view be aligned to the start, center, or end of the specified direction.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Align {
    Start,
    Center,
    End,
}

/// The primary UI component struct
#[derive(Debug)]
pub struct View {
    /// The unique name of the view.
    pub name: ViewId,

    /// The title of the view. Often used for the title of a button or label.
    pub title: String,

    /// The current bounds of the view.
    pub bounds: Bounds,

    /// Indicates if the view should grow, shrink, or have a fixed size along the vertical axis.
    pub v_flex: Flex,

    /// Indicates if the view should grow, shrink, or have a fixed size along the horizontal axis.
    pub h_flex: Flex,

    /// Indicates how the view should be aligned along horizontal axis.
    pub h_align: Align,

    /// Indicates how the view should be aligned along vertical axis.
    pub v_align: Align,

    /// Indicates if the view is visible or not.
    pub visible: bool,

    /// an optional object representing the state of the view
    pub state: Option<Box<dyn Any>>,

    /// an optional function to handle input events for this View
    pub input: Option<InputFn>,

    /// an optional function to perform layout for this View
    pub layout: Option<LayoutFn>,

    /// an optional function to draw this View
    pub draw: Option<DrawFn>,
}

impl View {
    pub fn with_name(mut self, name: ViewId) -> View {
        self.name = name;
        self
    }
    pub fn with_title(mut self, title: String) -> View {
        self.title = title;
        self
    }
    pub fn with_bounds(mut self, bounds: Bounds) -> View {
        self.bounds = bounds;
        self
    }
    pub fn with_state(mut self, state: Option<Box<dyn Any>>) -> View {
        self.state = state;
        self
    }
    pub fn with_input(mut self, input: Option<InputFn>) -> View {
        self.input = input;
        self
    }
    pub fn with_layout(mut self, layout: Option<LayoutFn>) -> View {
        self.layout = layout;
        self
    }
    pub fn with_draw(mut self, draw: Option<DrawFn>) -> View {
        self.draw = draw;
        self
    }
    pub fn with_flex(mut self, h_flex: Flex, v_flex: Flex) -> View {
        self.h_flex = h_flex;
        self.v_flex = v_flex;
        self
    }
    pub fn with_h_flex(mut self, flex: Flex) -> View {
        self.h_flex = flex;
        self
    }
    pub fn with_v_flex(mut self, flex: Flex) -> View {
        self.v_flex = flex;
        self
    }
    pub fn with_h_align(mut self, align: Align) -> View {
        self.h_align = align;
        self
    }
    pub fn with_v_align(mut self, align: Align) -> View {
        self.v_align = align;
        self
    }
    pub fn position_at(mut self, x: i32, y: i32) -> View {
        self.bounds.position.x = x;
        self.bounds.position.y = y;
        self
    }
    pub fn with_size(mut self, w: i32, h: i32) -> View {
        self.bounds.size.w = w;
        self.bounds.size.h = h;
        self
    }
    pub fn with_visible(mut self, visible: bool) -> View {
        self.visible = visible;
        self
    }
}

impl View {
    pub fn get_state<T: 'static>(&mut self) -> Option<&mut T> {
        if let Some(view) = &mut self.state {
            return view.downcast_mut::<T>();
        }
        None
    }
}

impl Default for View {
    fn default() -> Self {
        let id: ViewId = ViewId::new("noname");
        View {
            name: id.clone(),
            title: id.to_string(),
            bounds: Default::default(),

            h_flex: Flex::Shrink,
            v_flex: Flex::Shrink,
            h_align: Align::Center,
            v_align: Align::Center,

            visible: true,
            input: None,
            state: None,
            layout: None,
            draw: None,
        }
    }
}
