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
