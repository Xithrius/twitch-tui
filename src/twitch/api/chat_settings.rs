use color_eyre::{Result, eyre::ContextCompat};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::TWITCH_API_BASE_URL;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TwitchChatSettingsResponse {
    broadcaster_id: String,
    slow_mode: bool,
    slow_mode_wait_time: Option<usize>,
    follower_mode: bool,
    follower_mode_duration: Option<usize>,
    non_moderator_chat_delay: Option<bool>,
    non_moderator_chat_delay_duration: Option<usize>,
    subscriber_mode: bool,
    emote_mode: bool,
    unique_chat_mode: bool,
}

impl TwitchChatSettingsResponse {
    pub const fn slow_mode(&self) -> Option<usize> {
        self.slow_mode_wait_time
    }

    pub const fn follower_mode(&self) -> Option<usize> {
        self.follower_mode_duration
    }

    pub const fn non_moderator_chat(&self) -> Option<usize> {
        self.non_moderator_chat_delay_duration
    }

    pub const fn subscriber_mode(&self) -> bool {
        self.subscriber_mode
    }

    pub const fn emote_mode(&self) -> bool {
        self.emote_mode
    }

    pub const fn unique_chat_mode(&self) -> bool {
        self.unique_chat_mode
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct TwitchChatSettingsResponseList {
    data: Vec<TwitchChatSettingsResponse>,
}

/// Get the settings of the given broadcaster's chat
///
/// <https://dev.twitch.tv/docs/api/reference/#get-chat-settings>
pub async fn get_chat_settings(
    client: Option<&Client>,
    broadcaster_id: Option<&String>,
) -> Result<TwitchChatSettingsResponse> {
    let client = client.context("Twitch client has not been initialized")?;
    let broadcaster_id = broadcaster_id.context("No broadcaster ID has been set")?;

    let url = format!("{TWITCH_API_BASE_URL}/chat/settings?broadcaster_id={broadcaster_id}");

    let response_data = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json::<TwitchChatSettingsResponseList>()
        .await?
        .data
        .first()
        .context("Failed to get chat settings response")?
        .clone();

    Ok(response_data)
}

pub struct UpdateTwitchChatSettingsQuery {
    broadcaster_id: String,
    moderator_id: String,
}
impl UpdateTwitchChatSettingsQuery {
    pub const fn new(broadcaster_id: String, moderator_id: String) -> Self {
        Self {
            broadcaster_id,
            moderator_id,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct UpdateTwitchChatSettingsPayload {
    emote_mode: Option<bool>,
    follower_mode: Option<bool>,
    follower_mode_duration: Option<usize>,
    non_moderator_chat_delay: Option<bool>,
    non_moderator_chat_delay_duration: Option<usize>,
    slow_mode: Option<bool>,
    slow_mode_wait_time: Option<usize>,
    subscriber_mode: Option<bool>,
    unique_chat_mode: Option<bool>,
}

impl UpdateTwitchChatSettingsPayload {
    pub fn new_follower_mode(on: bool, duration: Option<usize>) -> Self {
        Self {
            follower_mode: Some(on),
            follower_mode_duration: duration,
            ..Self::default()
        }
    }
    pub fn new_slow_mode(on: bool, duration: Option<usize>) -> Self {
        Self {
            slow_mode: Some(on),
            slow_mode_wait_time: duration,
            ..Self::default()
        }
    }
    pub fn new_subscriber_mode(on: bool) -> Self {
        Self {
            subscriber_mode: Some(on),
            ..Self::default()
        }
    }
    pub fn new_emote_only_mode(on: bool) -> Self {
        Self {
            emote_mode: Some(on),
            ..Self::default()
        }
    }
}

/// Updates the broadcaster's chat settings
///
/// <https://dev.twitch.tv/docs/api/reference/#update-chat-settings>
pub async fn update_chat_settings(
    client: &Client,
    query: UpdateTwitchChatSettingsQuery,
    payload: UpdateTwitchChatSettingsPayload,
) -> Result<TwitchChatSettingsResponse> {
    let settings_query = &[
        ("broadcaster_id", query.broadcaster_id),
        ("moderator_id", query.moderator_id),
    ];
    let url = format!("{TWITCH_API_BASE_URL}/chat/settings");

    let response_data = client
        .patch(url)
        .query(settings_query)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?
        .json::<TwitchChatSettingsResponseList>()
        .await?
        .data
        .first()
        .context("Failed to get chat settings response")?
        .clone();
    Ok(response_data)
}
