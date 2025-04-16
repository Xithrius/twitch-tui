use std::{
    fmt::Display,
    string::{String, ToString},
    vec::Vec,
};

use color_eyre::Result;
use reqwest::Client;
use serde::Deserialize;

use super::oauth::{get_twitch_client, get_twitch_client_id};
use crate::{handlers::config::TwitchConfig, ui::components::utils::SearchItemGetter};

const FOLLOWER_COUNT: usize = 100;

#[derive(Deserialize, Debug, Clone, Default)]
#[allow(dead_code)]
pub struct FollowingUser {
    broadcaster_id: String,
    pub broadcaster_login: String,
    pub broadcaster_name: String,
    followed_at: String,
}

impl Display for FollowingUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.broadcaster_login)
    }
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

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Following {
    twitch_config: TwitchConfig,
    list: FollowingList,
}

// https://dev.twitch.tv/docs/api/reference/#get-followed-channels
pub async fn get_user_following(client: &Client, user_id: &str) -> Result<FollowingList> {
    Ok(client
        .get(format!(
            "https://api.twitch.tv/helix/channels/followed?user_id={user_id}&first={FOLLOWER_COUNT}",
        ))
        .send()
        .await?
        .error_for_status()?
        .json::<FollowingList>()
        .await?)
}

pub async fn get_following(twitch_config: &TwitchConfig) -> Result<FollowingList> {
    let client_id = get_twitch_client_id(None).await?;
    let client = get_twitch_client(client_id, twitch_config.token.as_deref()).await?;

    get_user_following(&client, &client_id.user_id).await
}

impl Following {
    pub fn new(twitch_config: TwitchConfig) -> Self {
        Self {
            twitch_config,
            list: FollowingList::default(),
        }
    }
}

impl SearchItemGetter<String> for Following {
    async fn get_items(&mut self) -> Result<Vec<String>> {
        let following = get_following(&self.twitch_config).await;

        following.map(|v| {
            v.data
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
        })
    }
}
