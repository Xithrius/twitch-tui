pub fn align_text(text: &str, alignment: &str, maximum_length: &u16) -> String {
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

pub fn vector2_col_max<T>(vec2: Vec<Vec<T>>) -> (u16, u16)
where
    T: AsRef<str>,
{
    let col0 = vec2.iter().map(|v| v[0].as_ref().len()).max().unwrap();
    let col1 = vec2.iter().map(|v| v[1].as_ref().len()).max().unwrap();

    (col0 as u16, col1 as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_string_vec2() {
        let vec2 = vec![vec!["", "s"], vec!["longer string", "lll"]];

        let (col0, col1) = vector2_col_max(vec2);

        assert_eq!(col0, 13);
        assert_eq!(col1, 3);
    }

    #[test]
    fn test_string_vec2() {
        let vec2 = vec![
            vec!["".to_string(), "another".to_string()],
            vec!["".to_string(), "the last string".to_string()],
        ];

        let (col0, col1) = vector2_col_max(vec2);

        assert_eq!(col0, 0);
        assert_eq!(col1, 15);
    }
}
