use color_eyre::Result;
use reqwest::{Client, Method};

use super::{BroadcasterQuery, TWITCH_API_BASE_URL, request_bodiless};

/// Adds the specified user as a VIP in the broadcaster's channel
///
/// <https://dev.twitch.tv/docs/api/reference/#add-channel-vip>
pub async fn vip_twitch_user(client: &Client, query: BroadcasterQuery) -> Result<()> {
    let url = format!("{TWITCH_API_BASE_URL}/channels/vips");
    request_bodiless(client, Method::POST, url, query).await
}

/// Removes the specified user as a VIP in the broadcaster's channel
///
/// <https://dev.twitch.tv/docs/api/reference/#remove-channel-vip>
pub async fn unvip_twitch_user(client: &Client, query: BroadcasterQuery) -> Result<()> {
    let url = format!("{TWITCH_API_BASE_URL}/channels/vips");
    request_bodiless(client, Method::DELETE, url, query).await
}
