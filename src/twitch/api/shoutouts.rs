use color_eyre::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::TWITCH_API_BASE_URL;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShoutoutQuery {
    from_broadcaster_id: String,
    to_broadcaster_id: String,
    moderator_id: String,
}

impl ShoutoutQuery {
    pub const fn new(
        from_broadcaster_id: String,
        to_broadcaster_id: String,
        moderator_id: String,
    ) -> Self {
        Self {
            from_broadcaster_id,
            to_broadcaster_id,
            moderator_id,
        }
    }
}

/// Sends a Shoutout to the specified broadcaster.
///
/// <https://dev.twitch.tv/docs/api/reference/#send-a-shoutout>
pub async fn shoutout_twitch_user(client: &Client, query: ShoutoutQuery) -> Result<()> {
    let url = format!("{TWITCH_API_BASE_URL}/chat/shoutouts");

    client
        .post(url)
        .query(&query)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
