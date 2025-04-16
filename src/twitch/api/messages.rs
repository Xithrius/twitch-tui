use color_eyre::{Result, eyre::ContextCompat};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::TWITCH_API_BASE_URL;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NewTwitchMessage {
    broadcaster_id: String,
    user_id: String,
    message: String,
}

impl NewTwitchMessage {
    pub const fn new(broadcaster_id: String, user_id: String, message: String) -> Self {
        Self {
            broadcaster_id,
            user_id,
            message,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TwitchNewMessageResponse {
    message_id: String,
    is_sent: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct TwitchNewMessageResponseList {
    data: Vec<TwitchNewMessageResponse>,
}

/// Sends a message to the respective channel.
/// This was chosen vs websocket stdin due to being able to handle errors
/// in a cleaner way.
///
/// <https://dev.twitch.tv/docs/api/reference/#send-chat-message>
pub async fn send_twitch_message(
    client: &Client,
    new_message: NewTwitchMessage,
) -> Result<TwitchNewMessageResponse> {
    let url = format!("{TWITCH_API_BASE_URL}/chat/messages");

    let response_data = client
        .post(url)
        .json(&new_message)
        .send()
        .await?
        .error_for_status()?
        .json::<TwitchNewMessageResponseList>()
        .await?
        .data
        .first()
        .context("Could not get new Twitch message response")?
        .clone();

    Ok(response_data)
}
