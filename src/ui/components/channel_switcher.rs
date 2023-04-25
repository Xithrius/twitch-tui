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
};

use super::{utils::InputWidget, Component};

pub struct ChannelSwitcherWidget {
    _config: SharedCompleteConfig,
    input: InputWidget,
    focused: bool,
}

impl ChannelSwitcherWidget {
    pub fn new(config: SharedCompleteConfig) -> Self {
        Self {
            _config: config.clone(),
            input: InputWidget::new(config, "Channel switcher"),
            focused: false,
        }
    }

    pub const fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn toggle_focus(&mut self) {
        self.focused = !self.focused;
    }
}

impl ToString for ChannelSwitcherWidget {
    fn to_string(&self) -> String {
        self.input.to_string()
    }
}

impl Component for ChannelSwitcherWidget {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Option<Rect>, emotes: Option<Emotes>) {
        self.input.draw(f, area, emotes);
    }

    fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            match key {
                Key::Esc => self.toggle_focus(),
                Key::Enter => {
                    return None;
                }
                _ => return self.input.event(event),
            }
        }

        None
    }
}
