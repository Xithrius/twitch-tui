use tui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    symbols::DOT,
    text::Spans,
    widgets::{Block, Tabs},
    Frame,
};

use crate::{handlers::state::State, ui::LayoutAttributes};

const TABS_TO_RENDER: [State; 5] = [
    State::Normal,
    State::Insert,
    State::Help,
    State::ChannelSwitch,
    State::MessageSearch,
];

pub fn render_state_tabs<T: Backend>(
    frame: &mut Frame<T>,
    layout: &LayoutAttributes,
    current_state: &State,
) {
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

    frame.render_widget(tabs, layout.last_chunk());
}
