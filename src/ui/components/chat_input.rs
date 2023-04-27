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
    ui::components::{utils::InputWidget, Component},
};

pub struct ChatInputWidget {
    _config: SharedCompleteConfig,
    tx: Sender<TwitchAction>,
    input: InputWidget,
}

impl ChatInputWidget {
    pub fn new(config: SharedCompleteConfig, tx: Sender<TwitchAction>) -> Self {
        Self {
            _config: config.clone(),
            tx,
            input: InputWidget::new(config, "Chat"),
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
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Option<Rect>, emotes: Option<Emotes>) {
        self.input.draw(f, area, emotes);
    }

    fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            match key {
                Key::Enter => {
                    self.tx
                        .send(TwitchAction::Privmsg(self.input.to_string()))
                        .unwrap();

                    self.input.update("");
                }
                Key::Esc => {
                    self.input.toggle_focus();
                }
                _ => {
                    self.input.event(event);
                }
            }
        }

        None
    }
}
