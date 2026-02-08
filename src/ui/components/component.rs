use bon::bon;
use tui::{Frame, layout::Rect};

use crate::{
    config::SharedCoreConfig,
    emotes::SharedEmotes,
    events::{Event, InternalEvent},
    handlers::{context::SharedMessages, filters::SharedFilters, storage::SharedStorage},
    ui::components::{
        ChatWidget, DashboardWidget, DebugWidget, ErrorWidget, HelpWidget, StateTabsWidget,
    },
};

static WINDOW_SIZE_TOO_SMALL_ERROR: &[&str] = &[
    "Window too small!",
    "Must allow for at least 60x10.",
    "Restart and resize.",
];

pub trait Component {
    fn draw(&mut self, f: &mut Frame, area: Option<Rect>);

    #[allow(clippy::unused_async)]
    async fn event(&mut self, event: &Event) -> Option<InternalEvent>;
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

#[bon]
impl Components {
    #[builder]
    pub fn new(
        config: &SharedCoreConfig,
        storage: SharedStorage,
        filters: SharedFilters,
        messages: SharedMessages,
        emotes: &SharedEmotes,
    ) -> Self {
        let window_size_error =
            ErrorWidget::new(config.clone(), WINDOW_SIZE_TOO_SMALL_ERROR.to_vec());

        Self {
            tabs: StateTabsWidget::new(config.clone()),
            debug: DebugWidget::new(config.clone()),

            chat: ChatWidget::new(config.clone(), messages, &storage, emotes, filters),
            dashboard: DashboardWidget::new(config.clone(), storage),
            help: HelpWidget::new(config.clone()),
            window_size_error,
        }
    }
}
