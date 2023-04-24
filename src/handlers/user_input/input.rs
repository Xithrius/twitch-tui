// use regex::Regex;
// use rustyline::{At, Word};
// use tokio::sync::broadcast::Sender;

// use crate::{
//     emotes::{unload_all_emotes, Emotes},
//     handlers::{
//         app::App,
//         config::CompleteConfig,
//         data::DataBuilder,
//         state::State,
//         user_input::events::{Event, Events, Key},
//     },
//     twitch::TwitchAction,
//     ui::statics::{NAME_RESTRICTION_REGEX, TWITCH_MESSAGE_LIMIT},
// };

pub enum TerminalAction {
    Quitting,
    BackOneLayer,
}

// struct UserActionAttributes<'a, 'b> {
//     app: &'a mut App,
//     config: &'b mut CompleteConfig,
//     tx: Sender<TwitchAction>,
//     key: Key,
// }

// impl<'a, 'b> UserActionAttributes<'a, 'b> {
//     fn new(
//         app: &'a mut App,
//         config: &'b mut CompleteConfig,
//         tx: Sender<TwitchAction>,
//         key: Key,
//     ) -> Self {
//         Self {
//             app,
//             config,
//             tx,
//             key,
//         }
//     }
// }

// fn handle_insert_enter_key(action: &mut UserActionAttributes<'_, '_>, emotes: &mut Emotes) {
//     let UserActionAttributes {
//         app,
//         config,
//         key: _,
//         tx,
//     } = action;

//     match app.get_state() {
//         State::Insert => {
//             let input_message = &mut app.input_buffer;

//             if input_message.is_empty()
//                 || app.filters.contaminated(input_message.as_str())
//                 || input_message.len() > *TWITCH_MESSAGE_LIMIT
//             {
//                 return;
//             }

//             let mut message = DataBuilder::user(
//                 config.twitch.username.to_string(),
//                 input_message.to_string(),
//             );
//             message.parse_emotes(emotes);

//             app.messages.push_front(message);

//             tx.send(TwitchAction::Privmsg(input_message.to_string()))
//                 .unwrap();

//             if let Some(msg) = input_message.strip_prefix('@') {
//                 app.storage.add("mentions", msg.to_string());
//             }

//             let mut possible_command = String::new();

//             input_message.clone_into(&mut possible_command);

//             input_message.update("", 0);

//             if possible_command.as_str() == "/clear" {
//                 app.clear_messages();
//             }
//         }
//         State::ChannelSwitch => {
//             let input_message = &mut app.input_buffer;

//             if input_message.is_empty()
//                 || !Regex::new(&NAME_RESTRICTION_REGEX)
//                     .unwrap()
//                     .is_match(input_message)
//             {
//                 return;
//             }

//             // TODO: if input message is the same as the current config, return to normal state.

//             app.messages.clear();
//             unload_all_emotes(emotes);

//             tx.send(TwitchAction::Join(input_message.to_string()))
//                 .unwrap();

//             config.twitch.channel = input_message.to_string();

//             app.storage.add("channels", input_message.to_string());

//             input_message.update("", 0);

//             app.set_state(State::Normal);
//         }
//         _ => {}
//     }
// }

// fn handle_insert_type_movements(action: &mut UserActionAttributes<'_, '_>, emotes: &mut Emotes) {
//     let UserActionAttributes {
//         app,
//         config: _,
//         key,
//         tx: _,
//     } = action;

//     let input_buffer = &mut app.input_buffer;

//     match key {
//         Key::Up => {
//             if app.get_state() == State::Insert {
//                 app.set_state(State::Normal);
//             }
//         }
//         Key::Ctrl('f') | Key::Right => {
//             input_buffer.move_forward(1);
//         }
//         Key::Ctrl('b') | Key::Left => {
//             input_buffer.move_backward(1);
//         }
//         Key::Ctrl('a') | Key::Home => {
//             input_buffer.move_home();
//         }
//         Key::Ctrl('e') | Key::End => {
//             input_buffer.move_end();
//         }
//         Key::Alt('f') => {
//             input_buffer.move_to_next_word(At::AfterEnd, Word::Emacs, 1);
//         }
//         Key::Alt('b') => {
//             input_buffer.move_to_prev_word(Word::Emacs, 1);
//         }
//         Key::Ctrl('t') => {
//             input_buffer.transpose_chars();
//         }
//         Key::Alt('t') => {
//             input_buffer.transpose_words(1);
//         }
//         Key::Ctrl('u') => {
//             input_buffer.discard_line();
//         }
//         Key::Ctrl('k') => {
//             input_buffer.kill_line();
//         }
//         Key::Ctrl('w') => {
//             input_buffer.delete_prev_word(Word::Emacs, 1);
//         }
//         Key::Ctrl('d') => {
//             input_buffer.delete(1);
//         }
//         Key::Backspace | Key::Delete => {
//             input_buffer.backspace(1);
//         }
//         Key::Tab => {
//             let suggestion = app.buffer_suggestion.clone();

