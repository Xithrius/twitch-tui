use tokio::sync::broadcast::Sender;
use tui::{backend::Backend, layout::Rect, Frame};

use crate::{
    emotes::Emotes,
    handlers::{
        config::SharedCompleteConfig,
        user_input::{events::Event, input::TerminalAction},
    },
    twitch::TwitchAction,
};

use super::{utils::InputWidget, Component};

#[derive(Debug)]
pub struct ChannelSwitcherWidget {
    _config: SharedCompleteConfig,
    _tx: Sender<TwitchAction>,
    input: InputWidget,
}

impl ChannelSwitcherWidget {
    pub fn new(config: SharedCompleteConfig, tx: Sender<TwitchAction>) -> Self {
        Self {
            _config: config.clone(),
            _tx: tx.clone(),
            input: InputWidget::new(config, tx, "Channel switcher"),
        }
    }
}

impl Component for ChannelSwitcherWidget {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Option<Rect>, emotes: Option<Emotes>) {
        self.input.draw(f, area, emotes);
    }

    fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        self.input.event(event)
    }
}
