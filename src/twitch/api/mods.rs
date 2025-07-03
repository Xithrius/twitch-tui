use color_eyre::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::TWITCH_API_BASE_URL;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ModQuery {
    broadcaster_id: String,
    user_id: String,
}

impl ModQuery {
    pub const fn new(broadcaster_id: String, user_id: String) -> Self {
        Self {
            broadcaster_id,
            user_id,
        }
    }
}

/// Adds a moderator to the broadcaster's chat room
///
/// <https://dev.twitch.tv/docs/api/reference/#add-channel-moderator>
pub async fn mod_twitch_user(client: &Client, query: ModQuery) -> Result<()> {
    let url = format!("{TWITCH_API_BASE_URL}/moderation/moderators");

    let mod_query = &[
        ("broadcaster_id", query.broadcaster_id),
        ("user_id", query.user_id),
    ];

    client
        .post(url)
        .query(mod_query)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

/// Removes a moderator from the broadcaster's chat room
///
/// <https://dev.twitch.tv/docs/api/reference/#remove-channel-moderator>
pub async fn unmod_twitch_user(client: &Client, query: ModQuery) -> Result<()> {
    let url = format!("{TWITCH_API_BASE_URL}/moderation/moderators");

    let unmod_query = &[
        ("broadcaster_id", query.broadcaster_id),
        ("user_id", query.user_id),
    ];

    client
        .delete(url)
        .query(unmod_query)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
