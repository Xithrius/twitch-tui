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
    non_moderator_chat_delay: bool,
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
    client: &Client,
    broadcaster_id: &String,
) -> Result<TwitchChatSettingsResponse> {
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
