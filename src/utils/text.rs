use std::{borrow::Cow, sync::LazyLock};

use memchr::memmem::Finder;
use rustyline::line_buffer::LineBuffer;
use tui::{style::Style, text::Span};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Acquiring the horizontal position of the cursor so it can be rendered visually.
pub fn get_cursor_position(line_buffer: &LineBuffer) -> usize {
    line_buffer
        .as_str()
        .grapheme_indices(true)
        .take_while(|(offset, _)| *offset != line_buffer.pos())
        .map(|(_, cluster)| cluster.width())
        .sum()
}

pub fn split_cow_in_place<'a>(cow: &mut Cow<'a, str>, mid: usize) -> Cow<'a, str> {
    match *cow {
        Cow::Owned(ref mut s) => {
            let s2 = s[mid..].to_string();
            s.truncate(mid);
            Cow::Owned(s2)
        }
        Cow::Borrowed(s) => {
            let (s1, s2) = s.split_at(mid);
            *cow = Cow::Borrowed(s1);
            Cow::Borrowed(s2)
        }
    }
}

pub enum TitleStyle<'a> {
    Combined(&'a str, &'a str),
    Single(&'a str),
    Custom(Span<'a>),
}

pub fn title_line<'a>(contents: &'a [TitleStyle<'a>], style: Style) -> Vec<Span<'a>> {
    let mut complete = Vec::new();

    for (i, item) in contents.iter().enumerate() {
        let first_bracket = Span::raw(format!("{}[ ", if i == 0 { "" } else { " " }));

        complete.extend(match item {
            TitleStyle::Combined(title, value) => vec![
                first_bracket,
                Span::styled((*title).to_string(), style),
                Span::raw(format!(": {value} ]")),
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

/// Within an array of strings, find the first partial or full match, if any.
pub fn first_similarity(possibilities: &[String], search: &str) -> Option<String> {
    first_similarity_iter(possibilities.iter(), search)
}

pub fn first_similarity_iter<'a>(
    possibilities: impl Iterator<Item = &'a String>,
    search: &str,
) -> Option<String> {
    if search.is_empty() {
        return None;
    }

    possibilities
        .filter(|s| s.starts_with(search))
        .collect::<Vec<&String>>()
        .first()
        .and_then(|result| {
            if result.len() > search.len() {
                Some((*result).clone())
            } else {
                None
            }
        })
}

/// <https://stackoverflow.com/a/38406885/>
pub fn capitalize_first_char(s: &str) -> String {
    let mut c = s.chars();

    c.next().map_or_else(String::new, |f| {
        f.to_uppercase().collect::<String>() + c.as_str()
    })
}

/// This function handles the detection and parsing of the twitch /me command.
/// This command is received as an IRC CTCP ACTION, which wraps the message,
/// and has this format: `"\u{1}ACTION " + msg + "\u{1}"`.
pub fn parse_message_action(msg: &str) -> (&str, bool) {
    const IRC_CTCP_ACTION: &str = "\u{1}ACTION ";

    // Extract the message from the irc ctcp action
    msg.strip_prefix(IRC_CTCP_ACTION)
        .map_or_else(|| (msg, false), |msg| (&msg[..msg.len() - 1], true))
}

/// Some twitch clients bypass the 30s timeout for duplicate messages by appending a space followed
/// by the `'\u{e0000}'` character to the end of the message.
///
/// This character should not be rendered, and should have a width of 0.
/// ratatui, which uses the crate `unicode-width`, and multiple terminals assume it has a width of 1.
/// This creates rendering issues in terminals that correctly avoid rendering this character.
///
/// As it is not meant to be rendered, we can just remove this character from the message.
pub fn clean_message(msg: &str) -> String {
    const U_E0000: char = '\u{e0000}';
    const U_E0000_LEN: usize = U_E0000.len_utf8();
    const U_E0000_STR: &str = "\u{e0000}";
    static FINDER: LazyLock<Finder> = LazyLock::new(|| Finder::new(U_E0000_STR));

    let msg = msg.trim_matches(['\0', ' ', U_E0000]);

    let bytes = msg.as_bytes();
    let matches = FINDER.find_iter(bytes).collect::<Vec<_>>();

    if matches.is_empty() {
        return msg.to_string();
    }

    // For the unlikely (but possible) case where this character appears in the middle of the message, remove it.
    // Do it manually instead of the naive `msg.replace(U_E0000, "")` which is slow.
    let mut output = Vec::with_capacity(msg.len() - U_E0000_LEN * matches.len());
    let mut last_match = 0;

    for idx in matches {
        output.extend_from_slice(&bytes[last_match..idx]);
        last_match = idx + U_E0000_LEN;
    }

    output.extend_from_slice(&bytes[last_match..]);

    // Unwrapping here is safe as the input is already valid utf8, and we only remove `U_E0000` from the string.
    String::from_utf8(output).unwrap()
}

#[cfg(test)]
mod tests {
    use tui::{
        style::{Color, Modifier},
        text::Line,
    };

    use super::*;
    use crate::ui::components::utils::InputListener;

    #[test]
    fn test_get_cursor_position_with_single_byte_graphemes() {
        let text = "never gonna give you up";
        let mut line_buffer = LineBuffer::with_capacity(25);
        let mut input_listener = InputListener;
        line_buffer.insert_str(0, text, &mut input_listener);

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
        let mut input_listener = InputListener;
        line_buffer.insert_str(0, text, &mut input_listener);

        assert_eq!(get_cursor_position(&line_buffer), 0);
        line_buffer.move_forward(1);
        assert_eq!(get_cursor_position(&line_buffer), 2);
        line_buffer.move_forward(2);
        assert_eq!(get_cursor_position(&line_buffer), 6);
    }

    #[test]
    fn test_2_dimensional_vector_to_line() {
        let s = Line::from(title_line(
            &[TitleStyle::Combined("Time", "Some time")],
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));

        assert_eq!(s.width(), "[ Time: Some time ]".len());
    }

    #[test]
    fn test_first_similarity_no_search_no_output() {
        let v = vec!["asdf".to_string()];

        let output = first_similarity(&v, "");

        assert_eq!(output, None);
    }

    #[test]
    fn test_first_similarity_some_output() {
        let v = vec!["Nope".to_string()];

        let output = first_similarity(&v, "No");

        assert_eq!(output, Some("Nope".to_string()));
    }

    #[test]
    fn test_first_similarity_no_output() {
        let v = vec!["Something".to_string()];

        let output = first_similarity(&v, "blah");

        assert_eq!(output, None);
    }

    #[test]
    fn test_first_similarity_no_input_no_output() {
        let output = first_similarity(&[], "asdf");

        assert_eq!(output, None);
    }

    #[test]
    fn clean_message_end() {
        let output = clean_message("foo \u{e0000}");

        assert_eq!(output, "foo");
    }

    #[test]
    fn clean_message_multiple_end() {
        let output = clean_message("foo \u{e0000} \u{e0000} \u{e0000}");

        assert_eq!(output, "foo");
    }

    #[test]
    fn clean_message_middle() {
        let output = clean_message("foo\u{e0000}bar \u{e0000} baz \u{e0000}");

        assert_eq!(output, "foobar  baz");
    }

    #[test]
    fn test_parse_message_action() {
        let (output, highlight) = parse_message_action("\u{1}ACTION foo\u{1}");

        assert_eq!(output, "foo");
        assert!(highlight);
    }

    #[test]
    fn test_parse_message_action2() {
        let (output, highlight) = parse_message_action("\u{1}ACTION foo\u{e0000}\u{1}");

        assert_eq!(output, "foo\u{e0000}");
        assert!(highlight);
    }

    #[test]
    fn test_parse_message_no_action() {
        let (output, highlight) = parse_message_action("foo\u{e0000}");

        assert_eq!(output, "foo\u{e0000}");
        assert!(!highlight);
    }

    #[test]
    fn parse_clean_message_action() {
        let (msg, highlight) =
            parse_message_action("\u{1}ACTION foo\u{e0000}bar \u{e0000} baz \u{1}");
        let output = clean_message(msg);

        assert_eq!(output, "foobar  baz");
        assert!(highlight);
    }
}
