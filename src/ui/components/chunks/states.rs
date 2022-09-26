use tui::{
    backend::Backend,
    style::{Color, Style},
    symbols::DOT,
    text::Spans,
    widgets::{Block, Tabs},
};

use crate::{handlers::app::State, ui::WindowAttributes};

const TABS_TO_RENDER: [State; 5] = [
    State::Normal,
    State::Insert,
    State::Help,
    State::ChannelSwitch,
    State::MessageSearch,
];

pub fn render_state_tabs<T: Backend>(window: WindowAttributes<T>) {
    let WindowAttributes {
        frame,
        app: _,
        layout,
    } = window;

    let tab_titles = TABS_TO_RENDER
        .iter()
        .map(|t| Spans::from(t.to_string()))
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(Block::default())
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow))
        .divider(DOT);

    frame.render_widget(tabs, layout.last_chunk());
}
