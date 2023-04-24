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

#[derive(Debug)]
pub struct ChatWidget {
    config: SharedCompleteConfig,
    tx: Sender<TwitchAction>,
}

impl ChatWidget {
    pub fn new(config: SharedCompleteConfig, tx: Sender<TwitchAction>) -> Self {
        Self { config, tx }
    }
}

impl Component for ChatWidget {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Option<Rect>) {
        let component_area = area.map_or_else(|| f.size(), |a| a);

        todo!()
    }

    fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            match key {
                Key::Char('q') => return Some(TerminalAction::Quitting),
                Key::Esc => return Some(TerminalAction::BackOneLayer),
                Key::Ctrl('p') => panic!("Manual panic triggered by user."),
                _ => {}
            }
        }

        None
    }
}

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
