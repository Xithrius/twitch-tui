use regex::Regex;
use tokio::sync::broadcast::Sender;
use tui::{backend::Backend, layout::Rect, Frame};

use crate::{
    emotes::Emotes,
    handlers::{
        config::SharedCompleteConfig,
        state::State,
        storage::SharedStorage,
        user_input::events::{Event, Key},
    },
    terminal::TerminalAction,
    twitch::TwitchAction,
    ui::{
        components::{utils::InputWidget, Component},
        statics::NAME_RESTRICTION_REGEX,
    },
    utils::text::first_similarity,
};

pub struct ChannelSwitcherWidget {
    config: SharedCompleteConfig,
    storage: SharedStorage,
    input: InputWidget,
    tx: Sender<TwitchAction>,
}

impl ChannelSwitcherWidget {
    pub fn new(
        config: SharedCompleteConfig,
        tx: Sender<TwitchAction>,
        storage: SharedStorage,
    ) -> Self {
        let input_validator = Box::new(|s: String| -> bool {
            Regex::new(&NAME_RESTRICTION_REGEX)
                .unwrap()
                .is_match(s.as_str())
        });

        let input_suggester = Box::new(|storage: SharedStorage, s: String| -> Option<String> {
            first_similarity(
                &storage
                    .borrow()
                    .get("channels")
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>(),
                &s,
            )
        });

        let input = InputWidget::new(
            config.clone(),
            "Channel switcher",
            Some(input_validator),
            Some((storage.clone(), input_suggester)),
        );

        Self {
            config,
            storage,
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
                        let current_input = self.input.to_string();

                        self.tx
                            .send(TwitchAction::Join(current_input.clone()))
                            .unwrap();

                        self.input.toggle_focus();

                        if self.config.borrow().storage.channels {
                            self.storage
                                .borrow_mut()
                                .add("channels", current_input.clone());
                        }

                        self.config.borrow_mut().twitch.channel = current_input;

                        self.input.update("");

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
