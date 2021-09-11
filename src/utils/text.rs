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

pub fn vector2_col_sums(vec2: Vec<Vec<&str>>) -> (u16, u16) {
    let col0 = vec2.iter().map(|v| v[0].len()).sum::<usize>();
    let col1 = vec2.iter().map(|v| v[1].len()).sum::<usize>();

    (col0 as u16, col1 as u16)
}
