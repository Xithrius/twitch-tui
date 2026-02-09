use tokio::sync::mpsc::Sender;
use tui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    config::SharedCoreConfig,
    events::{Event, InternalEvent},
    ui::components::Component,
    utils::styles::{NO_COLOR, TEXT_DARK_STYLE},
};

#[derive(Debug, Clone)]
pub struct ErrorWidget {
    config: SharedCoreConfig,
    event_tx: Sender<Event>,
    message: Vec<&'static str>,
    focused: bool,
}

impl ErrorWidget {
    pub const fn new(
        config: SharedCoreConfig,
        event_tx: Sender<Event>,
        message: Vec<&'static str>,
    ) -> Self {
        Self {
            config,
            event_tx,
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

    async fn event(&mut self, event: &Event) -> color_eyre::Result<()> {
        if let Event::Input(key) = event {
            let keybinds = self.config.keybinds.selection.clone();
            match key {
                key if keybinds.quit.contains(key) => {
                    self.event_tx
                        .send(Event::Internal(InternalEvent::Quit))
                        .await?;
                }
                key if keybinds.back_to_previous_window.contains(key) => {
                    self.event_tx
                        .send(Event::Internal(InternalEvent::BackOneLayer))
                        .await?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
