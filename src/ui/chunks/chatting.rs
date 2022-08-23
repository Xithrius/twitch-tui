use tui::{backend::Backend, terminal::Frame};

use crate::{
    handlers::app::App,
    ui::{statics::COMMANDS, LayoutAttributes},
};

use super::insert_box_chunk;

pub fn message_input<T: Backend>(
    frame: &mut Frame<T>,
    app: &mut App,
    layout: LayoutAttributes,
    mention_suggestions: bool,
) {
    let input_buffer = app.current_buffer();

    let current_input = input_buffer.to_string();

    let suggestion = if let Some(start_character) = input_buffer.chars().next() {
        let first_result = |choices: Vec<String>, choice: String| -> String {
            if let Some(result) = choices
                .iter()
                .filter(|s| s.starts_with(&choice[1..]))
                .collect::<Vec<&String>>()
                .first()
            {
                result.to_string()
            } else {
                "".to_string()
            }
        };

        match start_character {
            '/' => format!(
                "/{}",
                first_result(
                    COMMANDS
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>(),
                    input_buffer.to_string(),
                )
            ),
            '@' => {
                if mention_suggestions {
                    format!(
                        "@{}",
                        first_result(
                            app.storage.get("mentions".to_string()),
                            input_buffer.to_string(),
                        )
                    )
                } else {
                    "".to_string()
                }
            }
            _ => "".to_string(),
        }
    } else {
        "".to_string()
    };

    insert_box_chunk(frame, layout, *input_buffer, Some(suggestion));

    app.buffer_suggestion = suggestion;
}
