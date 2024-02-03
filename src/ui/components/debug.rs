use chrono::{DateTime, Local};
use tui::{
    layout::{Constraint, Rect},
    prelude::Alignment,
    style::{Color, Modifier, Style},
    widgets::{block::Position, Block, Borders, Clear, Row, Table},
    Frame,
};

use crate::{
    handlers::{
        config::{SharedCompleteConfig, ToVec},
        user_input::events::{Event, Key},
    },
    terminal::TerminalAction,
    ui::components::Component,
    utils::text::{title_line, TitleStyle},
};

#[derive(Debug, Clone)]
pub struct DebugWidget {
    config: SharedCompleteConfig,
    focused: bool,
    startup_time: DateTime<Local>,
}

impl DebugWidget {
    pub fn new(config: SharedCompleteConfig, startup_time: DateTime<Local>) -> Self {
        Self {
            config,
            focused: false,
            startup_time,
        }
    }

    pub const fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn toggle_focus(&mut self) {
        self.focused = !self.focused;
    }

    fn get_config_values(&self) -> Vec<(String, Vec<(String, String)>)> {
        let c = self.config.borrow();

        vec![
            ("Twitch Config".to_string(), c.twitch.to_vec()),
            ("Terminal Config".to_string(), c.terminal.to_vec()),
            ("Storage Config".to_string(), c.storage.to_vec()),
            ("Filter Config".to_string(), c.filters.to_vec()),
            ("Frontend Config".to_string(), c.frontend.to_vec()),
        ]
    }
}

impl Component for DebugWidget {
    fn draw(&mut self, f: &mut Frame, area: Option<Rect>) {
        let r = area.map_or_else(|| f.size(), |a| a);

        let configs = self.get_config_values();

        let rows = configs
            .iter()
            .enumerate()
            .flat_map(|(i, (t, values))| {
                let mut inner_rows = if i > 0 {
                    vec![
                        Row::new::<Vec<String>>(vec![]),
                        Row::new(vec![t.to_string()])
                            .style(Style::default().add_modifier(Modifier::BOLD)),
                    ]
                } else {
                    vec![Row::new(vec![t.to_string()])
                        .style(Style::default().add_modifier(Modifier::BOLD))]
                };

                for (k, v) in values {
                    inner_rows.push(Row::new(vec![k.to_string(), v.to_string()]));
                }

                inner_rows
            })
            .collect::<Vec<Row>>();

        let title_binding = [TitleStyle::Single("Debug")];

        let table = Table::new(rows, &[Constraint::Length(25), Constraint::Length(25)]).block(
            Block::default()
                .title(title_line(
                    &title_binding,
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(self.config.borrow().frontend.border_type.clone().into()),
        );

        f.render_widget(Clear, r);
        f.render_widget(table, r);

        let title_binding = self
            .startup_time
            .format(&self.config.borrow().frontend.datetime_format)
            .to_string();

        let title = [TitleStyle::Combined("Startup time", &title_binding)];

        let bottom_block = Block::default()
            .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
            .border_type(self.config.borrow().frontend.border_type.clone().into())
            .title(title_line(
                &title,
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
            ))
            .title_position(Position::Bottom)
            .title_alignment(Alignment::Left);

        let rect = Rect::new(r.x, r.bottom() - 1, r.width, 1);

        f.render_widget(bottom_block, rect);
    }

    async fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            match key {
                Key::Char('q') => return Some(TerminalAction::Quit),
                Key::Esc => {
                    self.toggle_focus();

                    return Some(TerminalAction::BackOneLayer);
                }
                Key::Ctrl('p') => panic!("Manual panic triggered by user."),
                _ => {}
            }
        }

        None
    }
}
