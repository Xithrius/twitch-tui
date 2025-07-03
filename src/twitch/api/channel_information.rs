use color_eyre::{Result, eyre::ContextCompat};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::TWITCH_API_BASE_URL;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct GameResponse {
    id: String,
    name: String,
    box_art_url: String,
    igdb_id: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct GameResponseList {
    data: Vec<GameResponse>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct ChannelInformationLabel {
    id: String,
    is_enabled: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChannelInformationResponse {
    broadcaster_id: String,
    broadcaster_login: String,
    broadcaster_name: String,
    broadcaster_language: String,
    game_id: String,
    game_name: String,
    title: String,
    delay: usize,
    tags: Vec<String>,
    content_classification_labels: Vec<String>,
    is_branded_content: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct ChannelInformationResponseList {
    data: Vec<ChannelInformationResponse>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct UpdateChannelInformationPayload {
    game_id: Option<String>,
    broadcaster_language: Option<String>,
    title: Option<String>,
    delay: Option<usize>,
    tags: Option<Vec<String>>,
    content_classification_labels: Vec<ChannelInformationLabel>,
    is_branded_content: Option<bool>,
}

impl UpdateChannelInformationPayload {
    pub fn new_title(title: &str) -> Self {
        Self {
            title: Some(title.to_string()),
            ..Self::default()
        }
    }
    pub fn new_category(game_id: &str) -> Self {
        Self {
            game_id: Some(game_id.to_string()),
            ..Self::default()
        }
    }
}

/// Gets the game ID of the specified game name
///
/// <https://dev.twitch.tv/docs/api/reference/#get-games>
pub async fn get_game_id(client: &Client, game_name: &str) -> Result<String> {
    let response_game_id = client
        .get(format!("{TWITCH_API_BASE_URL}/games?name={game_name}"))
        .send()
        .await?
        .error_for_status()?
        .json::<GameResponseList>()
        .await?
        .data
        .first()
        .context("Could not get game id.")?
        .id
        .clone();

    Ok(response_game_id)
}

/// Gets information about a channel
///
/// <https://dev.twitch.tv/docs/api/reference/#get-channel-information>
#[allow(unused)]
pub async fn get_channel_information(
    client: &Client,
    broadcaster_id: String,
) -> Result<ChannelInformationResponse> {
    let url = format!("{TWITCH_API_BASE_URL}/channels?broadcaster_id={broadcaster_id}");

    let response_data = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json::<ChannelInformationResponseList>()
        .await?
        .data
        .first()
        .context("Could not get Twitch timeout response")?
        .clone();

    Ok(response_data)
}

/// Updates a channel's properties
///
/// <https://dev.twitch.tv/docs/api/reference/#modify-channel-information>
pub async fn update_channel_information(
    client: &Client,
    broadcaster_id: String,
    payload: UpdateChannelInformationPayload,
) -> Result<()> {
    let url = format!("{TWITCH_API_BASE_URL}/channels?broadcaster_id={broadcaster_id}");
    client
        .patch(url)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}
