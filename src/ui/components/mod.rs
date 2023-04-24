mod channel_switcher;
mod chatting;
mod dashboard;
mod debug;
mod error;
mod help;
mod state_tabs;
pub mod utils;

pub use channel_switcher::ChannelSwitcherWidget;
// pub use chatting::render_chat_box;
pub use dashboard::DashboardWidget;
pub use debug::DebugWidget;
pub use error::ErrorWidget;
pub use help::render_help_window;
pub use state_tabs::render_state_tabs;

use tokio::sync::broadcast::Sender;
use toml::Table;
use tui::{backend::Backend, layout::Rect, Frame};

use crate::{
    handlers::{
        config::SharedCompleteConfig,
        storage::{SharedStorage, Storage},
        user_input::{
            events::{Event, Key},
            input::TerminalAction,
        },
    },
    twitch::TwitchAction,
};

pub trait Component {
    #[allow(unused_variables)]
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Option<Rect>) {
        todo!()
    }

    fn event(&mut self, event: Event) -> Option<TerminalAction> {
        if matches!(event, Event::Input(Key::Char('q'))) {
            return Some(TerminalAction::Quitting);
        } else if let Event::Input(key) = event {
            match key {
                _ => todo!(),
            }
        }

        None
    }
}

pub struct Components {
    // Error window(s)
    pub error: ErrorWidget,

    // Full window widgets
    pub dashboard: DashboardWidget,
    // pub chat: ChatWidget,
    pub debug: DebugWidget,

    // Popup widgets
    pub channel_switcher: ChannelSwitcherWidget,
}

impl Components {
    pub fn new(
        config: &SharedCompleteConfig,
        raw_config: Option<Table>,
        tx: Sender<TwitchAction>,
        storage: SharedStorage,
    ) -> Self {
        Self {
            error: ErrorWidget::new(config.clone()),
            dashboard: DashboardWidget::new(config.clone(), storage),
            debug: DebugWidget::new(config.clone(), raw_config),
            channel_switcher: ChannelSwitcherWidget::new(config.clone(), tx),
        }
    }
}