//             if let Some(suggestion_buffer) = suggestion {
//                 app.input_buffer
//                     .update(suggestion_buffer.as_str(), suggestion_buffer.len());
//             }
//         }
//         Key::Enter => handle_insert_enter_key(action, emotes),
//         Key::Char(c) => {
//             input_buffer.insert(*c, 1);
//         }
//         Key::Esc => {
//             input_buffer.update("", 0);
//             app.set_state(State::Normal);
//         }
//         _ => {}
//     }
// }

// fn handle_user_scroll(app: &mut App, key: Key) {
//     match app.get_state() {
//         State::Insert | State::MessageSearch | State::Normal => {
//             let limit = app.scrolling.get_offset() < app.messages.len();

//             match key {
//                 Key::ScrollUp => {
//                     if limit {
//                         app.scrolling.up();
//                     } else if app.scrolling.inverted() {
//                         app.scrolling.down();
//                     }
//                 }
//                 Key::ScrollDown => {
//                     if app.scrolling.inverted() {
//                         if limit {
//                             app.scrolling.up();
//                         }
//                     } else {
//                         app.scrolling.down();
//                     }
//                 }
//                 _ => {}
//             }
//         }
//         _ => {}
//     }
// }

// pub async fn handle_stateful_user_input(
//     events: &mut Events,
//     app: &mut App,
//     config: &mut CompleteConfig,
//     tx: Sender<TwitchAction>,
//     emotes: &mut Emotes,
// ) -> Option<TerminalAction> {
//     if let Some(Event::Input(key)) = events.next().await {
//         handle_user_scroll(app, key);

//         match app.get_state() {
//             State::Help => {
//                 if matches!(key, Key::Esc) {
//                     if let Some(previous_state) = app.get_previous_state() {
//                         app.set_state(previous_state);
//                     }
//                 }
//             }
//             State::ChannelSwitch => {
//                 if matches!(key, Key::Esc) {
//                     if let Some(previous_state) = app.get_previous_state() {
//                         app.set_state(previous_state);
//                     } else {
//                         app.set_state(config.terminal.start_state.clone());
//                     }
//                 } else {
//                     let mut action = UserActionAttributes::new(app, config, tx, key);

//                     handle_insert_type_movements(&mut action, emotes);
//                 }
//             }
//             State::Insert | State::MessageSearch => {
//                 let mut action = UserActionAttributes::new(app, config, tx, key);

//                 handle_insert_type_movements(&mut action, emotes);
//             }
//             State::Normal => match key {
//                 Key::Char('c') => app.set_state(State::Normal),
//                 Key::Char('s') => app.set_state(State::ChannelSwitch),
//                 Key::Ctrl('f') => app.set_state(State::MessageSearch),
//                 // Key::Ctrl('d') => app.debug.toggle(),
//                 Key::Ctrl('t') => app.filters.toggle(),
//                 Key::Ctrl('r') => app.filters.reverse(),
//                 Key::Char('i') | Key::Insert => app.set_state(State::Insert),
//                 Key::Char('@' | '/') => {
//                     app.set_state(State::Insert);
//                     app.input_buffer.update(&key.to_string(), 1);
//                 }
//                 Key::Ctrl('p') => panic!("Manual panic triggered by user."),
//                 Key::Char('S') => app.set_state(State::Dashboard),
//                 Key::Char('?') => app.set_state(State::Help),
//                 Key::Char('q') => return Some(TerminalAction::Quitting),
//                 Key::Esc => {
//                     app.scrolling.jump_to(0);

//                     app.set_state(State::Normal);
//                 }
//                 _ => {}
//             },
//             _ => {}
//         }
//     }

//     None
// }
