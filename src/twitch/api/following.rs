use std::fmt::Display;

use color_eyre::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::TWITCH_API_BASE_URL;

const FOLLOWER_COUNT: usize = 100;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
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

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
struct Pagination {
    cursor: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct FollowingList {
    pub total: u64,
    pub data: Vec<FollowingUser>,
    pagination: Pagination,
}

/// <https://dev.twitch.tv/docs/api/reference/#get-followed-channels>
pub async fn get_user_following(client: &Client, user_id: &str) -> Result<FollowingList> {
    Ok(client
        .get(format!(
            "{TWITCH_API_BASE_URL}/channels/followed?user_id={user_id}&first={FOLLOWER_COUNT}"
        ))
        .send()
        .await?
        .error_for_status()?
        .json::<FollowingList>()
        .await?)
}
