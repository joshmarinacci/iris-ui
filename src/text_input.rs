use crate::geom::Bounds;
use crate::gfx::TextStyle;
use crate::input::{InputEvent, OutputAction, TextAction};
use crate::view::{Align, View, ViewId};
use crate::{DrawEvent, GuiEvent};
use alloc::boxed::Box;
use alloc::string::String;
use log::info;

pub struct TextInputState {
    cursor: usize,
    text: String,
}

impl TextInputState {
    fn cursor_back(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.text[..self.cursor]
                .char_indices()
                .next_back()
                .map_or(0, |(i, _)| i);
        }
    }
    fn cursor_forward(&mut self) {
        if let Some(ch) = self.text[self.cursor..].chars().next() {
            self.cursor += ch.len_utf8();
        }
    }
    fn delete_back(&mut self) {
        if self.cursor > 0 {
            self.cursor_back();
            self.text.remove(self.cursor);
        }
    }
    fn delete_forward(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }
    fn insert_char(&mut self, key: char) {
        self.text.insert(self.cursor, key);
        self.cursor += key.len_utf8();
    }
}

fn draw_text_input(e: &mut DrawEvent) {
    e.ctx.fill_rect(&e.view.bounds, &e.theme.standard.fill);
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.standard.text);
    let style = TextStyle::new(e.theme.font, &e.theme.standard.text).with_halign(Align::Start);

    let bounds = e.view.bounds.clone();
    if let Some(state) = e.view.get_state::<TextInputState>() {
        e.ctx.fill_text(&bounds, &state.text, &style);
    }

    if let Some(focused) = e.focused {
        if focused == &e.view.name {
            e.ctx
                .stroke_rect(&e.view.bounds.contract(2), &e.theme.standard.text);
            if let Some(state) = e.view.get_state::<TextInputState>() {
                let n = state.cursor as i32;
                let w = e.theme.font.char_width();
                let h = e.theme.font.char_height();
                e.ctx.fill_rect(
                    &Bounds::new(
                        e.view.bounds.position.x + n * w + 5,
                        e.view.bounds.position.y + 5,
                        2,
                        h + 4,
                    ),
                    &e.theme.accented.fill,
                );
            }
        }
    }
}

fn input_text_input(event: &mut GuiEvent) -> Option<OutputAction> {
    info!("text input got event {:?}", event.event_type);
    event.scene.mark_dirty_view(event.target);
    if let Some(state) = event.scene.get_view_state::<TextInputState>(event.target) {
        match &event.event_type {
            InputEvent::Text(text_action) => {
                match &text_action {
                    TextAction::TypedAscii(key) => match *key {
                        8 => {
                            state.delete_back();
                        }
                        13 => {
                            info!("doing return");
                            return Some(OutputAction::Command("Completed".into()));
                        }
                        _ => {
                            state.insert_char(*key as char);
                        }
                    },
                    TextAction::Left => state.cursor_back(),
                    TextAction::Right => state.cursor_forward(),
                    TextAction::Up => {}
                    TextAction::Down => {}
                    TextAction::BackDelete => state.delete_back(),
                    TextAction::ForwardDelete => state.delete_forward(),
                    TextAction::Enter => {
                        return Some(OutputAction::Command("Completed".into()));
                    }
                    TextAction::Unknown => {}
                }
                event.scene.mark_dirty_view(event.target);
            }
            InputEvent::Tap(_pt) => {
                event.scene.set_focused(event.target);
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::TextInputState;
    use alloc::string::String;

    fn state(text: &str) -> TextInputState {
        let cursor = text.len();
        TextInputState { text: String::from(text), cursor }
    }

    #[test]
    fn cursor_moves_by_char_not_byte() {
        // "é" is 2 bytes (U+00E9). cursor should land at byte 0, not byte 1.
        let mut s = state("é");
        assert_eq!(s.cursor, 2);
        s.cursor_back();
        assert_eq!(s.cursor, 0, "cursor_back must step over the full 2-byte char");
        s.cursor_forward();
        assert_eq!(s.cursor, 2, "cursor_forward must step over the full 2-byte char");
    }

    #[test]
    fn delete_back_removes_full_char() {
        let mut s = state("aé");
        // cursor is at byte 3 (end). delete_back should remove 'é' (2 bytes).
        s.delete_back();
        assert_eq!(s.text, "a");
        assert_eq!(s.cursor, 1);
    }

    #[test]
    fn delete_forward_removes_full_char() {
        let mut s = state("éb");
        s.cursor = 0;
        s.delete_forward();
        assert_eq!(s.text, "b");
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn insert_char_advances_by_char_width() {
        let mut s = state("");
        s.insert_char('é'); // 2-byte char
        assert_eq!(s.cursor, 2, "cursor must advance by len_utf8 after insert");
        assert_eq!(s.text, "é");
        s.insert_char('a'); // 1-byte char
        assert_eq!(s.cursor, 3);
        assert_eq!(s.text, "éa");
    }

    #[test]
    fn ascii_round_trip() {
        let mut s = state("hello");
        assert_eq!(s.cursor, 5);
        s.cursor_back();
        assert_eq!(s.cursor, 4);
        s.cursor_forward();
        assert_eq!(s.cursor, 5);
        s.delete_back();
        assert_eq!(s.text, "hell");
        assert_eq!(s.cursor, 4);
    }
}

pub fn make_text_input(name: &ViewId, title: &str) -> View {
    View {
        name: name.clone(),
        title: title.into(),
        bounds: Bounds::new(0, 0, 100, 30),
        visible: true,
        state: Some(Box::new(TextInputState {
            text: title.into(),
            cursor: title.len(),
        })),
        input: Some(input_text_input),
        layout: Some(|_e| {
            // if let Some(view) = e.scene.get_view_mut(e.target) {
            //     view.bounds = util::calc_bounds(view.bounds, e.theme.bold_font, &view.title);
            // }
        }),
        draw: Some(draw_text_input),
        ..Default::default()
    }
}
