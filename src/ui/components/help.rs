use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, Row, Table},
    Frame,
};

use crate::{
    ui::statics::{HELP_COLUMN_TITLES, HELP_KEYBINDS},
    utils::styles::COLUMN_TITLE,
};

// Once a solution is found to calculate constraints,
// this will be removed.
const TABLE_CONSTRAINTS: [Constraint; 3] =
    [Constraint::Min(11), Constraint::Min(8), Constraint::Min(38)];

pub fn render_help_window<T: Backend>(f: &mut Frame<T>, area: Rect) {
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
        .header(Row::new(HELP_COLUMN_TITLES.iter().copied()).style(COLUMN_TITLE))
        .block(
            Block::default().borders(Borders::ALL).title("[ Keybinds ]"), // .border_type(frontend.border_type.into()),
        )
        .widths(&TABLE_CONSTRAINTS)
        .column_spacing(2);
    // .style(match app.theme {
    //     Theme::Light => BORDER_NAME_LIGHT,
    //     _ => BORDER_NAME_DARK,
    // });

    f.render_widget(Clear, area);
    f.render_widget(help_table, area);
}
