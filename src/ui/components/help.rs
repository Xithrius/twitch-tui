use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::{
    emotes::Emotes,
    handlers::config::SharedCompleteConfig,
    ui::{
        components::Component,
        statics::{HELP_COLUMN_TITLES, HELP_KEYBINDS},
    },
    utils::styles::COLUMN_TITLE,
};

// Once a solution is found to calculate constraints, this will be removed.
const TABLE_CONSTRAINTS: [Constraint; 3] =
    [Constraint::Min(11), Constraint::Min(8), Constraint::Min(38)];

#[derive(Debug, Clone)]
pub struct HelpWidget {
    config: SharedCompleteConfig,
}

impl HelpWidget {
    pub fn new(config: SharedCompleteConfig) -> Self {
        Self { config }
    }
}

impl Component for HelpWidget {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, _emotes: Option<&mut Emotes>) {
        let mut rows = vec![];

        for (s, v) in HELP_KEYBINDS.iter() {
            for (i, (key, desc)) in v.iter().enumerate() {
                rows.push(Row::new(vec![
                    if i == 0 {
                        Cell::from((*s).to_string())
                            .style(Style::default().add_modifier(Modifier::BOLD))
                    } else {
                        Cell::from("")
                    },
                    Cell::from((*key).to_string()),
                    Cell::from((*desc).to_string()),
                ]));
            }

            rows.push(Row::new(vec![Cell::from("")]));
        }

        let help_table = Table::new(rows)
            .header(Row::new(HELP_COLUMN_TITLES.iter().copied()).style(COLUMN_TITLE))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("[ Keybinds ]")
                    .border_type(self.config.borrow().frontend.border_type.clone().into()),
            )
            .widths(&TABLE_CONSTRAINTS)
            .column_spacing(2);
        // .style(match app.theme {
        //     Theme::Light => BORDER_NAME_LIGHT,
        //     _ => BORDER_NAME_DARK,
        // });

        f.render_widget(help_table, area);
    }
}
