use rustyline::line_buffer::LineBuffer;
use textwrap::core::display_width;
use tui::{style::Style, text::Span};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::handlers::config::Alignment;

pub fn align_text(text: &str, alignment: Alignment, maximum_length: u16) -> String {
    assert!(
        maximum_length >= 1,
        "Parameter of 'maximum_length' cannot be below 1."
    );

    // Compute the display width of `text` with support of emojis and CJK characters
    let mut dw = display_width(text);

    if dw > maximum_length as usize {
        dw = maximum_length as usize;
    }

    match alignment {
        Alignment::Right => format!("{}{}", " ".repeat(maximum_length as usize - dw), text),
        Alignment::Center => {
            let side_spaces =
                " ".repeat(((maximum_length / 2) - (((dw / 2) as f32).floor() as u16)) as usize);
            format!("{}{}{}", side_spaces, text, side_spaces)
        }
        Alignment::Left => text.to_string(),
    }
}

/// Acquiring the horizontal position of the cursor so it can be rendered visually.
pub fn get_cursor_position(line_buffer: &LineBuffer) -> usize {
    line_buffer
        .as_str()
        .grapheme_indices(true)
        .take_while(|(offset, _)| *offset != line_buffer.pos())
        .map(|(_, cluster)| cluster.width())
        .sum()
}

pub enum TitleStyle<'a> {
    Combined(&'a str, &'a str),
    Single(&'a str),
    Custom(Span<'a>),
}

pub fn title_spans(contents: Vec<TitleStyle>, style: Style) -> Vec<Span> {
    let mut complete = Vec::new();

    for (i, item) in contents.iter().enumerate() {
        let first_bracket = Span::raw(format!("{}[ ", if i == 0 { "" } else { " " }));

        complete.extend(match item {
            TitleStyle::Combined(title, value) => vec![
                first_bracket,
                Span::styled((*title).to_string(), style),
                Span::raw(format!(": {} ]", value)),
            ],
            TitleStyle::Single(value) => vec![
                first_bracket,
                Span::styled((*value).to_string(), style),
                Span::raw(" ]"),
            ],
            TitleStyle::Custom(span) => vec![first_bracket, span.clone(), Span::raw(" ]")],
        });
    }

    complete
}

/// Return the first suggestion out of a bunch of possibilities
/// based on starting characters of the search string.
pub fn suggestion_query(search: &str, possibilities: Vec<String>) -> Option<String> {
    possibilities
        .iter()
        .filter(|s| s.starts_with(search))
        .collect::<Vec<&String>>()
        .first()
        .and_then(|result| {
            if result.len() > search.len() {
                Some((*result).to_string())
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use tui::{
        style::{Color, Modifier},
        text::Spans,
    };

    use super::*;

    #[test]
    #[should_panic(expected = "Parameter of 'maximum_length' cannot be below 1.")]
    fn test_text_align_maximum_length() {
        align_text("", Alignment::Left, 0);
    }

    #[test]
    fn test_text_align_left() {
        assert_eq!(align_text("a", Alignment::Left, 10), "a".to_string());
        assert_eq!(align_text("a", Alignment::Left, 1), "a".to_string());
    }

    #[test]
    fn test_text_align_right() {
        assert_eq!(
            align_text("a", Alignment::Right, 10),
            format!("{}{}", " ".repeat(9), "a")
        );
        assert_eq!(align_text("a", Alignment::Right, 1), "a".to_string());
        assert_eq!(align_text("你好", Alignment::Right, 5), " 你好");
        assert_eq!(align_text("👑123", Alignment::Right, 6), " 👑123");
    }

    #[test]
    fn test_text_align_center() {
        assert_eq!(
            align_text("a", Alignment::Center, 11),
            format!("{}{}{}", " ".repeat(5), "a", " ".repeat(5))
        );
        assert_eq!(align_text("a", Alignment::Center, 1), "a".to_string());
        assert_eq!(align_text("你好", Alignment::Center, 6), " 你好 ");
        assert_eq!(align_text("👑123", Alignment::Center, 7), " 👑123 ");
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

    #[test]
    fn test_2_dimensional_vector_to_spans() {
        let s = Spans::from(title_spans(
            vec![TitleStyle::Combined("Time", "Some time")],
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));

        assert_eq!(s.width(), "[ Time: Some time ]".len());
    }

    #[test]
    fn test_partial_suggestion_output() {
        let v = vec!["Nope".to_string()];

        let output = suggestion_query("No", v);

        assert_eq!(output, Some("Nope".to_string()));
    }
}
