use std::{
    convert::Into,
    fmt::Display,
    string::{String, ToString},
    vec::Vec,
};

use color_eyre::{Result, eyre::ContextCompat};
use reqwest::Client;
use serde::Deserialize;

use super::TWITCH_API_BASE_URL;
use crate::{config::SharedCoreConfig, twitch::oauth::TwitchOauth};

const FOLLOWER_COUNT: usize = 100;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct FollowingUser {
    broadcaster_login: String,
    // broadcaster_id: String,
    // broadcaster_name: String,
    // followed_at: String,
}

impl Display for FollowingUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.broadcaster_login)
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct StreamingUser {
    pub user_login: String,
    pub game_name: String,
    pub title: String,
}

impl Display for StreamingUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            user_login,
            game_name,
            title,
        } = self;
        let fmt_game = format!("[{game_name:.22}]");
        write!(f, "{user_login:<25.25}: {fmt_game:<24} {title}",)
    }
}

impl From<StreamingUser> for FollowingUser {
    fn from(value: StreamingUser) -> Self {
        Self {
            broadcaster_login: value.to_string(),
        }
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
struct Pagination {
    #[allow(unused)]
    cursor: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct FollowingChannelList {
    #[allow(unused)]
    pub total: u64,
    pub data: Vec<FollowingUser>,
    #[allow(unused)]
    pagination: Pagination,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct LiveChannelList {
    pub data: Vec<StreamingUser>,
    pagination: Pagination,
}

impl From<LiveChannelList> for FollowingChannelList {
    fn from(val: LiveChannelList) -> Self {
        Self {
            total: val.data.len() as u64,
            data: val.data.into_iter().map(Into::into).collect(),
            pagination: val.pagination,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Following {
    pub config: SharedCoreConfig,
    pub twitch_oauth: TwitchOauth,
    #[allow(unused)]
    list: FollowingChannelList,
}

impl Following {
    pub fn new(config: SharedCoreConfig, twitch_oauth: TwitchOauth) -> Self {
        Self {
            config,
            twitch_oauth,
            list: FollowingChannelList::default(),
        }
    }
}

/// <https://dev.twitch.tv/docs/api/reference/#get-followed-channels>
pub async fn get_user_following(
    client: &Client,
    user_id: &str,
    live: bool,
) -> Result<FollowingChannelList> {
    let channels = if live {
        let url = format!(
            "{TWITCH_API_BASE_URL}/streams/followed?user_id={user_id}&first={FOLLOWER_COUNT}",
        );

        client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<LiveChannelList>()
            .await?
            .into()
    } else {
        let url = format!(
            "{TWITCH_API_BASE_URL}/channels/followed?user_id={user_id}&first={FOLLOWER_COUNT}",
        );

        client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<FollowingChannelList>()
            .await?
    };

    Ok(channels)
}

pub async fn get_following(twitch_oauth: TwitchOauth, live: bool) -> Result<FollowingChannelList> {
    let client = twitch_oauth
        .client()
        .context("Unable to get OAuth from twitch OAuth")?;
    let user_id = twitch_oauth
        .user_id()
        .context("Unable to get user ID from Twitch OAuth")?;

    get_user_following(&client, &user_id, live).await
}
