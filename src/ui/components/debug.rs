use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Row, Table},
    Frame,
};

use crate::{
    emotes::Emotes,
    handlers::config::SharedCompleteConfig,
    utils::text::{title_spans, TitleStyle},
};

use super::Component;

#[derive(Debug, Clone)]
pub struct DebugWidget {
    config: SharedCompleteConfig,
    focused: bool,
}

impl DebugWidget {
    pub fn new(config: SharedCompleteConfig) -> Self {
        Self {
            config,
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

impl Component for DebugWidget {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Option<Rect>, _emotes: Option<Emotes>) {
        let area = area.map_or_else(
            || {
                let rect = f.size();

                let new_rect = Rect::new(rect.x, rect.y + 1, rect.width - 1, rect.height - 2);

                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(new_rect)[1]
            },
            |a| a,
        );

        // TODO: Add more debug stuff
        let config = self.config.borrow();

        let rows = vec![Row::new(vec!["Current channel", &config.twitch.channel])];

        let title_binding = [TitleStyle::Single("Debug")];

        let table = Table::new(rows)
            .block(
                Block::default()
                    .title(title_spans(
                        &title_binding,
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_type(self.config.borrow().frontend.border_type.clone().into()),
            )
            .widths(&[Constraint::Length(10), Constraint::Length(10)]);

        f.render_widget(Clear, area);
        f.render_widget(table, area);
    }
}
