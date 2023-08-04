use std::fmt::Display;

use reqwest::Client;
use serde::Deserialize;
use tokio::{runtime::Handle, task};

use crate::handlers::config::TwitchConfig;

use super::oauth::{get_channel_id, get_twitch_client};

const FOLLOWER_COUNT: usize = 100;

#[derive(Deserialize, Debug, Clone, Default)]
#[allow(dead_code)]
pub struct FollowingUser {
    broadcaster_id: String,
    pub broadcaster_login: String,
    pub broadcaster_name: String,
    followed_at: String,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[allow(dead_code)]
struct Pagination {
    cursor: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[allow(dead_code)]
pub struct FollowingList {
    pub total: u64,
    pub data: Vec<FollowingUser>,
    pagination: Pagination,
}

// https://dev.twitch.tv/docs/api/reference/#get-followed-channels
pub async fn get_user_following(client: &Client, user_id: i32) -> FollowingList {
    client
        .get(format!(
            "https://api.twitch.tv/helix/channels/followed?user_id={user_id}&first={FOLLOWER_COUNT}",
        ))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json::<FollowingList>()
        .await
        .unwrap()
}

pub async fn get_following(twitch_config: &TwitchConfig) -> FollowingList {
    let oauth_token = twitch_config.token.clone();
    let app_user = twitch_config.username.clone();

    let client = get_twitch_client(oauth_token).await.unwrap();

    let user_id = get_channel_id(&client, &app_user).await.unwrap();

    get_user_following(&client, user_id).await
}

impl FollowingList {
    pub fn get_followed_channels(twitch_config: TwitchConfig) -> Self {
        task::block_in_place(move || {
            Handle::current().block_on(async move { get_following(&twitch_config.clone()).await })
        })
    }
}

impl Display for FollowingUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.broadcaster_login.fmt(f)
    }
}

impl From<Vec<FollowingUser>> for FollowingList {
    fn from(value: Vec<FollowingUser>) -> Self {
        Self {
            total: value.len() as u64,
            data: value,
            ..Default::default()
        }
    }
}
