use rustyline::line_buffer::LineBuffer;
use std::vec::IntoIter;
use textwrap::core::display_width;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub fn align_text(text: &str, alignment: &str, maximum_length: u16) -> String {
    if maximum_length < 1 {
        panic!("Parameter of 'maximum_length' cannot be below 1.");
    }

    // Compute the display width of `text` with support of emojis and CJK characters
    let mut dw = display_width(text);
    if dw > maximum_length as usize {
        dw = maximum_length as usize;
    }
    match alignment {
        "right" => format!("{}{}", " ".repeat(maximum_length as usize - dw), text),
        "center" => {
            let side_spaces =
                " ".repeat(((maximum_length / 2) - (((dw / 2) as f32).floor() as u16)) as usize);
            format!("{}{}{}", side_spaces, text, side_spaces)
        }
        _ => text.to_string(),
    }
}

pub enum VectorColumnMax<T> {
    One(T),
    All(Vec<T>),
}

impl<T> IntoIterator for VectorColumnMax<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            VectorColumnMax::All(item) => item.into_iter(),
            VectorColumnMax::One(item) => vec![item].into_iter(),
        }
    }
}

pub fn vector_column_max<T>(vec: &[Vec<T>], indexer: Option<usize>) -> IntoIter<u16>
where
    T: AsRef<str>,
{
    if vec.is_empty() {
        panic!("Vector length should be greater than or equal to 1.")
    }

    let column_max = |vec: &[Vec<T>], index: usize| -> u16 {
        vec.iter().map(|v| v[index].as_ref().len()).max().unwrap() as u16
    };

    match indexer {
        Some(index) => VectorColumnMax::One(column_max(vec, index)).into_iter(),
        None => {
            let column_amount = vec[0].len();

            let mut column_max_lengths: Vec<u16> = vec![];

            for i in 0..column_amount {
                column_max_lengths.push(column_max(vec, i));
            }

            VectorColumnMax::All(column_max_lengths).into_iter()
        }
    }
}

pub fn get_cursor_position(line_buffer: &LineBuffer) -> usize {
    line_buffer
        .as_str()
        .grapheme_indices(true)
        .take_while(|(offset, _)| *offset != line_buffer.pos())
        .map(|(_, cluster)| cluster.width())
        .sum()
}

#[test]
#[should_panic(expected = "Parameter of 'maximum_length' cannot be below 1.")]
fn test_text_align_maximum_length() {
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
    assert_eq!(align_text("你好", "right", 5), " 你好");
    assert_eq!(align_text("👑123", "right", 6), " 👑123");
}

#[test]
fn test_text_align_center() {
    assert_eq!(
        align_text("a", "center", 11),
        format!("{}{}{}", " ".repeat(5), "a", " ".repeat(5))
    );
    assert_eq!(align_text("a", "center", 1), "a".to_string());
    assert_eq!(align_text("你好", "center", 6), " 你好 ");
    assert_eq!(align_text("👑123", "center", 7), " 👑123 ");
}

#[test]
#[should_panic(expected = "Vector length should be greater than or equal to 1.")]
fn test_vector_column_max_empty_vector() {
    let vec: Vec<Vec<String>> = vec![];

    vector_column_max(&vec, None);
}

#[test]
fn test_vector_column_max_reference_strings() {
    let vec = vec![vec!["", "s"], vec!["longer string", "lll"]];

    let mut output_vec_all = vector_column_max(&vec, None);

    assert_eq!(output_vec_all.next(), Some(13));
    assert_eq!(output_vec_all.next(), Some(3));

    let mut output_vec_one = vector_column_max(&vec, Some(0));

    assert_eq!(output_vec_one.next(), Some(13));
}

#[test]
fn test_vector_column_max_strings() {
    let vec = vec![
        vec!["".to_string(), "another".to_string()],
        vec!["".to_string(), "the last string".to_string()],
    ];

    let mut output_vec_all = vector_column_max(&vec, None);

    assert_eq!(output_vec_all.next(), Some(0));
    assert_eq!(output_vec_all.next(), Some(15));

    let mut output_vec_one = vector_column_max(&vec, Some(0));

    assert_eq!(output_vec_one.next(), Some(0));
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
