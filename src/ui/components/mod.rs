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
    handlers::{
        config::SharedCompleteConfig,
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
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Option<Rect>) {
        let component_area = area.map_or_else(|| f.size(), |a| a);

        todo!()
    }

    fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            match key {
                Key::Char('q') => return Some(TerminalAction::Quitting),
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
            chat: ChatWidget::new(config.clone(), tx.clone()),
            dashboard: DashboardWidget::new(config.clone(), storage),
            debug: DebugWidget::new(config.clone(), raw_config),
            help: HelpWidget::new(config.clone()),
            channel_switcher: ChannelSwitcherWidget::new(config.clone(), tx.clone()),
        }
    }
}
