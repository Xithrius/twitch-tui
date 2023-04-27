use tokio::sync::broadcast::Sender;
use tui::{backend::Backend, layout::Rect, Frame};

use crate::{
    emotes::Emotes,
    handlers::{
        config::SharedCompleteConfig,
        user_input::{
            events::{Event, Key},
            input::TerminalAction,
        },
    },
    twitch::TwitchAction,
    ui::{
        components::{utils::InputWidget, Component},
        statics::TWITCH_MESSAGE_LIMIT,
    },
};

pub struct ChatInputWidget {
    _config: SharedCompleteConfig,
    input: InputWidget,
    tx: Sender<TwitchAction>,
}

impl ChatInputWidget {
    pub fn new(config: SharedCompleteConfig, tx: Sender<TwitchAction>) -> Self {
        let input_validator = Box::new(|s: String| -> bool { s.len() < *TWITCH_MESSAGE_LIMIT });

        let input = InputWidget::new(config.clone(), "Chat", Some(input_validator), None);

        Self {
            _config: config,
            tx,
            input,
        }
    }

    pub const fn is_focused(&self) -> bool {
        self.input.is_focused()
    }

    pub fn toggle_focus(&mut self) {
        self.input.toggle_focus();
    }
}

impl ToString for ChatInputWidget {
    fn to_string(&self) -> String {
        self.input.to_string()
    }
}

impl Component for ChatInputWidget {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect, emotes: Option<Emotes>) {
        self.input.draw(f, area, emotes);
    }

    fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            match key {
                Key::Enter => {
                    if self.input.is_valid() {
                        self.tx
                            .send(TwitchAction::Privmsg(self.input.to_string()))
                            .unwrap();

                        self.input.update("");
                    }
                }
                Key::Esc => {
                    self.input.toggle_focus();
                }
                _ => {
                    self.input.event(event);
                }
            }
        }

        // if let Some(msg) = input_message.strip_prefix('@') {
        //     app.storage.add("mentions", msg.to_string());
        // }

        // let mut possible_command = String::new();

        // input_message.clone_into(&mut possible_command);

        // input_message.update("", 0);

        // if possible_command.as_str() == "/clear" {
        //     app.clear_messages();
        // }
        // }

        None
    }
}
