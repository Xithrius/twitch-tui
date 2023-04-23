use toml::Table as TomlTable;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table},
    Frame,
};

use crate::{
    handlers::config::{FrontendConfig, SharedCompleteConfig},
    utils::text::{title_spans, TitleStyle},
};

use super::Component;

// #[derive(Debug, PartialEq, Clone)]
// pub struct DebugWindow {
//     visible: bool,
//     pub raw_config: Option<Table>,
// }

// impl DebugWindow {
//     const fn new(visible: bool, raw_config: Option<Table>) -> Self {
//         Self {
//             visible,
//             raw_config,
//         }
//     }

//     pub const fn is_visible(&self) -> bool {
//         self.visible
//     }

//     pub fn toggle(&mut self) {
//         self.visible = !self.visible;
//     }
// }

#[derive(Debug, Clone)]
pub struct DebugWidget {
    config: SharedCompleteConfig,
    raw_config: Option<TomlTable>,
    visible: bool,
}

impl DebugWidget {
    pub fn new(config: SharedCompleteConfig, raw_config: Option<TomlTable>) -> Self {
        Self {
            config,
            raw_config,
            visible: false,
        }
    }
}

impl Component for DebugWidget {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Option<Rect>) {
        let mut rows = vec![];

        if let Some(mut raw) = self.raw_config.clone() {
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
                    .border_type(self.config.borrow().frontend.border_type.clone().into()),
            )
            .widths(&[
                Constraint::Length(10),
                Constraint::Length(25),
                Constraint::Min(50),
            ]);

        f.render_widget(table, area.unwrap());
    }
}
