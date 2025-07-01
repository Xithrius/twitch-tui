use std::fmt::Display;

use tui::{Frame, layout::Rect};

use crate::{
    emotes::SharedEmotes,
    handlers::{
        config::SharedCoreConfig,
        storage::SharedStorage,
        user_input::events::{Event, Key},
    },
    terminal::TerminalAction,
    twitch::TwitchAction,
    ui::{
        components::{Component, emote_picker::EmotePickerWidget, utils::InputWidget},
        statics::{SUPPORTED_COMMANDS, TWITCH_MESSAGE_LIMIT},
    },
    utils::text::first_similarity,
};

pub struct ChatInputWidget {
    config: SharedCoreConfig,
    storage: SharedStorage,
    input: InputWidget<SharedStorage>,
    emote_picker: EmotePickerWidget,
}

impl ChatInputWidget {
    pub fn new(config: SharedCoreConfig, storage: SharedStorage, emotes: SharedEmotes) -> Self {
        let input_validator = Box::new(|_, s: String| -> bool {
            {
                if !s.is_empty() && s.len() < TWITCH_MESSAGE_LIMIT {
                    s.strip_prefix('/').is_none_or(|command| {
                        let command = command.split(' ').next().unwrap_or("");
                        SUPPORTED_COMMANDS.contains(&command)
                    })
                } else {
                    false
                }
            }
        });

        // User should be known of how close they are to the message length limit.
        let visual_indicator =
            Box::new(|s: String| -> String { format!("{} / {}", s.len(), TWITCH_MESSAGE_LIMIT) });

        let input_suggester = Box::new(|storage: SharedStorage, s: String| -> Option<String> {
            s.chars()
                .next()
                .and_then(|start_character| match start_character {
                    '/' => {
                        let possible_suggestion = first_similarity(
                            &SUPPORTED_COMMANDS
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<String>>(),
                            &s[1..],
                        );

                        let default_suggestion = possible_suggestion.clone();

                        possible_suggestion.map_or(default_suggestion, |s| Some(format!("/{s}")))
                    }
                    '@' => {
                        let possible_suggestion =
                            first_similarity(&storage.borrow().get("mentions"), &s[1..]);

                        let default_suggestion = possible_suggestion.clone();

                        possible_suggestion.map_or(default_suggestion, |s| Some(format!("@{s}")))
                    }
                    _ => None,
                })
        });

        let input = InputWidget::new(
            config.clone(),
            "Chat",
            Some((storage.clone(), input_validator)),
            Some(visual_indicator),
            Some((storage.clone(), input_suggester)),
        );

        let emote_picker = EmotePickerWidget::new(config.clone(), emotes);

        Self {
            config,
            storage,
            input,
            emote_picker,
        }
    }

    pub const fn is_focused(&self) -> bool {
        self.input.is_focused()
    }

    pub const fn toggle_focus(&mut self) {
        self.input.toggle_focus();
    }

    pub fn toggle_focus_with(&mut self, s: &str) {
        self.input.toggle_focus_with(s);
    }
}

impl Display for ChatInputWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.input)
    }
}

impl Component for ChatInputWidget {
    fn draw(&mut self, f: &mut Frame, area: Option<Rect>) {
        self.input.draw(f, area);

        if self.emote_picker.is_focused() {
            self.emote_picker.draw(f, None);
        }
    }

    async fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if self.emote_picker.is_focused() {
            if let Some(TerminalAction::Enter(TwitchAction::Message(emote))) =
                self.emote_picker.event(event).await
            {
                self.input.insert(&emote);
                self.input.insert(" ");
            }
        } else if let Event::Input(key) = event {
            match key {
                Key::Enter => {
                    if self.input.is_valid() {
                        let current_input = self.input.to_string();

                        let action =
                            TerminalAction::Enter(TwitchAction::Message(current_input.clone()));

                        self.input.clear();

                        if let Some(message) = current_input.strip_prefix('@') {
                            if self.config.borrow().storage.mentions {
                                self.storage
                                    .borrow_mut()
                                    .add("mentions", message.to_string());
                            }
                        }

                        return Some(action);
                    }
                }
                Key::Alt('e') => {
                    if self.config.borrow().frontend.is_emotes_enabled() {
                        self.emote_picker.toggle_focus();
                    }
                }
                Key::Esc => {
                    self.input.toggle_focus();
                }
                _ => {
                    self.input.event(event).await;
                }
            }
        }

        None
    }
}
