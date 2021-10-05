use rustyline::line_buffer::LineBuffer;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub fn align_text(text: &str, alignment: &str, maximum_length: u16) -> String {
    if maximum_length < 1 {
        panic!("Parameter of 'maximum_length' cannot be below 1.");
    }

    match alignment {
        "left" => text.to_string(),
        "right" => format!(
            "{}{}",
            " ".repeat((maximum_length - text.len() as u16) as usize),
            text
        ),
        "center" => {
            let side_spaces = " ".repeat(
                ((maximum_length / 2) - (((text.len() / 2) as f32).floor() as u16)) as usize,
            );

            format!("{}{}{}", side_spaces, text, side_spaces)
        }
        _ => text.to_string(),
    }
}

pub fn vector2_col_max<T>(vec2: &[Vec<T>]) -> (u16, u16)
where
    T: AsRef<str>,
{
    let col0 = vec2.iter().map(|v| v[0].as_ref().len()).max().unwrap();
    let col1 = vec2.iter().map(|v| v[1].as_ref().len()).max().unwrap();

    (col0 as u16, col1 as u16)
}

pub fn get_cursor_position(line_buffer: &LineBuffer) -> usize {
    line_buffer
        .as_str()
        .grapheme_indices(true)
        .take_while(|(offset, _)| *offset != line_buffer.pos())
        .map(|(_, cluster)| cluster.width())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Parameter of 'maximum_length' cannot be below 1.")]
    fn test_maximum_length() {
        align_text("", "left", 0);
    }

    #[test]
    fn test_text_align_left() {
        assert_eq!(align_text("a", "left", 10), "a".to_string());
        assert_eq!(align_text("a", "left", 1), "a".to_string());
    }

    #[test]
    fn test_text_align_right() {
        assert_eq!(
            align_text("a", "right", 10),
            format!("{}{}", " ".repeat(9), "a")
        );
        assert_eq!(align_text("a", "right", 1), "a".to_string());
    }

    #[test]
    fn test_text_align_center() {
        assert_eq!(
            align_text("a", "center", 10),
            format!("{}{}{}", " ".repeat(5), "a", " ".repeat(5))
        );
        assert_eq!(align_text("a", "center", 1), "a".to_string());
    }

    #[test]
    fn test_reference_string_vec2() {
        let vec2 = vec![vec!["", "s"], vec!["longer string", "lll"]];

        let (col0, col1) = vector2_col_max(&vec2);

        assert_eq!(col0, 13);
        assert_eq!(col1, 3);
    }

    #[test]
    fn test_string_vec2() {
        let vec2 = vec![
            vec!["".to_string(), "another".to_string()],
            vec!["".to_string(), "the last string".to_string()],
        ];

        let (col0, col1) = vector2_col_max(&vec2);

        assert_eq!(col0, 0);
        assert_eq!(col1, 15);
    }

    #[test]
    fn test_get_cursor_position_with_single_byte_graphemes() {
        let text = "never gonna give you up";
        let mut line_buffer = LineBuffer::with_capacity(25);
        line_buffer.insert_str(0, text);

        assert_eq!(get_cursor_position(&line_buffer), 0);
        line_buffer.move_forward(1);
        assert_eq!(get_cursor_position(&line_buffer), 1);
        line_buffer.move_forward(2);
        assert_eq!(get_cursor_position(&line_buffer), 3);
    }

    #[test]
    fn test_get_cursor_position_with_three_byte_graphemes() {
        let text = "绝对不会放弃你";
        let mut line_buffer = LineBuffer::with_capacity(25);
        line_buffer.insert_str(0, text);

        assert_eq!(get_cursor_position(&line_buffer), 0);
        line_buffer.move_forward(1);
        assert_eq!(get_cursor_position(&line_buffer), 2);
        line_buffer.move_forward(2);
        assert_eq!(get_cursor_position(&line_buffer), 6);
    }
}
