use std::fmt::Display;

use tui::{Frame, layout::Rect};

use crate::{
    handlers::{
        config::SharedCoreConfig,
        user_input::events::{Event, Key},
    },
    terminal::TerminalAction,
    ui::{
        components::{Component, utils::InputWidget},
        statics::TWITCH_MESSAGE_LIMIT,
    },
};

pub struct MessageSearchWidget {
    input: InputWidget<()>,
}

impl MessageSearchWidget {
    pub fn new(config: SharedCoreConfig) -> Self {
        let input_validator =
            Box::new(|(), s: String| -> bool { !s.is_empty() && s.len() <= TWITCH_MESSAGE_LIMIT });

        // Indication that user won't get any good results near the twitch message length limit.
        // TODO: In the future, this should be replaced with how many results have been found.
        let visual_indicator =
            Box::new(|s: String| -> String { format!("{} / {}", s.len(), TWITCH_MESSAGE_LIMIT) });

        let input = InputWidget::builder()
            .config(config)
            .title("Message search")
            .input_validator(((), input_validator))
            .visual_indicator(visual_indicator)
            .build();

        Self { input }
    }

    pub const fn is_focused(&self) -> bool {
        self.input.is_focused()
    }

    pub const fn toggle_focus(&mut self) {
        self.input.toggle_focus();
    }
}

impl Display for MessageSearchWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.input)
    }
}

impl Component for MessageSearchWidget {
    fn draw(&mut self, f: &mut Frame, area: Option<Rect>) {
        self.input.draw(f, area);
    }

    async fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            match key {
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
