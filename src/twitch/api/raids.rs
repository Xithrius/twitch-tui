use chrono::{DateTime, Utc};
use color_eyre::{Result, eyre::ContextCompat};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::TWITCH_API_BASE_URL;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RaidQuery {
    from_broadcaster_id: String,
    to_broadcaster_id: String,
}

impl RaidQuery {
    pub const fn new(from_broadcaster_id: String, to_broadcaster_id: String) -> Self {
        Self {
            from_broadcaster_id,
            to_broadcaster_id,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TwitchRaidResponse {
    created_at: DateTime<Utc>,
    is_mature: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct TwitchRaidResponseList {
    data: Vec<TwitchRaidResponse>,
}

/// Raid another channel by sending the broadcaster's viewers to the targeted channel
///
/// <https://dev.twitch.tv/docs/api/reference/#start-a-raid>
pub async fn raid_twitch_user(client: &Client, query: RaidQuery) -> Result<TwitchRaidResponse> {
    let url = format!("{TWITCH_API_BASE_URL}/raids");

    let raid_query = &[
        ("from_broadcaster_id", query.from_broadcaster_id),
        ("to_broadcaster_id", query.to_broadcaster_id),
    ];

    let response_data = client
        .post(url)
        .query(raid_query)
        .send()
        .await?
        .error_for_status()?
        .json::<TwitchRaidResponseList>()
        .await?
        .data
        .first()
        .context("Could not get Twitch raid response")?
        .clone();

    Ok(response_data)
}

/// Cancel a pending raid
///
/// <https://dev.twitch.tv/docs/api/reference/#cancel-a-raid>
pub async fn unraid_twitch_user(client: &Client, broadcaster_id: String) -> Result<()> {
    let url = format!("{TWITCH_API_BASE_URL}/raids");

    let unraid_query = &[("broadcaster_id", broadcaster_id)];

    client
        .delete(url)
        .query(unraid_query)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}
