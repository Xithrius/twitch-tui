use tui::{
    backend::Backend,
    layout::Constraint,
    terminal::Frame,
    widgets::{Block, Borders, Clear, Row, Table},
};

use crate::{
    ui::{
        popups::{centered_popup, Centering},
        statics::{HELP_COLUMN_TITLES, HELP_KEYBINDS},
    },
    utils::{styles, text::vector_column_max},
};

pub fn keybinds<T: Backend>(frame: &mut Frame<T>) {
    let table_widths = vector_column_max(&HELP_KEYBINDS, None)
        .into_iter()
        .map(Constraint::Min)
        .collect::<Vec<Constraint>>();

    let help_table = Table::new(HELP_KEYBINDS.iter().map(|k| Row::new(k.iter().copied())))
        .header(Row::new(HELP_COLUMN_TITLES.iter().copied()).style(styles::COLUMN_TITLE))
        .block(Block::default().borders(Borders::ALL).title("[ Keybinds ]"))
        .widths(&table_widths)
        .column_spacing(2)
        .style(styles::BORDER_NAME);

    let area = centered_popup(Centering::Window(50, 50), frame.size());

    frame.render_widget(Clear, area);
    frame.render_widget(help_table, area);
}
