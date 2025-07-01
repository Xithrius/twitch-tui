use color_eyre::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::TWITCH_API_BASE_URL;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VipQuery {
    broadcaster_id: String,
    user_id: String,
}

impl VipQuery {
    pub const fn new(broadcaster_id: String, user_id: String) -> Self {
        Self {
            broadcaster_id,
            user_id,
        }
    }
}

/// Adds the specified user as a VIP in the broadcaster's channel
///
/// <https://dev.twitch.tv/docs/api/reference/#add-channel-vip>
pub async fn vip_twitch_user(client: &Client, query: VipQuery) -> Result<()> {
    let url = format!("{TWITCH_API_BASE_URL}/channels/vips");

    let vip_query = &[
        ("user_id", query.user_id),
        ("broadcaster_id", query.broadcaster_id),
    ];

    client
        .post(url)
        .query(vip_query)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

/// Removes the specified user as a VIP in the broadcaster's channel
///
/// <https://dev.twitch.tv/docs/api/reference/#remove-channel-vip>
pub async fn unvip_twitch_user(client: &Client, query: VipQuery) -> Result<()> {
    let url = format!("{TWITCH_API_BASE_URL}/channels/vips");

    let unvip_query = &[
        ("user_id", query.user_id),
        ("broadcaster_id", query.broadcaster_id),
    ];

    client
        .delete(url)
        .query(unvip_query)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
