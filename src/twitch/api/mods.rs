use color_eyre::Result;
use reqwest::{Client, Method};

use super::{BroadcasterQuery, TWITCH_API_BASE_URL, request_bodiless};

/// Adds a moderator to the broadcaster's chat room
///
/// <https://dev.twitch.tv/docs/api/reference/#add-channel-moderator>
pub async fn mod_twitch_user(client: &Client, query: BroadcasterQuery) -> Result<()> {
    let url = format!("{TWITCH_API_BASE_URL}/moderation/moderators");
    request_bodiless(client, Method::POST, url, query).await
}

/// Removes a moderator from the broadcaster's chat room
///
/// <https://dev.twitch.tv/docs/api/reference/#remove-channel-moderator>
pub async fn unmod_twitch_user(client: &Client, query: BroadcasterQuery) -> Result<()> {
    let url = format!("{TWITCH_API_BASE_URL}/moderation/moderators");
    request_bodiless(client, Method::DELETE, url, query).await
}
