use std::slice::Iter;

use color_eyre::{
    Result,
    eyre::{Context, ContextCompat},
};
use tokio::sync::mpsc::Sender;
use tui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
};

use crate::{
    config::SharedCoreConfig,
    events::{Event, InternalEvent, Key, TwitchAction, TwitchEvent, get_keybind_text},
    handlers::{state::State, storage::SharedStorage},
    twitch::oauth::TwitchOauth,
    ui::components::{ChannelSwitcherWidget, Component, FollowingWidget},
    utils::styles::{DASHBOARD_SECTION_STYLE, DASHBOARD_TITLE_COLOR_STYLE, TEXT_DARK_STYLE},
};

const DASHBOARD_TITLE: [&str; 5] = [
    "   __           _ __       __          __        _ ",
    "  / /__      __(_) /______/ /_        / /___  __(_)",
    " / __/ | /| / / / __/ ___/ __ \\______/ __/ / / / / ",
    "/ /_ | |/ |/ / / /_/ /__/ / / /_____/ /_/ /_/ / /  ",
    "\\__/ |__/|__/_/\\__/\\___/_/ /_/      \\__/\\__,_/_/   ",
];

pub struct DashboardWidget {
    config: SharedCoreConfig,
    event_tx: Sender<Event>,
    storage: SharedStorage,
    channel_input: ChannelSwitcherWidget,
    following: FollowingWidget,
    channel_selection: Option<u32>,
}

impl DashboardWidget {
    pub fn new(
        config: SharedCoreConfig,
        twitch_oauth: TwitchOauth,
        event_tx: Sender<Event>,
        storage: SharedStorage,
    ) -> Self {
        let channel_input =
            ChannelSwitcherWidget::new(config.clone(), event_tx.clone(), storage.clone());
        let following = FollowingWidget::new(config.clone(), twitch_oauth, event_tx.clone());

        Self {
            config,
            event_tx,
            storage,
            channel_input,
            following,
            channel_selection: None,
        }
    }

    fn create_interactive_list_widget<'a>(
        &'a self,
        items: &'a [String],
        index_offset: usize,
    ) -> List<'a> {
        let selection = self
            .channel_selection
            .map_or(String::new(), |selection| selection.to_string());
        List::new(items.iter().enumerate().map(|(i, s)| {
            let index_offset_str = (i + index_offset).to_string();
            let selection_offset = if index_offset_str.starts_with(&selection) {
                selection.len()
            } else {
                0
            };
            ListItem::new(Line::from(vec![
                Span::raw("["),
                Span::styled(
                    index_offset_str
                        .get(0..selection_offset)
                        .unwrap_or_default()
                        .to_string(),
                    Style::default().fg(Color::Red).bold(),
                ),
                Span::styled(
                    index_offset_str
                        .get(selection_offset..)
                        .unwrap_or_default()
                        .to_string(),
                    Style::default().fg(Color::LightMagenta),
                ),
                Span::raw("] "),
                Span::raw(s),
            ]))
        }))
        .style(*TEXT_DARK_STYLE)
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
    }

    fn render_dashboard_title_widget(frame: &mut Frame, chunk: Rect) {
        let w = Paragraph::new(
            DASHBOARD_TITLE
                .iter()
                .map(|&s| Line::from(vec![Span::raw(s)]))
                .collect::<Vec<Line>>(),
        )
        .style(*DASHBOARD_TITLE_COLOR_STYLE);

        frame.render_widget(w, chunk);
    }

    fn render_channel_selection_widget(
        &self,
        frame: &mut Frame,
        v_chunks: &mut Iter<Rect>,
        current_channel: String,
        default_channels: &[String],
    ) -> Result<()> {
        let [
            selected_title_chunk,
            selected_content_chunk,
            favorite_title_chunk,
            favorite_content_chunk,
            recent_title_chunk,
            recent_content_chunk,
        ] = std::array::try_from_fn(|_| {
            v_chunks
                .next()
                .context("Failed to get all dashboard vertical chunks for channel selection")
        })?;

        frame.render_widget(
            Paragraph::new("Currently selected channel").style(*DASHBOARD_SECTION_STYLE),
            *selected_title_chunk,
        );

        let current_channel_selection = Paragraph::new(Line::from(vec![
            Span::raw("["),
            Span::styled(
                get_keybind_text(&self.config.keybinds.dashboard.join),
                Style::default().fg(Color::LightMagenta),
            ),
            Span::raw("] "),
            Span::raw(current_channel),
        ]));

        frame.render_widget(current_channel_selection, *selected_content_chunk);

        frame.render_widget(
            Paragraph::new("Favorite channels").style(*DASHBOARD_SECTION_STYLE),
            *favorite_title_chunk,
        );

        if default_channels.is_empty() {
            frame.render_widget(Paragraph::new("None"), *favorite_content_chunk);
        } else {
            let default_channels_widget = self.create_interactive_list_widget(default_channels, 0);

            frame.render_widget(default_channels_widget, *favorite_content_chunk);
        }

        frame.render_widget(
            Paragraph::new("Most recent channels").style(*DASHBOARD_SECTION_STYLE),
            *recent_title_chunk,
        );

        let recent_channels = self
            .storage
            .borrow()
            .get_last_n("channels", 5, true)
            .context("Failed to get recent channels")?;

        if recent_channels.is_empty() {
            frame.render_widget(Paragraph::new("None"), *recent_content_chunk);
        } else {
            let recent_channels_widget =
                self.create_interactive_list_widget(&recent_channels, default_channels.len());

            frame.render_widget(recent_channels_widget, *recent_content_chunk);
        }

        Ok(())
    }

    fn render_quit_selection_widget(&self, frame: &mut Frame, chunk: Rect) {
        let spans = vec![
            Span::raw("["),
            Span::styled(
                get_keybind_text(&self.config.keybinds.dashboard.quit),
                Style::default().fg(Color::LightMagenta),
            ),
            Span::raw("] "),
            Span::raw("Quit"),
        ];

        let quit_option = Paragraph::new(Line::from(spans));

        frame.render_widget(quit_option, chunk);
    }
}

