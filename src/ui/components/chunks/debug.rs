use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table},
    Frame,
};

use crate::{
    handlers::{app::DebugWindow, config::FrontendConfig},
    utils::text::{title_spans, TitleStyle},
};

pub fn render_debug_window<T: Backend>(
    frame: &mut Frame<T>,
    area: Rect,
    debug: DebugWindow,
    frontend: FrontendConfig,
) {
    let mut rows = vec![];

    if let Some(mut raw) = debug.raw_config {
        // To avoid getting the user's token leaked in front of others.
        raw.remove("twitch");

        for item in raw.iter() {
            rows.push(Row::new(vec![item.0.to_string()]));
            let inner_map = item.1.as_table();
            if let Some(inner) = inner_map {
                for inner_item in inner.iter() {
                    rows.push(Row::new(vec![
                        " ".to_string(),
                        inner_item.0.to_string(),
                        inner_item.1.to_string(),
                    ]));
                }
            }
        }
    }

    let title_binding = [TitleStyle::Single("Debug")];

    let table = Table::new(rows)
        .block(
            Block::default()
                .title(title_spans(
                    &title_binding,
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(frontend.border_type.into()),
        )
        .widths(&[
            Constraint::Length(10),
            Constraint::Length(25),
            Constraint::Min(50),
        ]);

    frame.render_widget(table, area);
}
