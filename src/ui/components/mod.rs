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

mod emote_picker;
pub mod utils;

use std::sync::LazyLock;

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
use tui::{Frame, layout::Rect};

use crate::{
    emotes::SharedEmotes,
    handlers::{
        app::SharedMessages,
        config::SharedCoreConfig,
        filters::SharedFilters,
        storage::SharedStorage,
        user_input::events::{Event, Key},
    },
    terminal::TerminalAction,
};

static WINDOW_SIZE_TOO_SMALL_ERROR: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "Window too small!",
        "Must allow for at least 60x10.",
        "Restart and resize.",
    ]
});

pub trait Component {
    #[allow(unused_variables)]
    fn draw(&mut self, f: &mut Frame, area: Option<Rect>) {
        todo!()
    }

    #[allow(clippy::unused_async)]
    async fn event(&mut self, event: &Event) -> Option<TerminalAction> {
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

    // Errors
    pub window_size_error: ErrorWidget,
}

impl Components {
    pub fn new(
        config: &SharedCoreConfig,
        storage: SharedStorage,
        filters: SharedFilters,
        messages: SharedMessages,
        emotes: &SharedEmotes,
        startup_time: DateTime<Local>,
    ) -> Self {
        let window_size_error = ErrorWidget::new(WINDOW_SIZE_TOO_SMALL_ERROR.to_vec());

        Self {
            tabs: StateTabsWidget::new(config.clone()),
            debug: DebugWidget::new(config.clone(), startup_time),

            chat: ChatWidget::new(config.clone(), messages, &storage, emotes, filters),
            dashboard: DashboardWidget::new(config.clone(), storage),
            help: HelpWidget::new(config.clone()),
            window_size_error,
        }
    }
}
