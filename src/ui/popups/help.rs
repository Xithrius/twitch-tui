use tui::{
    backend::Backend,
    layout::Constraint,
    widgets::{Block, Borders, Clear, Row, Table},
};

use crate::{
    ui::{
        popups::{centered_popup, Centering, WindowType},
        statics::{HELP_COLUMN_TITLES, HELP_KEYBINDS},
        WindowAttributes,
    },
    utils::{styles, text::vector_column_max},
};

pub fn ui_show_keybinds<'a: 'b, 'b, 'c, T: Backend>(window: WindowAttributes<'a, 'b, 'c, T>) {
    let WindowAttributes {
        frame,
        app,
        layout: _,
    } = window;

    let table_widths = vector_column_max(&HELP_KEYBINDS)
        .into_iter()
        .map(Constraint::Min)
        .collect::<Vec<Constraint>>();

    let help_table = Table::new(HELP_KEYBINDS.iter().map(|k| Row::new(k.iter().copied())))
        .header(Row::new(HELP_COLUMN_TITLES.iter().copied()).style(styles::COLUMN_TITLE))
        .block(Block::default().borders(Borders::ALL).title("[ Keybinds ]"))
        .widths(&table_widths)
        .column_spacing(2)
        .style(app.theme_style);

    let area = centered_popup(
        WindowType::Window(
            Centering::Middle(frame.size().height),
            HELP_KEYBINDS.len() as u16,
        ),
        frame.size(),
    );

    frame.render_widget(Clear, area);
    frame.render_widget(help_table, area);
}
