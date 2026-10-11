use chrono::{DateTime, Utc};
use color_eyre::{Result, eyre::ContextCompat};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{ModeratorQuery, TWITCH_API_BASE_URL};
use crate::twitch::api::ResponseList;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TimeoutInnerPayload {
    user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

impl TimeoutInnerPayload {
    pub const fn new(user_id: String, duration: Option<usize>, reason: Option<String>) -> Self {
        Self {
            user_id,
            duration,
            reason,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TimeoutPayload {
    data: TimeoutInnerPayload,
}

impl TimeoutPayload {
    pub const fn new(user_id: String, duration: Option<usize>, reason: Option<String>) -> Self {
        let inner = TimeoutInnerPayload::new(user_id, duration, reason);

        Self { data: inner }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TwitchTimeoutResponse {
    broadcaster_id: String,
    moderator_id: String,
    user_id: String,
    created_at: DateTime<Utc>,
    end_time: Option<DateTime<Utc>>,
}

/// Bans a user from participating in the specified broadcaster’s chat room or puts them in a timeout.
///
/// <https://dev.twitch.tv/docs/api/reference/#ban-user>
pub async fn timeout_twitch_user(
    client: &Client,
    query: ModeratorQuery,
    payload: TimeoutPayload,
) -> Result<TwitchTimeoutResponse> {
    let url = format!("{TWITCH_API_BASE_URL}/moderation/bans");

    let response_data = client
        .post(url)
        .query(&query)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?
        .json::<ResponseList<TwitchTimeoutResponse>>()
        .await?
        .data
        .first()
        .context("Failed to get timeout response data")?
        .clone();
    Ok(response_data)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UnbanQuery {
    broadcaster_id: String,
    moderator_id: String,
    user_id: String,
}

impl UnbanQuery {
    pub const fn new(broadcaster_id: String, moderator_id: String, user_id: String) -> Self {
        Self {
            broadcaster_id,
            moderator_id,
            user_id,
        }
    }
}

/// Removes the ban or timeout that was placed on the specified user
///
/// <https://dev.twitch.tv/docs/api/reference/#unban-user>
pub async fn unban_twitch_user(client: &Client, query: UnbanQuery) -> Result<()> {
    let url = format!("{TWITCH_API_BASE_URL}/moderation/bans");
    client
        .delete(&url)
        .query(&query)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}
