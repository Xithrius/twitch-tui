use anyhow::Result;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    terminal::Frame,
    widgets::{Block, Borders, Row, Table},
};

use crate::{
    ui::keys::{COLUMN_TITLES, INSERT_MODE, NORMAL_MODE},
    utils::{styles, text::vector_column_max},
};

pub fn draw_keybinds_ui<T>(frame: &mut Frame<T>) -> Result<()>
where
    T: Backend,
{
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(frame.size());

    // Normal mode keybinds
    let normal_table_widths = vector_column_max(&INSERT_MODE, None)
        .into_iter()
        .map(|w| Constraint::Min(w))
        .collect::<Vec<Constraint>>();

    let normal_mode_table = Table::new(NORMAL_MODE.iter().map(|k| Row::new(k.iter().copied())))
        .header(Row::new(COLUMN_TITLES.iter().copied()).style(styles::COLUMN_TITLE))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("[ Normal Mode Keybinds ]"),
        )
        .widths(&normal_table_widths)
        .column_spacing(2)
        .style(styles::BORDER_NAME);

    frame.render_widget(normal_mode_table, vertical_chunks[0]);

    // Insert mode keybinds
    let insert_table_widths = vector_column_max(&INSERT_MODE, None)
        .into_iter()
        .map(|w| Constraint::Min(w))
        .collect::<Vec<Constraint>>();

    let insert_mode_table = Table::new(INSERT_MODE.iter().map(|k| Row::new(k.iter().copied())))
        .header(Row::new(COLUMN_TITLES.iter().copied()).style(styles::COLUMN_TITLE))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("[ Insert Mode Keybinds ]"),
        )
        .widths(&insert_table_widths)
        .column_spacing(2)
        .style(styles::BORDER_NAME);

    frame.render_widget(insert_mode_table, vertical_chunks[1]);

    Ok(())
}
