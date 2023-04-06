use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::{
    handlers::{app::App, config::CompleteConfig},
    ui::components::dashboard::DASHBOARD_TITLE,
};

const FIRST_N_ITEMS: std::ops::Range<u32> = 0..5;

pub fn render_dashboard_ui<T: Backend>(
    frame: &mut Frame<T>,
    app: &mut App,
    config: &CompleteConfig,
) {
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(DASHBOARD_TITLE.len() as u16),
            Constraint::Min(3),
        ])
        .margin(2)
        .split(frame.size());

    let paragraph = Paragraph::new(
        DASHBOARD_TITLE
            .iter()
            .map(|&s| Spans::from(vec![Span::raw(s)]))
            .collect::<Vec<Spans>>(),
    )
    .block(Block::default().borders(Borders::NONE))
    .style(Style::default().fg(Color::White))
    .alignment(Alignment::Center);

    frame.render_widget(paragraph, v_chunks[0]);

    let items = app
        .storage
        .get("channels")
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            if FIRST_N_ITEMS.contains(&(i as u32)) {
                Some(ListItem::new(s.clone()))
            } else {
                None
            }
        })
        .collect::<Vec<ListItem>>();

    let list = List::new(items)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>");

    frame.render_widget(list, v_chunks[1]);
}
