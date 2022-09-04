use tui::backend::Backend;

use crate::{
    ui::{
        insert_box_chunk,
        statics::{COMMANDS, TWITCH_MESSAGE_LIMIT},
        WindowAttributes,
    },
    utils::text::suggestion_query,
};

pub fn ui_insert_message<T: Backend>(window: WindowAttributes<T>, mention_suggestions: bool) {
    let WindowAttributes {
        frame: _,
        app,
        layout: _,
    } = &window;

    let input_buffer = app.current_buffer();

    let current_input = input_buffer.to_string();

    let suggestion = if mention_suggestions {
        input_buffer
            .chars()
            .next().and_then(|start_character| match start_character {
                '/' => {
                    let possible_suggestion = suggestion_query(
                        current_input[1..].to_string(),
                        COMMANDS
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<String>>(),
                    );

                    if let Some(s) = possible_suggestion {
                        Some(format!("/{}", s))
                    } else {
                        possible_suggestion
                    }
                }
                '@' => {
                    let possible_suggestion = suggestion_query(
                        current_input[1..].to_string(),
                        app.storage.get("mentions".to_string()),
                    );

                    possible_suggestion.map_or(possible_suggestion, |s| Some(format!("@{}", s)))
                }
                _ => None,
            })
    } else {
        None
    };

    insert_box_chunk(
        window,
        format!(
            "Message Input: {} / {}",
            current_input.len(),
            *TWITCH_MESSAGE_LIMIT
        )
        .as_str(),
        None,
        suggestion,
        Some(Box::new(|s: String| -> bool {
            s.len() < *TWITCH_MESSAGE_LIMIT
        })),
    );
}
