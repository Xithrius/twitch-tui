use regex::Regex;
use tokio::sync::broadcast::Sender;
use tui::{backend::Backend, layout::Rect, Frame};

use crate::{
    emotes::Emotes,
    handlers::{
        config::SharedCompleteConfig,
        state::State,
        user_input::{
            events::{Event, Key},
            input::TerminalAction,
        },
    },
    twitch::TwitchAction,
    ui::{
        components::{utils::InputWidget, Component},
        statics::NAME_RESTRICTION_REGEX,
    },
};

pub struct ChannelSwitcherWidget {
    _config: SharedCompleteConfig,
    input: InputWidget,
    tx: Sender<TwitchAction>,
}

impl ChannelSwitcherWidget {
    pub fn new(config: SharedCompleteConfig, tx: Sender<TwitchAction>) -> Self {
        let input = InputWidget::new(
            config.clone(),
            "Channel switcher",
            Some(Box::new(|s: String| -> bool {
                Regex::new(&NAME_RESTRICTION_REGEX)
                    .unwrap()
                    .is_match(s.as_str())
            })),
        );

        Self {
            _config: config,
            input,
            tx,
        }
    }

    pub const fn is_focused(&self) -> bool {
        self.input.is_focused()
    }

    pub fn toggle_focus(&mut self) {
        self.input.toggle_focus();
    }
}

impl ToString for ChannelSwitcherWidget {
    fn to_string(&self) -> String {
        self.input.to_string()
    }
}

impl Component for ChannelSwitcherWidget {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect, emotes: Option<Emotes>) {
        self.input.draw(f, area, emotes);
    }

    fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            match key {
                Key::Enter => {
                    if self.input.is_valid() {
                        self.tx
                            .send(TwitchAction::Join(self.input.to_string()))
                            .unwrap();

                        self.input.toggle_focus();

                        return Some(TerminalAction::SwitchState(State::Normal));
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

        None
    }
}
