use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Row, Table},
    Frame,
};

use crate::{
    emotes::Emotes,
    handlers::config::SharedCompleteConfig,
    twitch::oauth::{get_channel_id, get_twitch_client, get_user_following, FollowingList},
    ui::components::Component,
    utils::text::{title_line, TitleStyle},
};

#[derive(Debug, Clone)]
pub struct FollowingWidget {
    config: SharedCompleteConfig,
    focused: bool,
    following: Option<FollowingList>,
}

impl FollowingWidget {
    pub fn new(config: SharedCompleteConfig) -> Self {
        Self {
            config,
            focused: false,
            following: None,
        }
    }

    // pub fn get_following(&mut self) {
    //     let oauth_token = self.config.borrow().twitch.token.clone();
    //     let app_user = self.config.borrow().twitch.username.clone();

    //     let output = tokio::task::spawn_blocking(move || {
    //         let rt = tokio::runtime::Runtime::new().unwrap();

    //         rt.block_on(async {
    //             let client = get_twitch_client(oauth_token).await.unwrap();

    //             let user_id = get_channel_id(&client, &app_user).await.unwrap();

    //             Some(get_user_following(&client, user_id).await)
    //         })
    //     });

    //     mem::drop(output.and_then(|x| async move {
    //         self.following = x;

    //         Ok(())
    //     }));
    // }

    pub async fn get_following(&mut self) {
        let oauth_token = self.config.borrow().twitch.token.clone();
        let app_user = self.config.borrow().twitch.username.clone();

        let client = get_twitch_client(oauth_token).await.unwrap();

        let user_id = get_channel_id(&client, &app_user).await.unwrap();

        self.following = Some(get_user_following(&client, user_id).await);
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

        if let Some(followed_channels) = self.following.clone() {
            for channel in followed_channels.data {
                rows.push(Row::new(vec![channel.broadcaster_name.clone()]));
            }
        }

        let title_binding = [TitleStyle::Single("Following")];

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
            .widths(&[Constraint::Length(10), Constraint::Length(10)]);

        f.render_widget(Clear, area);
        f.render_widget(table, area);
    }
}
