use std::sync::OnceLock;

use color_eyre::{Result, eyre::ContextCompat};
use reqwest::{
    Client,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use serde::Deserialize;

// curl -X POST 'https://api.twitch.tv/helix/chat/messages' \
// -H 'Authorization: Bearer 2gbdx6oar67tqtcmt49t3wpcgycthx' \
// -H 'Client-Id: wbmytr93xzw8zbg0p1izqyzzc5mbiz' \
// -H 'Content-Type: application/json' \
// -d '{
//   "broadcaster_id": "12826",
//   "sender_id": "141981764",
//   "message": "Hello, world! twitchdevHype",
//   "for_source_only": true
// }'

/// Sends a message to the respective channel.
/// This was chosen vs websocket stdin due to being able to handle errors
/// in a cleaner way.
///
/// <https://dev.twitch.tv/docs/api/reference/#send-chat-message>
pub async fn send_twitch_message(client: &Client, message: &str) -> Result<()> {
    todo!()
}
