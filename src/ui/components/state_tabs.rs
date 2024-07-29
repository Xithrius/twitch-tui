use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols::DOT,
    text::Line,
    widgets::{Block, Tabs},
    Frame,
};

use crate::{
    handlers::{config::SharedCompleteConfig, state::State},
    utils::{styles::STATE_TABS_STYLE, text::capitalize_first_char},
};

const TABS_TO_RENDER: [State; 3] = [State::Dashboard, State::Normal, State::Help];

#[derive(Debug, Clone)]
pub struct StateTabsWidget {
    _config: SharedCompleteConfig,
}

impl StateTabsWidget {
    pub fn new(config: SharedCompleteConfig) -> Self {
        Self { _config: config }
    }

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
