use color_eyre::Result;
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
    ui::components::Component,
    utils::{styles::STATE_TABS_STYLE, text::capitalize_first_char},
};

const STATE_TABS: [State; 3] = [State::Dashboard, State::Normal, State::Help];

#[derive(Debug, Clone, Default)]
pub struct StateTabsWidget {
    selected_tab: usize,
}

impl StateTabsWidget {
    pub fn new(config: &SharedCoreConfig) -> Self {
        let selected_tab = Self::get_state_index(&config.terminal.first_state);

        Self { selected_tab }
    }

    fn get_state_index(state: &State) -> usize {
        STATE_TABS
            .iter()
            .position(|s| s == state)
            .unwrap_or_default()
    }

    pub fn select(&mut self, state: &State) {
        self.selected_tab = Self::get_state_index(state);
    }
}

impl Component for StateTabsWidget {
    fn draw(&mut self, f: &mut Frame, area: Option<Rect>) -> Result<()> {
        let r = area.unwrap_or_else(|| f.area());

        let tab_titles = STATE_TABS
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
            .select(self.selected_tab);

        f.render_widget(tabs, r);

        Ok(())
    }
}
