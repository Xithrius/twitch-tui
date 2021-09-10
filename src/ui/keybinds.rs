use anyhow::Result;
use tui::backend::Backend;
use tui::terminal::Frame;
use tui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Row, Table},
};

use crate::handlers::config::CompleteConfig;

pub fn draw_keybinds_ui<T>(frame: &mut Frame<T>, config: CompleteConfig) -> Result<()>
where
    T: Backend,
{
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints([Constraint::Min(10)].as_ref())
        .split(frame.size());

    let keybinds = vec![
        ("Bring up the chat window", config.keybinds.chat),
        ("Bring up the table for keybinds", config.keybinds.keybinds),
        ("See all the users in chat", config.keybinds.users),
        ("Quit this application", config.keybinds.quit),
    ];

    let table = Table::new(vec![Row::new(vec![])])
        .block(Block::default().borders(Borders::ALL).title("[ Keybinds ]"))
        .column_spacing(1);

    frame.render_widget(table, vertical_chunks[0]);

    Ok(())
}
