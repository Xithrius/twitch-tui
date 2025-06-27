use chrono::{DateTime, Utc};
use color_eyre::{Result, eyre::ContextCompat};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::TWITCH_API_BASE_URL;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TimeoutQuery {
    broadcaster_id: String,
    moderator_id: String,
}

impl TimeoutQuery {
    pub const fn new(broadcaster_id: String, moderator_id: String) -> Self {
        Self {
            broadcaster_id,
            moderator_id,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TimeoutInnerPayload {
    user_id: String,
    duration: Option<usize>,
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
    data: Vec<TimeoutInnerPayload>,
}

impl TimeoutPayload {
    pub fn new(user_id: String, duration: Option<usize>, reason: Option<String>) -> Self {
        let inner = TimeoutInnerPayload::new(user_id, duration, reason);

        Self { data: vec![inner] }
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

#[derive(Deserialize, Serialize, Debug, Clone)]
struct TwitchTimeoutResponseList {
    data: Vec<TwitchTimeoutResponse>,
}

/// Bans a user from participating in the specified broadcaster’s chat room or puts them in a timeout.
///
/// <https://dev.twitch.tv/docs/api/reference/#ban-user>
pub async fn timeout_twitch_user(
    client: &Client,
    query: TimeoutQuery,
    payload: TimeoutPayload,
) -> Result<TwitchTimeoutResponse> {
    let url = format!("{TWITCH_API_BASE_URL}/moderation/bans");

    let timeout_query = &[
        ("broadcaster_id", query.broadcaster_id),
        ("moderator_id", query.moderator_id),
    ];

    let response_data = client
        .post(url)
        .query(timeout_query)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?
        .json::<TwitchTimeoutResponseList>()
        .await?
        .data
        .first()
        .context("Could not get Twitch timeout response")?
        .clone();

    Ok(response_data)
}
