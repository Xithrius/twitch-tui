use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    terminal::Frame,
    text::{Line, Span},
    widgets::{block::Title, Block, Borders, Clear, Paragraph},
};

use crate::{emotes::Emotes, ui::components::Component};

#[derive(Debug, Clone)]
pub struct ErrorWidget {
    message: Vec<&'static str>,
    focused: bool,
}

impl ErrorWidget {
    pub const fn new(message: Vec<&'static str>) -> Self {
        Self {
            message,
            focused: false,
        }
    }

    pub const fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn toggle_focus(&mut self) {
        self.focused = !self.focused;
    }
}

impl Component for ErrorWidget {
    fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        area: Option<Rect>,
        _emotes: Option<&mut Emotes>,
    ) {
        let r = area.map_or_else(|| f.size(), |a| a);

        let paragraph = Paragraph::new(
            self.message
                .iter()
                .map(|&s| Line::from(vec![Span::raw(s)]))
                .collect::<Vec<Line>>(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .title(Title::from("[ ERROR ]").alignment(Alignment::Center)),
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);

        f.render_widget(Clear, r);
        f.render_widget(paragraph, r);
    }
}
