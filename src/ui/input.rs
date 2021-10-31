use rustyline::{At, Word};

use crate::handlers::{app::App, event::Key};

pub async fn draw_input_ui(app: &mut App, input_box: &str, key: Key) {
    let input_buffer = app.input_boxes.get_mut(input_box).unwrap();

    match key {
        Key::Ctrl('f') | Key::Right => {
            input_buffer.move_forward(1);
        }
        Key::Ctrl('b') | Key::Left => {
            input_buffer.move_backward(1);
        }
        Key::Ctrl('a') | Key::Home => {
            input_buffer.move_home();
        }
        Key::Ctrl('e') | Key::End => {
            input_buffer.move_end();
        }
        Key::Alt('f') => {
            input_buffer.move_to_next_word(At::AfterEnd, Word::Emacs, 1);
        }
        Key::Alt('b') => {
            input_buffer.move_to_prev_word(Word::Emacs, 1);
        }
        Key::Ctrl('t') => {
            input_buffer.transpose_chars();
        }
        Key::Alt('t') => {
            input_buffer.transpose_words(1);
        }
        Key::Ctrl('u') => {
            input_buffer.discard_line();
        }
        Key::Ctrl('k') => {
            input_buffer.kill_line();
        }
        Key::Ctrl('w') => {
            input_buffer.delete_prev_word(Word::Emacs, 1);
        }
        Key::Ctrl('d') => {
            input_buffer.delete(1);
        }
        Key::Backspace | Key::Delete => {
            input_buffer.backspace(1);
        }
        Key::Char(c) => {
            input_buffer.insert(c, 1);
        }
        _ => {}
    }
}
