use std::{collections::HashMap, slice::Iter};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{List, ListItem, Paragraph},
};

use crate::{
    handlers::{
        app::App,
        config::{CompleteConfig, SharedCompleteConfig},
        state::State,
        storage::Storage,
        user_input::{
            events::{Event, Key},
            input::TerminalAction,
        },
    },
    ui::components::Component,
    utils::styles::DASHBOARD_TITLE_COLOR,
};

const DASHBOARD_TITLE: [&str; 5] = [
    "   __           _ __       __          __        _ ",
    "  / /__      __(_) /______/ /_        / /___  __(_)",
    " / __/ | /| / / / __/ ___/ __ \\______/ __/ / / / / ",
    "/ /_ | |/ |/ / / /_/ /__/ / / /_____/ /_/ /_/ / /  ",
    "\\__/ |__/|__/_/\\__/\\___/_/ /_/      \\__/\\__,_/_/   ",
];

#[derive(Debug)]
pub struct DashboardWidget<'a> {
    config: SharedCompleteConfig,
    storage: &'a Storage,
}

impl<'a> DashboardWidget<'a> {
    pub fn new(config: SharedCompleteConfig, storage: &'a Storage) -> Self {
        Self { config, storage }
    }
}

impl<'a> DashboardWidget<'a> {
    fn create_interactive_list_widget(&self, items: &'a [String], index_offset: usize) -> List<'_> {
        List::new(
            items
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    ListItem::new(Spans::from(vec![
                        Span::raw("["),
                        Span::styled(
                            (i + index_offset).to_string(),
                            Style::default().fg(Color::LightMagenta),
                        ),
                        Span::raw("] "),
                        Span::raw(s),
                    ]))
                })
                .collect::<Vec<ListItem>>(),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
    }

    fn render_dashboard_title_widget<T: Backend>(
        &self,
        frame: &mut Frame<T>,
        v_chunks: &mut Iter<Rect>,
    ) {
        let w = Paragraph::new(
            DASHBOARD_TITLE
                .iter()
                .map(|&s| Spans::from(vec![Span::raw(s)]))
                .collect::<Vec<Spans>>(),
        )
        .style(DASHBOARD_TITLE_COLOR);

        frame.render_widget(w, *v_chunks.next().unwrap());
    }

    fn render_channel_selection_widget<T: Backend>(
        &self,
        frame: &mut Frame<T>,
        v_chunks: &mut Iter<Rect>,
        current_channel: String,
        default_channels: &[String],
    ) {
        frame.render_widget(
            Paragraph::new("Currently selected channel")
                .style(Style::default().fg(Color::LightRed)),
            *v_chunks.next().unwrap(),
        );

        let current_channel_selection = Paragraph::new(Spans::from(vec![
            Span::raw("["),
            Span::styled(
                "ENTER".to_string(),
                Style::default().fg(Color::LightMagenta),
            ),
            Span::raw("] "),
            Span::raw(current_channel),
        ]));

        frame.render_widget(current_channel_selection, *v_chunks.next().unwrap());

        frame.render_widget(
            Paragraph::new("Configured default channels")
                .style(Style::default().fg(Color::LightRed)),
            *v_chunks.next().unwrap(),
        );

        if default_channels.is_empty() {
            frame.render_widget(Paragraph::new("None"), *v_chunks.next().unwrap());
        } else {
            let default_channels_widget = self.create_interactive_list_widget(default_channels, 0);

            frame.render_widget(default_channels_widget, *v_chunks.next().unwrap());
        }

        frame.render_widget(
            Paragraph::new("Most recent channels").style(Style::default().fg(Color::LightRed)),
            *v_chunks.next().unwrap(),
        );

        let recent_channels = self.storage.get_last_n("channels", 5, true);

        if recent_channels.is_empty() {
            frame.render_widget(Paragraph::new("None"), *v_chunks.next().unwrap());
        } else {
            let recent_channels_widget =
                self.create_interactive_list_widget(&recent_channels, default_channels.len());

            frame.render_widget(recent_channels_widget, *v_chunks.next().unwrap());
        }
    }

    fn render_quit_selection_widget<T: Backend>(
        &self,
        frame: &mut Frame<T>,
        v_chunks: &mut Iter<Rect>,
    ) {
        let quit_option = Paragraph::new(Spans::from(vec![
            Span::raw("["),
            Span::styled("q", Style::default().fg(Color::LightMagenta)),
            Span::raw("] "),
            Span::raw("Quit"),
        ]));

        frame.render_widget(quit_option, *v_chunks.next().unwrap());
    }
}

impl<'a> Component for DashboardWidget<'a> {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Option<Rect>) {
        let area = area.map_or_else(|| f.size(), |a| a);

        let start_screen_channels_len =
            self.config.borrow().frontend.start_screen_channels.len() as u16;

        let recent_channels_len = self.storage.get("channels").len() as u16;

        let v_chunk_binding = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                // Twitch-tui ASCII logo
                Constraint::Min(DASHBOARD_TITLE.len() as u16 + 2),
                // Currently selected channel title, content
                Constraint::Length(2),
                Constraint::Min(2),
                // Configured default channels title, content
                Constraint::Length(2),
                Constraint::Min(if start_screen_channels_len == 0 {
                    2
                } else {
                    start_screen_channels_len + 1
                }),
                // Recent channel title, content
                Constraint::Length(2),
                Constraint::Min(if recent_channels_len == 0 {
                    2
                } else {
                    recent_channels_len + 1
                }),
                // Quit
                Constraint::Min(1),
            ])
            .margin(2)
            .split(area);

        let mut v_chunks = v_chunk_binding.iter();

        self.render_dashboard_title_widget(f, &mut v_chunks);

        self.render_channel_selection_widget(
            f,
            &mut v_chunks,
            self.config.borrow().twitch.channel.clone(),
            &self.config.borrow().frontend.start_screen_channels.clone(),
        );

        self.render_quit_selection_widget(f, &mut v_chunks);
    }

    fn event(&mut self, event: Event) -> Option<TerminalAction> {
        if matches!(event, Event::Input(Key::Char('q'))) {
            return Some(TerminalAction::Quitting);
        }

        None
    }
}
