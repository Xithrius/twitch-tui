use std::sync::OnceLock;

use color_eyre::{Result, eyre::ContextCompat};
use reqwest::{
    Client,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};

use super::TWITCH_API_BASE_URL;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NewTwitchMessage {
    broadcaster_id: String,
    user_id: String,
    message: String,
}

impl NewTwitchMessage {
    pub fn new(broadcaster_id: String, user_id: String, message: String) -> Self {
        Self {
            broadcaster_id,
            user_id,
            message,
        }
    }
}

// curl -X POST 'https://api.twitch.tv/helix/chat/messages' \
// -H 'Authorization: Bearer 2gbdx6oar67tqtcmt49t3wpcgycthx' \
// -H 'Client-Id: wbmytr93xzw8zbg0p1izqyzzc5mbiz' \
// -H 'Content-Type: application/json' \
// -d '{
//   "broadcaster_id": "12826",
//   "sender_id": "141981764",
//   "message": "Hello, world! twitchdevHype",
// }'

/// Sends a message to the respective channel.
/// This was chosen vs websocket stdin due to being able to handle errors
/// in a cleaner way.
///
/// <https://dev.twitch.tv/docs/api/reference/#send-chat-message>
pub async fn send_twitch_message(client: &Client, new_message: NewTwitchMessage) -> Result<()> {
    let url = format!("{}/chat/messages", TWITCH_API_BASE_URL);

    Ok(())

    // let session_id = session_id.context("Session ID is empty")?;

    // let subscription = ReceivedTwitchSubscription::new(
    //     subscription_type,
    //     channel_id,
    //     client_id.user_id.clone(),
    //     session_id,
    // );

    // let response = client
    //     .post(url)
    //     .header("Content-Type", "application/json")
    //     .json(&subscription)
    //     .send()
    //     .await?;

    // let response_data: TwitchSubscriptionResponse = response.json().await?;

    // Ok(response_data)
}
