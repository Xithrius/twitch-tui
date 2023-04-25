mod channel_switcher;
mod chat;
mod dashboard;
mod debug;
mod error;
mod help;
mod state_tabs;
pub mod utils;

pub use channel_switcher::ChannelSwitcherWidget;
pub use chat::ChatWidget;
pub use dashboard::DashboardWidget;
pub use debug::DebugWidget;
pub use error::ErrorWidget;
pub use help::HelpWidget;
pub use state_tabs::render_state_tabs;

use tokio::sync::broadcast::Sender;
use toml::Table;
use tui::{backend::Backend, layout::Rect, Frame};

use crate::{
    emotes::{Emotes, SharedEmotes},
    handlers::{
        app::SharedMessages,
        config::SharedCompleteConfig,
        filters::SharedFilters,
        storage::SharedStorage,
        user_input::{
            events::{Event, Key},
            input::TerminalAction,
        },
    },
    twitch::TwitchAction,
};

pub trait Component {
    #[allow(unused_variables)]
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Option<Rect>, emotes: Option<Emotes>) {
        let component_area = area.map_or_else(|| f.size(), |a| a);

        todo!()
    }

    fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            match key {
                Key::Char('q') => return Some(TerminalAction::Quit),
                Key::Esc => return Some(TerminalAction::BackOneLayer),
                Key::Ctrl('p') => panic!("Manual panic triggered by user."),
                _ => {}
            }
        }

        None
    }
}

pub struct Components {
    // Error window(s)
    pub error: ErrorWidget,

    // Full window widgets
    pub chat: ChatWidget,
    pub dashboard: DashboardWidget,
    pub debug: DebugWidget,
    pub help: HelpWidget,
}

impl Components {
    pub fn new(
        config: &SharedCompleteConfig,
        tx: Sender<TwitchAction>,
        storage: SharedStorage,
        filters: SharedFilters,
        messages: SharedMessages,
    ) -> Self {
        Self {
            error: ErrorWidget::new(config.clone()),
            chat: ChatWidget::new(config.clone(), tx.clone(), messages, filters),
            dashboard: DashboardWidget::new(config.clone(), tx, storage),
            debug: DebugWidget::new(config.clone()),
            help: HelpWidget::new(config.clone()),
        }
    }
}
