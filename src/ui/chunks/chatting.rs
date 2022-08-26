use tui::{backend::Backend, terminal::Frame};

use crate::{
    handlers::app::App,
    ui::{
        statics::{COMMANDS, TWITCH_MESSAGE_LIMIT},
        LayoutAttributes,
    },
    utils::text::suggestion_partition,
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

    let suggestion = if mention_suggestions {
        if let Some(start_character) = input_buffer.chars().next() {
            match start_character {
                '/' => suggestion_partition(
                    current_input,
                    COMMANDS
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>(),
                ),
                '@' => suggestion_partition(current_input, app.storage.get("mentions".to_string())),
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
        layout,
        input_buffer,
        suggestion.clone(),
        Some(Box::new(|s: String| -> bool {
            s.len() < *TWITCH_MESSAGE_LIMIT
        })),
    );

    app.buffer_suggestion = suggestion;
}
