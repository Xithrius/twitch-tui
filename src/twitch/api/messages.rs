use color_eyre::{Result, eyre::ContextCompat};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::TWITCH_API_BASE_URL;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NewTwitchMessage {
    #[serde(rename = "broadcaster_id")]
    channel_id: String,
    #[serde(rename = "sender_id")]
    user_id: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_parent_message_id: Option<String>,
}

impl NewTwitchMessage {
    pub const fn new(
        broadcaster_id: String,
        user_id: String,
        message: String,
        reply_parent_message_id: Option<String>,
    ) -> Self {
        Self {
            channel_id: broadcaster_id,
            user_id,
            message,
            reply_parent_message_id,
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::NewTwitchMessage;

    #[test]
    fn reply_parent_only_serialized_when_replying() {
        let plain = NewTwitchMessage::new("b".into(), "u".into(), "hi".into(), None);
        assert_eq!(
            serde_json::to_value(plain).unwrap(),
            json!({"broadcaster_id": "b", "sender_id": "u", "message": "hi"})
        );

        let reply = NewTwitchMessage::new("b".into(), "u".into(), "hi".into(), Some("p".into()));
        assert_eq!(
            serde_json::to_value(reply).unwrap(),
            json!({"broadcaster_id": "b", "sender_id": "u", "message": "hi", "reply_parent_message_id": "p"})
        );
    }
}
