use std::ops::Index;

use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Row, Table, TableState},
    Frame,
};

use crate::{
    emotes::Emotes,
    handlers::{
        config::SharedCompleteConfig,
        user_input::events::{Event, Key},
    },
    terminal::TerminalAction,
    twitch::{
        oauth::{get_channel_id, get_twitch_client, get_user_following, FollowingList},
        TwitchAction,
    },
    ui::{components::Component, statics::NAME_MAX_CHARACTERS},
    utils::text::{title_line, TitleStyle},
};

#[derive(Debug, Clone)]
pub struct FollowingWidget {
    config: SharedCompleteConfig,
    focused: bool,
    following: FollowingList,
    state: TableState,
}

impl FollowingWidget {
    pub fn new(config: SharedCompleteConfig) -> Self {
        Self {
            config,
            focused: false,
            following: FollowingList::default(),
            state: TableState::default(),
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.following.data.len() - 1 {
                    self.following.data.len() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = self
            .state
            .selected()
            .map_or(0, |i| if i == 0 { 0 } else { i - 1 });
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }

    pub async fn get_following(&mut self) {
        let oauth_token = self.config.borrow().twitch.token.clone();
        let app_user = self.config.borrow().twitch.username.clone();

        let client = get_twitch_client(oauth_token).await.unwrap();

        let user_id = get_channel_id(&client, &app_user).await.unwrap();

        self.following = get_user_following(&client, user_id).await;
    }

    pub const fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn toggle_focus(&mut self) {
        self.focused = !self.focused;
    }
}

impl Component for FollowingWidget {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, _emotes: Option<&mut Emotes>) {
        let mut rows = vec![];

        for channel in self.following.clone().data {
            rows.push(Row::new(vec![channel.broadcaster_name.clone()]));
        }

        let title_binding = [TitleStyle::Single("Following")];

        let constraint_binding = [Constraint::Length(NAME_MAX_CHARACTERS as u16)];

        let table = Table::new(rows)
            .block(
                Block::default()
                    .title(title_line(
                        &title_binding,
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_type(self.config.borrow().frontend.border_type.clone().into()),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .widths(&constraint_binding);

        f.render_widget(Clear, area);
        f.render_stateful_widget(table, area, &mut self.state);
    }

    fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            match key {
                Key::Char('q') => return Some(TerminalAction::Quit),
                Key::Esc => {
                    self.unselect();
                    self.toggle_focus();

                    return Some(TerminalAction::BackOneLayer);
                }
                Key::Ctrl('p') => panic!("Manual panic triggered by user."),
                Key::ScrollDown => self.next(),
                Key::ScrollUp => self.previous(),
                Key::Enter => {
                    if let Some(i) = self.state.selected() {
                        self.toggle_focus();

                        self.unselect();

                        let selected_channel = &self.following.data.index(i).broadcaster_login;

                        self.config.borrow_mut().twitch.channel = selected_channel.clone();

                        return Some(TerminalAction::Enter(TwitchAction::Join(
                            selected_channel.clone(),
                        )));
                    }
                }
                _ => {}
            }
        }

        None
    }
}