impl Component for DashboardWidget {
    fn draw(&mut self, f: &mut Frame, area: Option<Rect>) -> Result<()> {
        let r = area.unwrap_or_else(|| f.area());

        let favorite_channels_len = {
            let l = self.config.frontend.favorite_channels.len() as u16;

            if l == 0 { 2 } else { l + 1 }
        };

        let recent_channels_len = {
            let current_len = self.storage.borrow().get("channels")?.len() as u16;
            let recent_channel_config_count = self.config.frontend.recent_channel_count;

            if current_len == 0 {
                2
            } else if current_len <= recent_channel_config_count {
                current_len + 1
            } else {
                recent_channel_config_count + 1
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
                // Favorite channels title, content
                Constraint::Length(2),
                Constraint::Length(favorite_channels_len),
                // Recent channel title, content
                Constraint::Length(2),
                Constraint::Length(recent_channels_len),
                // Quit
                Constraint::Length(1),
            ])
            .margin(2)
            .split(r);

        let mut v_chunks = v_chunk_binding.iter();

        let title_chunk = v_chunks.next().context("Failed to get title chunk")?;
        Self::render_dashboard_title_widget(f, *title_chunk);

        self.render_channel_selection_widget(
            f,
            &mut v_chunks,
            self.config.twitch.channel.clone(),
            &self.config.frontend.favorite_channels.clone(),
        )?;

        let quit_chunk = v_chunks.next().context("Failed to get quit chunk")?;
        self.render_quit_selection_widget(f, *quit_chunk);

        if self.channel_input.is_focused() {
            self.channel_input.draw(f, None)?;
        } else if self.following.is_focused() {
            self.following.draw(f, None)?;
        }

        Ok(())
    }

    async fn event(&mut self, event: &Event) -> Result<()> {
        if let Event::Input(key) = event {
            if self.channel_input.is_focused() {
                return self.channel_input.event(event).await;
            } else if self.following.is_focused() {
                return self.following.event(event).await;
            }

            let keybinds = &self.config.keybinds.dashboard;
            match key {
                key if keybinds.quit.contains(key) => {
                    self.event_tx
                        .send(Event::Internal(InternalEvent::Quit))
                        .await?;
                }
                key if keybinds.recent_channels_search.contains(key) => {
                    self.channel_input.toggle_focus();
                }
                key if keybinds.followed_channels_search.contains(key) => {
                    self.following.toggle_focus().await;
                }
                key if keybinds.join.contains(key) => {
                    let configured_channel = self.config.twitch.channel.clone();
                    self.event_tx
                        .send(Event::Twitch(TwitchEvent::Action(
                            TwitchAction::JoinChannel(configured_channel),
                        )))
                        .await?;
                }
                key if keybinds.help.contains(key) => {
                    self.event_tx
                        .send(Event::Internal(InternalEvent::SwitchState(State::Help)))
                        .await?;
                }
                Key::Char(c) => {
                    if let Some(digit) = c.to_digit(10) {
                        let mut channels = self.config.frontend.favorite_channels.clone();
                        let mut selected_channels = self.storage.borrow().get("channels")?;
                        selected_channels.reverse();

                        channels.extend(selected_channels);

                        let selection =
                            if let Some(favorites_selection) = self.channel_selection.as_mut() {
                                *favorites_selection = *favorites_selection * 10 + digit;
                                *favorites_selection
                            } else {
                                self.channel_selection = Some(digit);
                                digit
                            } as usize;

                        if let Some(channel) = channels.get(selection) {
                            self.storage
                                .borrow_mut()
                                .add("channels", channel.clone())
                                .context(format!("Failed to add channel {channel} to storage"))?;

                            if selection != 0 && selection * 10 <= channels.len() {
                                return Ok(());
                            }

                            self.channel_selection = None;
                            self.event_tx
                                .send(Event::Twitch(TwitchEvent::Action(
                                    TwitchAction::JoinChannel(channel.clone()),
                                )))
                                .await?;

                            return Ok(());
                        }
                        self.channel_selection = None;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
