use chrono::{DateTime, Local};
use color_eyre::Result;
use tokio::sync::mpsc::Sender;
use tui::{
    Frame,
    layout::{Constraint, Rect},
    prelude::Alignment,
    widgets::{Block, Borders, Clear, Row, Table, TitlePosition},
};

use crate::{
    config::SharedCoreConfig,
    events::{Event, InternalEvent},
    ui::components::Component,
    utils::{
        styles::{BOLD_STYLE, TITLE_STYLE},
        text::{TitleStyle, title_line},
    },
};

#[derive(Debug, Clone)]
pub struct DebugWidget {
    config: SharedCoreConfig,
    event_tx: Sender<Event>,
    focused: bool,
    startup_time: DateTime<Local>,
}

impl DebugWidget {
    pub fn new(config: SharedCoreConfig, event_tx: Sender<Event>) -> Self {
        let startup_time = Local::now();

        Self {
            config,
            event_tx,
            focused: false,
            startup_time,
        }
    }

    pub const fn is_focused(&self) -> bool {
        self.focused
    }

    pub const fn toggle_focus(&mut self) {
        self.focused = !self.focused;
    }

    fn get_config_values(&self) -> Vec<(String, Vec<(String, String)>)> {
        vec![
            (
                "Twitch Config".to_string(),
                self.config.twitch.clone().into(),
            ),
            (
                "Terminal Config".to_string(),
                self.config.terminal.clone().into(),
            ),
            (
                "Storage Config".to_string(),
                self.config.storage.clone().into(),
            ),
            (
                "Message Filters Config".to_string(),
                self.config.filters.message.clone().into(),
            ),
            (
                "Username Filters Config".to_string(),
                self.config.filters.username.clone().into(),
            ),
            (
                "Frontend Config".to_string(),
                self.config.frontend.clone().into(),
            ),
        ]
    }
}

impl Component for DebugWidget {
    fn draw(&mut self, f: &mut Frame, area: Option<Rect>) {
        let r = area.unwrap_or_else(|| f.area());

        let configs = self.get_config_values();

        let rows = configs
            .iter()
            .enumerate()
            .flat_map(|(i, (t, values))| {
                let mut inner_rows = if i > 0 {
                    vec![
                        Row::new::<Vec<String>>(vec![]),
                        Row::new(vec![t.clone()]).style(*BOLD_STYLE),
                    ]
                } else {
                    vec![Row::new(vec![t.clone()]).style(*BOLD_STYLE)]
                };

                for (k, v) in values {
                    inner_rows.push(Row::new(vec![k.clone(), v.clone()]));
                }

                inner_rows
            })
            .collect::<Vec<Row>>();

        let title_binding = [TitleStyle::Single("Debug")];

        let table = Table::new(rows, &[Constraint::Length(25), Constraint::Length(25)]).block(
            Block::default()
                .title(title_line(&title_binding, *TITLE_STYLE))
                .borders(Borders::ALL)
                .border_type(self.config.frontend.border_type.clone().into()),
        );

        f.render_widget(Clear, r);
        f.render_widget(table, r);

        let title_binding = self
            .startup_time
            .format(&self.config.frontend.datetime_format)
            .to_string();

        let title = [TitleStyle::Combined("Startup time", &title_binding)];

        let bottom_block = Block::default()
            .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
            .border_type(self.config.frontend.border_type.clone().into())
            .title(title_line(&title, *TITLE_STYLE))
            .title_position(TitlePosition::Bottom)
            .title_alignment(Alignment::Left);

        let rect = Rect::new(r.x, r.bottom() - 1, r.width, 1);

        f.render_widget(bottom_block, rect);
    }

    async fn event(&mut self, event: &Event) -> Result<()> {
        if let Event::Input(key) = event {
            let keybinds = &self.config.keybinds.normal;
            match key {
                key if keybinds.quit.contains(key) => {
                    self.event_tx
                        .send(Event::Internal(InternalEvent::Quit))
                        .await?;
                }
                key if keybinds.back_to_previous_window.contains(key) => {
                    self.toggle_focus();

                    self.event_tx
                        .send(Event::Internal(InternalEvent::BackOneLayer))
                        .await?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
