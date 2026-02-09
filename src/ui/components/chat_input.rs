use std::fmt::Display;

use color_eyre::Result;
use tokio::sync::mpsc::Sender;
use tui::{Frame, layout::Rect};

use crate::{
    config::SharedCoreConfig,
    emotes::SharedEmotes,
    events::{Event, InternalEvent, TwitchAction, TwitchEvent},
    handlers::storage::SharedStorage,
    ui::{
        components::{Component, EmotePickerWidget, utils::InputWidget},
        statics::{SUPPORTED_COMMANDS, TWITCH_MESSAGE_LIMIT},
    },
    utils::text::first_similarity,
};

pub struct ChatInputWidget {
    config: SharedCoreConfig,
    event_tx: Sender<Event>,
    storage: SharedStorage,
    input: InputWidget<SharedStorage>,
    emote_picker: EmotePickerWidget,
}

impl ChatInputWidget {
    pub fn new(
        config: SharedCoreConfig,
        event_tx: Sender<Event>,
        storage: SharedStorage,
        emotes: SharedEmotes,
    ) -> Self {
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
                            first_similarity(&storage.borrow().get("mentions"), &s[1..]).or_else(
                                || first_similarity(&storage.borrow().get("chatters"), &s[1..]),
                            );

                        let default_suggestion = possible_suggestion.clone();

                        possible_suggestion.map_or(default_suggestion, |s| Some(format!("@{s}")))
                    }
                    _ => None,
                })
        });

        let input = InputWidget::builder()
            .config(config.clone())
            .event_tx(event_tx.clone())
            .title("Chat")
            .input_validator((storage.clone(), input_validator))
            .visual_indicator(visual_indicator)
            .input_suggester((storage.clone(), input_suggester))
            .build();

        let emote_picker = EmotePickerWidget::new(config.clone(), event_tx.clone(), emotes);

        Self {
            config,
            event_tx,
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

    async fn event(&mut self, event: &Event) -> Result<()> {
        if let Event::Internal(InternalEvent::SelectEmote(emote)) = event {
            self.input.insert(emote);
            self.input.insert(" ");
        }

        if self.emote_picker.is_focused() {
            return self.emote_picker.event(event).await;
        }

        if let Event::Input(key) = event {
            let keybinds = self.config.keybinds.insert.clone();
            match key {
                key if keybinds.confirm_text_input.contains(key) => {
                    if self.input.is_valid() {
                        let current_input = self.input.to_string();

                        self.input.clear();

                        if let Some(message) = current_input.strip_prefix('@') {
                            if self.config.storage.mentions {
                                self.storage
                                    .borrow_mut()
                                    .add("mentions", message.to_string());
                            }
                        }

                        self.event_tx
                            .send(Event::Twitch(TwitchEvent::Action(TwitchAction::Message(
                                current_input,
                            ))))
                            .await?;
                    }
                }
                key if keybinds.toggle_emote_picker.contains(key) => {
                    if self.config.frontend.is_emotes_enabled() {
                        self.emote_picker.toggle_focus();
                    }
                }
                key if keybinds.back_to_previous_window.contains(key) => {
                    self.input.toggle_focus();
                }
                _ => {
                    self.input.event(event).await?;
                }
            }
        }

        Ok(())
    }
}
