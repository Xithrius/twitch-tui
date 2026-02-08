use tui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    handlers::{config::SharedCoreConfig, user_input::events::Event},
    terminal::TerminalAction,
    ui::components::Component,
    utils::styles::{NO_COLOR, TEXT_DARK_STYLE},
};

#[derive(Debug, Clone)]
pub struct ErrorWidget {
    config: SharedCoreConfig,
    message: Vec<&'static str>,
    focused: bool,
}

impl ErrorWidget {
    pub const fn new(config: SharedCoreConfig, message: Vec<&'static str>) -> Self {
        Self {
            config,
            message,
            focused: false,
        }
    }

    pub const fn is_focused(&self) -> bool {
        self.focused
    }

    pub const fn toggle_focus(&mut self) {
        self.focused = !self.focused;
    }
}

impl Component for ErrorWidget {
    fn draw(&mut self, f: &mut Frame, area: Option<Rect>) {
        let r = area.unwrap_or_else(|| f.area());

        let paragraph = Paragraph::new(
            self.message
                .iter()
                .map(|&s| Line::from(vec![Span::raw(s)]))
                .collect::<Vec<Line>>(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if *NO_COLOR {
                    Style::default()
                } else {
                    Style::default().fg(Color::Red)
                })
                .title_top(Line::from("[ ERROR ]").centered()),
        )
        .style(*TEXT_DARK_STYLE)
        .alignment(Alignment::Center);

        f.render_widget(Clear, r);
        f.render_widget(paragraph, r);
    }
    async fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            let keybinds = self.config.keybinds.selection.clone();
            match key {
                key if keybinds.quit.contains(key) => return Some(TerminalAction::Quit),
                key if keybinds.back_to_previous_window.contains(key) => {
                    return Some(TerminalAction::BackOneLayer);
                }
                _ => {}
            }
        }

        None
    }
}
