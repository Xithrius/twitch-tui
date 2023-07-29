use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Row, Table},
    Frame,
};

use crate::{
    emotes::Emotes,
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

    fn get_config_values(&self) -> Vec<(String, Vec<(String, String)>)> {
        let c = self.config.borrow();

        vec![
            ("Twitch Config".to_string(), c.twitch.to_vec()),
            ("Terminal Config".to_string(), c.terminal.to_vec()),
        ]
    }
}

impl Component for DebugWidget {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, _emotes: Option<&mut Emotes>) {
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

        let table = Table::new(rows)
            .block(
                Block::default()
                    .title(title_line(
                        &title_binding,
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_type(self.config.borrow().frontend.border_type.clone().into()),
            )
            // TODO: Automatically calculate the constraints
            .widths(&[Constraint::Length(20), Constraint::Length(20)]);

        f.render_widget(Clear, area);
        f.render_widget(table, area);
    }

    fn event(&mut self, event: &Event) -> Option<TerminalAction> {
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
