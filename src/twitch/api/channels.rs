use color_eyre::{Result, eyre::ContextCompat};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::TWITCH_API_BASE_URL;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Channel {
    id: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct ChannelList {
    data: Vec<Channel>,
}

/// Gets the channel ID of the specified channel name
///
/// <https://dev.twitch.tv/docs/api/reference/#get-users>
pub async fn get_channel_id(client: &Client, channel: &str) -> Result<String> {
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
        .clone();

    Ok(response_channel_id)
}
