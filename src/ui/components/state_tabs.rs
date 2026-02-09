use tui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols::DOT,
    text::Line,
    widgets::{Block, Tabs},
};

use crate::{
    config::SharedCoreConfig,
    handlers::state::State,
    utils::{styles::STATE_TABS_STYLE, text::capitalize_first_char},
};

const TABS_TO_RENDER: [State; 3] = [State::Dashboard, State::Normal, State::Help];

#[derive(Debug, Clone)]
pub struct StateTabsWidget {
    _config: SharedCoreConfig,
}

impl StateTabsWidget {
    pub const fn new(config: SharedCoreConfig) -> Self {
        Self { _config: config }
    }

    #[allow(clippy::unused_self)]
    pub fn draw(&self, f: &mut Frame, area: Option<Rect>, state: &State) {
        let tab_titles = TABS_TO_RENDER
            .iter()
            .map(|t| Line::from(capitalize_first_char(&t.to_string())))
            .collect::<Vec<Line>>();

        let tabs = Tabs::new(tab_titles)
            .block(Block::default())
            .style(*STATE_TABS_STYLE)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .remove_modifier(Modifier::DIM)
                    .add_modifier(Modifier::UNDERLINED),
            )
            .divider(DOT)
            .select(TABS_TO_RENDER.iter().position(|s| s == state).unwrap());

        f.render_widget(tabs, area.unwrap());
    }
}
