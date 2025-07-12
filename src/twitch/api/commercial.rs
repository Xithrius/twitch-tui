use color_eyre::{Result, eyre::ContextCompat};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::TWITCH_API_BASE_URL;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CommercialPayload {
    broadcaster_id: String,
    length: usize,
}

impl CommercialPayload {
    pub const fn new(broadcaster_id: String, length: usize) -> Self {
        Self {
            broadcaster_id,
            length,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TwitchCommercialResponse {
    length: usize,
    message: String,
    retry_after: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct TwitchCommercialResponseList {
    data: Vec<TwitchCommercialResponse>,
}

/// Starts a commercial on the specified channel
///
/// <https://dev.twitch.tv/docs/api/reference/#start-commercial>
pub async fn start_commercial(
    client: &Client,
    payload: CommercialPayload,
) -> Result<TwitchCommercialResponse> {
    let url = format!("{TWITCH_API_BASE_URL}/channels/commercial");

    let response_data = client
        .post(url)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?
        .json::<TwitchCommercialResponseList>()
        .await?
        .data
        .first()
        .context("Could not get Twitch commercial response")?
        .clone();

    Ok(response_data)
}
