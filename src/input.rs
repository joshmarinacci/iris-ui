use crate::geom::Point;
use crate::view::ViewId;
use alloc::string::String;

#[derive(Debug, Copy, Clone)]
pub enum InputEvent {
    /// The pointer (mouse button, or touch contact) went down at this point.
    PointerDown(Point),
    /// The pointer (mouse button, or touch contact) was released at this point.
    PointerUp(Point),
    Scroll(Point),
    Text(TextAction),
    Action(InputAction),
}
#[derive(Debug, Copy, Clone)]
pub enum InputAction {
    NavPrev,
    NavNext,
    FocusPrev,
    FocusNext,
    FocusSelect,
}
#[derive(Debug, Copy, Clone)]
pub enum TextAction {
    TypedAscii(u8),
    Left,
    Right,
    Up,
    Down,
    BackDelete,
    ForwardDelete,
    Enter,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct InputResult {
    pub source: ViewId,
    pub action: Option<OutputAction>,
    pub input: InputEvent,
}
#[derive(Debug, Clone)]
pub enum OutputAction {
    // Generic(Option(Box(dyn Any))),
    Command(String),
    Selected(String, usize),
    Focused(ViewId),
    TextChanged(String),
}
