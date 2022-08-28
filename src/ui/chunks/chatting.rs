use tui::backend::Backend;

use crate::{
    ui::{
        insert_box_chunk,
        statics::{COMMANDS, TWITCH_MESSAGE_LIMIT},
        WindowAttributes,
    },
    utils::text::suggestion_query,
};

pub fn ui_insert_message<'a: 'b, 'b, 'c, T: Backend>(
    window: WindowAttributes<'a, 'b, 'c, T>,
    mention_suggestions: bool,
) {
    let WindowAttributes { frame, app, layout } = window;

    let input_buffer = app.current_buffer();

    let current_input = input_buffer.to_string();

    let suggestion = if mention_suggestions {
        if let Some(start_character) = input_buffer.chars().next() {
            match start_character {
                '/' => suggestion_query(
                    current_input,
                    COMMANDS
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>(),
                ),
                '@' => suggestion_query(current_input, app.storage.get("mentions".to_string())),
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
        suggestion.clone(),
        Some(Box::new(|s: String| -> bool {
            s.len() < *TWITCH_MESSAGE_LIMIT
        })),
    );
}
