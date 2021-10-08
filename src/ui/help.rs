use anyhow::Result;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    terminal::Frame,
    widgets::{Block, Borders, Row, Table},
};

use crate::utils::{styles, text::vector2_col_max};

pub fn draw_keybinds_ui<T>(frame: &mut Frame<T>) -> Result<()>
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
        vec!["Bring up the chat window", "c"],
        vec!["Keybinds help (this window)", "?"],
        vec![
            "Exit out layer window/entire app when in normal mode",
            "Esc",
        ],
        vec!["Quit this application", "q"],
    ];

    let (maximum_description_width, maximum_keybind_width) = vector2_col_max(&keybinds);

    let column_names = keybinds.remove(0);

    let table_widths = vec![
        Constraint::Min(maximum_description_width),
        Constraint::Min(maximum_keybind_width),
    ];

    let table = Table::new(keybinds.iter().map(|k| Row::new(k.iter().copied())))
        .header(Row::new(column_names.iter().copied()).style(styles::COLUMN_TITLE))
        .block(Block::default().borders(Borders::ALL).title("[ Keybinds ]"))
        .widths(&table_widths)
        .column_spacing(2)
        .style(styles::BORDER_NAME);

    frame.render_widget(table, vertical_chunks[0]);

    Ok(())
}
