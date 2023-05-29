use std::slice::Iter;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
};

use crate::{
    emotes::Emotes,
    handlers::{
        config::SharedCompleteConfig,
        state::State,
        storage::SharedStorage,
        user_input::events::{Event, Key},
    },
    terminal::TerminalAction,
    twitch::TwitchAction,
    ui::components::{utils::centered_rect, ChannelSwitcherWidget, Component},
    utils::styles::DASHBOARD_TITLE_COLOR,
};

const DASHBOARD_TITLE: [&str; 5] = [
    "   __           _ __       __          __        _ ",
    "  / /__      __(_) /______/ /_        / /___  __(_)",
    " / __/ | /| / / / __/ ___/ __ \\______/ __/ / / / / ",
    "/ /_ | |/ |/ / / /_/ /__/ / / /_____/ /_/ /_/ / /  ",
    "\\__/ |__/|__/_/\\__/\\___/_/ /_/      \\__/\\__,_/_/   ",
];

const CHANNEL_LIST_AMOUNT: u16 = 5;

pub struct DashboardWidget {
    config: SharedCompleteConfig,
    storage: SharedStorage,
    channel_input: ChannelSwitcherWidget,
}

impl DashboardWidget {
    pub fn new(config: SharedCompleteConfig, storage: SharedStorage) -> Self {
        let channel_input = ChannelSwitcherWidget::new(config.clone(), storage.clone());

        Self {
            config,
            storage,
            channel_input,
        }
    }

    fn create_interactive_list_widget<'a>(
        &'a self,
        items: &'a [String],
        index_offset: usize,
    ) -> List<'_> {
        List::new(
            items
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    ListItem::new(Line::from(vec![
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

    fn render_dashboard_title_widget<B: Backend>(
        &self,
        frame: &mut Frame<B>,
        v_chunks: &mut Iter<Rect>,
    ) {
        let w = Paragraph::new(
            DASHBOARD_TITLE
                .iter()
                .map(|&s| Line::from(vec![Span::raw(s)]))
                .collect::<Vec<Line>>(),
        )
        .style(DASHBOARD_TITLE_COLOR);

        frame.render_widget(w, *v_chunks.next().unwrap());
    }

    fn render_channel_selection_widget<B: Backend>(
        &self,
        frame: &mut Frame<B>,
        v_chunks: &mut Iter<Rect>,
        current_channel: String,
        default_channels: &[String],
    ) {
        frame.render_widget(
            Paragraph::new("Currently selected channel")
                .style(Style::default().fg(Color::LightRed)),
            *v_chunks.next().unwrap(),
        );

        let current_channel_selection = Paragraph::new(Line::from(vec![
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

        let recent_channels = self.storage.borrow().get_last_n("channels", 5, true);

        if recent_channels.is_empty() {
            frame.render_widget(Paragraph::new("None"), *v_chunks.next().unwrap());
        } else {
            let recent_channels_widget =
                self.create_interactive_list_widget(&recent_channels, default_channels.len());

            frame.render_widget(recent_channels_widget, *v_chunks.next().unwrap());
        }
    }

    fn render_quit_selection_widget<B: Backend>(
        &self,
        frame: &mut Frame<B>,
        v_chunks: &mut Iter<Rect>,
    ) {
        let quit_option = Paragraph::new(Line::from(vec![
            Span::raw("["),
            Span::styled("q", Style::default().fg(Color::LightMagenta)),
            Span::raw("] "),
            Span::raw("Quit"),
        ]));

        frame.render_widget(quit_option, *v_chunks.next().unwrap());
    }
}

impl Component for DashboardWidget {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, emotes: Option<Emotes>) {
        let start_screen_channels_len =
            self.config.borrow().frontend.start_screen_channels.len() as u16;

        let recent_channels_len = self.storage.borrow().get("channels").len() as u16;

        let channel_list_constrainer = |l: u16| -> u16 {
            if l == 0 {
                2
            } else if l <= CHANNEL_LIST_AMOUNT {
                l + 1
            } else {
                CHANNEL_LIST_AMOUNT + 1
            }
        };

        let v_chunk_binding = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                // Twitch-tui ASCII logo
                Constraint::Length(DASHBOARD_TITLE.len() as u16 + 2),
                // Currently selected channel title, content
                Constraint::Length(2),
                Constraint::Length(2),
                // Configured default channels title, content
                Constraint::Length(2),
                Constraint::Length(channel_list_constrainer(start_screen_channels_len)),
                // Recent channel title, content
                Constraint::Length(2),
                Constraint::Length(channel_list_constrainer(recent_channels_len)),
                // Quit
                Constraint::Length(1),
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

        if self.channel_input.is_focused() {
            self.channel_input
                .draw(f, centered_rect(60, 20, f.size()), emotes);
        }
    }

    fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            if self.channel_input.is_focused() {
                return self.channel_input.event(event);
            }

            match key {
                Key::Ctrl('p') => panic!("Manual panic triggered by user."),
                Key::Char('q') => return Some(TerminalAction::Quit),
                Key::Char('s') => self.channel_input.toggle_focus(),
                Key::Enter => {
                    let action = TerminalAction::Enter(TwitchAction::Join(
                        self.config.borrow().twitch.channel.clone(),
                    ));

                    return Some(action);
                }
                Key::Char('?') => return Some(TerminalAction::SwitchState(State::Help)),
                Key::Char(c) => {
                    if let Some(selection) = c.to_digit(10) {
                        let mut channels =
                            self.config.borrow().frontend.start_screen_channels.clone();
                        channels.extend(self.storage.borrow().get_last_n(
                            "channels",
                            CHANNEL_LIST_AMOUNT as usize,
                            true,
                        ));

                        if let Some(channel) = channels.get(selection as usize) {
                            let action =
                                TerminalAction::Enter(TwitchAction::Join(channel.to_string()));

                            self.config.borrow_mut().twitch.channel = channel.to_string();
                            self.storage
                                .borrow_mut()
                                .add("channels", channel.to_string());

                            return Some(action);
                        }
                    }
                }
                _ => {}
            }
        }

        None
    }
}
