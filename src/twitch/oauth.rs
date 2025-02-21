use std::sync::OnceLock;

use color_eyre::{Result, eyre::ContextCompat};
use reqwest::{
    Client,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct ClientId {
    pub client_id: String,
    pub login: String,
    pub scopes: Vec<String>,
    pub user_id: String,
    pub expires_in: i32,
}

pub async fn get_twitch_client_id(token: Option<&str>) -> Result<&ClientId> {
    static TWITCH_CLIENT_ID: OnceLock<ClientId> = OnceLock::new();

    if let Some(id) = TWITCH_CLIENT_ID.get() {
        return Ok(id);
    }

    let token = token.context("Twitch token is empty")?;

    // Strips the `oauth:` prefix if it exists
    let token = token.strip_prefix("oauth:").unwrap_or(token);

    let client = Client::new();

    let data = client
        .get("https://id.twitch.tv/oauth2/validate")
        .header(AUTHORIZATION, &format!("OAuth {token}"))
        .send()
        .await?
        .error_for_status()
        .unwrap();

    let client_id = data.json::<ClientId>().await?;

    Ok(TWITCH_CLIENT_ID.get_or_init(|| client_id))
}

pub async fn get_twitch_client(oauth_token: Option<&str>) -> Result<Client> {
    let token = oauth_token
        .context("Twitch token is empty")?
        .strip_prefix("oauth:")
        .context("token does not start with `oauth:`")?;

    let client_id = &get_twitch_client_id(Some(token)).await?.client_id;

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}"))?,
    );
    headers.insert("Client-Id", HeaderValue::from_str(client_id)?);

    Ok(Client::builder().default_headers(headers).build()?)
}

#[derive(Deserialize)]
struct Channel {
    id: String,
}

#[derive(Deserialize)]
pub struct ChannelList {
    data: Vec<Channel>,
}

pub async fn get_channel_id(client: &Client, channel: &str) -> Result<i32> {
    Ok(client
        .get(format!("https://api.twitch.tv/helix/users?login={channel}",))
        .send()
        .await?
        .error_for_status()?
        .json::<ChannelList>()
        .await?
        .data
        .first()
        .context("Could not get channel id.")?
        .id
        .parse()?)
}
