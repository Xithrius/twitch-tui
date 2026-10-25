use bon::bon;
use tokio::sync::mpsc::Sender;
use tui::{Frame, layout::Rect};

use crate::{
    app::SharedMessages,
    config::SharedCoreConfig,
    emotes::SharedEmotes,
    events::Event,
    handlers::{filters::SharedFilters, storage::SharedStorage},
    twitch::oauth::TwitchOauth,
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
    fn draw(&mut self, f: &mut Frame, area: Option<Rect>) -> color_eyre::Result<()>;

    async fn event(&mut self, event: &Event) -> color_eyre::Result<()> {
        let _ = event;
        Ok(())
    }
}

pub struct Components {
    pub state_tabs: StateTabsWidget,
    // Partial window widgets
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
        twitch_oauth: TwitchOauth,
        event_tx: Sender<Event>,
        storage: SharedStorage,
        filters: SharedFilters,
        messages: SharedMessages,
        emotes: &SharedEmotes,
    ) -> Self {
        let window_size_error = ErrorWidget::new(
            config.clone(),
            event_tx.clone(),
            WINDOW_SIZE_TOO_SMALL_ERROR.to_vec(),
        );

        Self {
            state_tabs: StateTabsWidget::new(config),

            debug: DebugWidget::new(config.clone(), event_tx.clone()),

            chat: ChatWidget::new(
                config.clone(),
                twitch_oauth.clone(),
                event_tx.clone(),
                messages,
                &storage,
                emotes,
                filters,
            ),
            dashboard: DashboardWidget::new(
                config.clone(),
                twitch_oauth,
                event_tx.clone(),
                storage,
            ),
            help: HelpWidget::new(config.clone(), event_tx),
            window_size_error,
        }
    }
}
