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
    let WindowAttributes { frame, app, layout } = window;

    let input_buffer = app.current_buffer();

    let current_input = input_buffer.to_string();

    let suggestion = if mention_suggestions {
        if let Some(start_character) = input_buffer.chars().next() {
            match start_character {
                '/' => {
                    let possible_suggestion = suggestion_query(
                        current_input[1..].to_string(),
                        COMMANDS
                            .iter()
                            .map(|s| s.to_string())
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

                    if let Some(s) = possible_suggestion {
                        Some(format!("@{}", s))
                    } else {
                        possible_suggestion
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    } else {
        None
    };

    insert_box_chunk(
        frame,
        app,
        layout,
        None,
        suggestion,
        Some(Box::new(|s: String| -> bool {
            s.len() < *TWITCH_MESSAGE_LIMIT
        })),
    );
}
