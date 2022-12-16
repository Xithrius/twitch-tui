use tui::backend::Backend;

use crate::{
    ui::{
        components::render_insert_box,
        statics::{COMMANDS, TWITCH_MESSAGE_LIMIT},
        WindowAttributes,
    },
    utils::text::suggestion_query,
};

pub fn render_chat_box<T: Backend>(window: WindowAttributes<T>, mention_suggestions: bool) {
    let WindowAttributes {
        frame: _,
        app,
        layout: _,
        show_state_tabs: _,
    } = &window;

    let input_buffer = &app.input_buffer;

    let current_input = input_buffer.to_string();

    let suggestion = if mention_suggestions {
        input_buffer
            .chars()
            .next()
            .and_then(|start_character| match start_character {
                '/' => {
                    let possible_suggestion = suggestion_query(
                        &current_input[1..],
                        &COMMANDS
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<String>>(),
                    );

                    let default_suggestion = possible_suggestion.clone();

                    possible_suggestion.map_or(default_suggestion, |s| Some(format!("/{s}")))
                }
                '@' => {
                    let possible_suggestion =
                        suggestion_query(&current_input[1..], &app.storage.get("mentions"));

                    let default_suggestion = possible_suggestion.clone();

                    possible_suggestion.map_or(default_suggestion, |s| Some(format!("@{s}")))
                }
                _ => None,
            })
    } else {
        None
    };

    render_insert_box(
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
