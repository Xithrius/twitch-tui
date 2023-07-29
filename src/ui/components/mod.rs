mod channel_switcher;
mod chat;
mod chat_input;
mod dashboard;
mod debug;
mod error;
mod following;
mod help;
mod message_search;
mod state_tabs;

pub mod utils;

pub use channel_switcher::ChannelSwitcherWidget;
pub use chat::ChatWidget;
pub use chat_input::ChatInputWidget;
use chrono::{DateTime, Local};
pub use dashboard::DashboardWidget;
pub use debug::DebugWidget;
pub use error::ErrorWidget;
pub use help::HelpWidget;
pub use message_search::MessageSearchWidget;
pub use state_tabs::StateTabsWidget;

use tui::{backend::Backend, layout::Rect, Frame};

use crate::{
    emotes::Emotes,
    handlers::{
        app::SharedMessages,
        config::SharedCompleteConfig,
        filters::SharedFilters,
        storage::SharedStorage,
        user_input::events::{Event, Key},
    },
    terminal::TerminalAction,
    twitch::oauth::FollowingList,
};

pub trait Component {
    #[allow(unused_variables)]
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, emotes: Option<&mut Emotes>) {
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
    // Partial window widgets
    pub tabs: StateTabsWidget,
    pub debug: DebugWidget,

    // Full window widgets
    pub chat: ChatWidget,
    pub dashboard: DashboardWidget,
    pub help: HelpWidget,
    pub error: ErrorWidget,
}

impl Components {
    pub fn new(
        config: &SharedCompleteConfig,
        storage: SharedStorage,
        filters: SharedFilters,
        messages: SharedMessages,
        following: FollowingList,
        startup_time: DateTime<Local>,
    ) -> Self {
        Self {
            tabs: StateTabsWidget::new(config.clone()),
            debug: DebugWidget::new(config.clone(), startup_time),

            chat: ChatWidget::new(
                config.clone(),
                messages,
                &storage,
                filters,
                following.clone(),
            ),
            dashboard: DashboardWidget::new(config.clone(), storage, following),
            help: HelpWidget::new(config.clone()),
            error: ErrorWidget::new(),
        }
    }
}
