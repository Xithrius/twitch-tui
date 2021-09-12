use anyhow::Result;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    terminal::Frame,
    widgets::{Block, Borders, Row, Table},
};

use crate::{
    handlers::config::CompleteConfig,
    utils::{colors::WindowStyles, text::vector2_col_max},
};

pub fn draw_keybinds_ui<T>(frame: &mut Frame<T>, config: CompleteConfig) -> Result<()>
where
    T: Backend,
{
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(frame.size());

    let mut keybinds = vec![
        vec!["Description", "Keybind"],
        vec!["Bring up the chat window", config.keybinds.chat.as_str()],
        vec!["Keybinds help", config.keybinds.help.as_str()],
        vec!["Quit this application", config.keybinds.quit.as_str()],
    ];

    let (maximum_description_width, maximum_keybind_width) = vector2_col_max(keybinds.clone());

    let column_names = keybinds.remove(0);

    let table_widths = vec![
        Constraint::Min(maximum_description_width.clone()),
        Constraint::Min(maximum_keybind_width.clone()),
    ];

    let table = Table::new(
        keybinds
            .iter()
            .map(|k| Row::new(k.clone()))
            .collect::<Vec<Row>>(),
    )
    .header(Row::new(column_names.clone()).style(WindowStyles::new(WindowStyles::ColumnTitle)))
    .block(Block::default().borders(Borders::ALL).title("[ Keybinds ]"))
    .widths(&table_widths)
    .column_spacing(2)
    .style(WindowStyles::new(WindowStyles::BoarderName));

    frame.render_widget(table, vertical_chunks[0]);

    Ok(())
}
