use anyhow::{Context, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client,
};
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct ClientId {
    client_id: String,
    login: String,
    scopes: Vec<String>,
    user_id: String,
    expires_in: i32,
}

pub async fn get_twitch_client_id(token: &str) -> Result<String> {
    let client = Client::new();

    let data = client
        .get("https://id.twitch.tv/oauth2/validate")
        .header(AUTHORIZATION, &format!("OAuth {token}"))
        .send()
        .await?
        .error_for_status()
        .unwrap();

    let text = data.json::<ClientId>().await?;

    Ok(text.client_id)
}

pub async fn get_twitch_client(oauth_token: Option<String>) -> Result<Client> {
    let token = oauth_token
        .as_ref()
        .context("Twitch token is empty")?
        .strip_prefix("oauth:")
        .context("token does not start with `oauth:`")?;

    let client_id = get_twitch_client_id(token).await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}"))?,
    );
    headers.insert("Client-Id", HeaderValue::from_str(&client_id)?);

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
