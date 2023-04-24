use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table},
    Frame,
};

use crate::{
    emotes::Emotes,
    handlers::config::SharedCompleteConfig,
    utils::text::{title_spans, TitleStyle},
};

use super::Component;

#[derive(Debug, Clone)]
pub struct DebugWidget {
    config: SharedCompleteConfig,
    focused: bool,
}

impl DebugWidget {
    pub fn new(config: SharedCompleteConfig) -> Self {
        Self {
            config,
            focused: false,
        }
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn toggle_focus(&mut self) {
        self.focused = !self.focused;
    }
}

impl Component for DebugWidget {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Option<Rect>, _emotes: Option<Emotes>) {
        // TODO: Add more debug stuff
        let config = self.config.borrow();

        let rows = vec![Row::new(vec!["Current channel", &config.twitch.channel])];

        // if let Some(mut raw) = self.raw_config.clone() {
        //     // To avoid getting the user's token leaked in front of others.
        //     raw.remove("twitch");

        //     for item in raw.iter() {
        //         rows.push(Row::new(vec![item.0.to_string()]));
        //         let inner_map = item.1.as_table();
        //         if let Some(inner) = inner_map {
        //             for inner_item in inner.iter() {
        //                 rows.push(Row::new(vec![
        //                     " ".to_string(),
        //                     inner_item.0.to_string(),
        //                     inner_item.1.to_string(),
        //                 ]));
        //             }
        //         }
        //     }
        // }

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
            .widths(&[Constraint::Length(10), Constraint::Length(10)]);

        f.render_widget(table, area.unwrap());
    }
}
