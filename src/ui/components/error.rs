use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};

use crate::{emotes::Emotes, handlers::config::SharedCompleteConfig};

use super::Component;

const WINDOW_SIZE_ERROR_MESSAGE: [&str; 3] = [
    "Window to small!",
    "Must allow for at least 60x10.",
    "Restart and resize.",
];

#[derive(Debug, Clone)]
pub struct ErrorWidget {
    config: SharedCompleteConfig,
}

impl ErrorWidget {
    pub const fn new(config: SharedCompleteConfig) -> Self {
        Self { config }
    }
}

impl Component for ErrorWidget {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Option<Rect>, emotes: Option<Emotes>) {
        let area = area.map_or_else(|| f.size(), |a| a);

        let paragraph = Paragraph::new(
            WINDOW_SIZE_ERROR_MESSAGE
                .iter()
                .map(|&s| Spans::from(vec![Span::raw(s)]))
                .collect::<Vec<Spans>>(),
        )
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }
}
