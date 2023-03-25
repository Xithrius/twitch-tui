use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};

use crate::ui::components::dashboard::DASHBOARD_TITLE;

pub fn render_dashboard_ui<T: Backend>(frame: &mut Frame<T>) {
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(4)])
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
}
