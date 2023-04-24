use rustyline::{line_buffer::LineBuffer, At, Word};
use tokio::sync::broadcast::Sender;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    emotes::{Emotes, SharedEmotes},
    handlers::{
        config::SharedCompleteConfig,
        data::DataBuilder,
        user_input::{
            events::{Event, Key},
            input::TerminalAction,
        },
    },
    twitch::TwitchAction,
    ui::{
        components::Component,
        statics::{COMMANDS, TWITCH_MESSAGE_LIMIT},
    },
    utils::text::{first_similarity, get_cursor_position, title_spans, TitleStyle},
};

use super::utils::centered_rect;

// pub fn render_chat_box<T: Backend>(mention_suggestions: bool) {
//     let input_buffer = &app.input_buffer;

//     let current_input = input_buffer.to_string();

//     let suggestion = if mention_suggestions {
//         input_buffer
//             .chars()
//             .next()
//             .and_then(|start_character| match start_character {
//                 '/' => {
//                     let possible_suggestion = first_similarity(
//                         &COMMANDS
//                             .iter()
//                             .map(ToString::to_string)
//                             .collect::<Vec<String>>(),
//                         &current_input[1..],
//                     );

//                     let default_suggestion = possible_suggestion.clone();

//                     possible_suggestion.map_or(default_suggestion, |s| Some(format!("/{s}")))
//                 }
//                 '@' => {
//                     let possible_suggestion =
//                         first_similarity(&app.storage.get("mentions"), &current_input[1..]);

//                     let default_suggestion = possible_suggestion.clone();

//                     possible_suggestion.map_or(default_suggestion, |s| Some(format!("@{s}")))
//                 }
//                 _ => None,
//             })
//     } else {
//         None
//     };

//     render_insert_box(
//         window,
//         format!(
//             "Message Input: {} / {}",
//             current_input.len(),
//             *TWITCH_MESSAGE_LIMIT
//         )
//         .as_str(),
//         None,
//         suggestion,
//         Some(Box::new(|s: String| -> bool {
//             s.len() < *TWITCH_MESSAGE_LIMIT
//         })),
//     );
// }
