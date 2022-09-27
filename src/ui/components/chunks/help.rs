use tui::{
    backend::Backend,
    layout::Constraint,
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, Row, Table},
};

use crate::{
    ui::{
        statics::{HELP_COLUMN_TITLES, HELP_KEYBINDS},
        WindowAttributes,
    },
    utils::styles,
};

// Once a solution is found to calculate constraints,
// this will be removed.
const TABLE_CONSTRAINTS: [Constraint; 3] =
    [Constraint::Min(11), Constraint::Min(8), Constraint::Min(38)];

pub fn render_help_window<T: Backend>(window: WindowAttributes<T>) {
    let WindowAttributes { frame, app, layout } = window;

    let mut rows = vec![];

    for (s, v) in HELP_KEYBINDS.iter() {
        for (i, (key, desc)) in v.iter().enumerate() {
            rows.push(Row::new(vec![
                if i == 0 {
                    Cell::from(s.category()).style(Style::default().add_modifier(Modifier::BOLD))
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
        .header(Row::new(HELP_COLUMN_TITLES.iter().copied()).style(styles::COLUMN_TITLE))
        .block(Block::default().borders(Borders::ALL).title("[ Keybinds ]"))
        .widths(&TABLE_CONSTRAINTS)
        .column_spacing(2)
        .style(app.theme_style);

    frame.render_widget(Clear, layout.first_chunk());
    frame.render_widget(help_table, layout.first_chunk());
}
