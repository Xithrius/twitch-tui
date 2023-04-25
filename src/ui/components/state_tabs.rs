use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols::DOT,
    text::Spans,
    widgets::{Block, Tabs},
    Frame,
};

use crate::handlers::{config::SharedCompleteConfig, state::State};

const TABS_TO_RENDER: [State; 3] = [State::Dashboard, State::Normal, State::Help];

#[derive(Debug, Clone)]
pub struct StateTabsWidget {
    _config: SharedCompleteConfig,
}

impl StateTabsWidget {
    pub fn new(config: SharedCompleteConfig) -> Self {
        Self { _config: config }
    }

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Option<Rect>, state: &State) {
        let tab_titles = TABS_TO_RENDER
            .iter()
            .map(|t| Spans::from(t.to_string()))
            .collect();

        let tabs = Tabs::new(tab_titles)
            .block(Block::default())
            .style(Style::default().fg(Color::Gray).add_modifier(Modifier::DIM))
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
