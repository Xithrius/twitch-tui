use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols::DOT,
    text::Spans,
    widgets::{Block, Tabs},
    Frame,
};

use crate::handlers::state::State;

const TABS_TO_RENDER: [State; 3] = [State::Normal(None), State::Help, State::ChannelSwitch];

pub fn render_state_tabs<T: Backend>(f: &mut Frame<T>, area: Rect, current_state: &State) {
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
        .select(
            TABS_TO_RENDER
                .iter()
                .position(|s| s == current_state)
                .unwrap(),
        );

    f.render_widget(tabs, area);
}
