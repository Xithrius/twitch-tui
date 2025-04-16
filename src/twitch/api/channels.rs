use color_eyre::{Result, eyre::ContextCompat};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::TWITCH_API_BASE_URL;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Channel {
    id: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChannelList {
    data: Vec<Channel>,
}

/// <https://dev.twitch.tv/docs/api/reference/#get-users>
pub async fn get_channel_id(client: &Client, channel: &str) -> Result<i32> {
    let response_channel_id = client
        .get(format!("{TWITCH_API_BASE_URL}/users?login={channel}"))
        .send()
        .await?
        .error_for_status()?
        .json::<ChannelList>()
        .await?
        .data
        .first()
        .context("Could not get channel id.")?
        .id
        .parse()?;

    Ok(response_channel_id)
}
