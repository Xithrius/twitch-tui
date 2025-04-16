use std::sync::OnceLock;

use color_eyre::{Result, eyre::ContextCompat};
use reqwest::{
    Client,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TwitchOauth {
    pub client_id: String,
    pub login: String,
    pub scopes: Vec<String>,
    pub user_id: String,
    pub expires_in: i32,
}

pub async fn get_twitch_client_oauth(oauth_token: Option<&str>) -> Result<&TwitchOauth> {
    static TWITCH_CLIENT_ID: OnceLock<TwitchOauth> = OnceLock::new();

    if let Some(id) = TWITCH_CLIENT_ID.get() {
        return Ok(id);
    }

    let token = oauth_token
        .context("Twitch token is empty")?
        .strip_prefix("oauth:")
        .context("token does not start with `oauth:`")?;

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

    let client_id = data.json::<TwitchOauth>().await?;

    Ok(TWITCH_CLIENT_ID.get_or_init(|| client_id))
}

pub async fn get_twitch_client(
    client_id: &TwitchOauth,
    oauth_token: Option<&str>,
) -> Result<Client> {
    let token = oauth_token
        .context("Twitch token is empty")?
        .strip_prefix("oauth:")
        .context("token does not start with `oauth:`")?;

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}"))?,
    );
    headers.insert("Client-Id", HeaderValue::from_str(&client_id.client_id)?);

    Ok(Client::builder().default_headers(headers).build()?)
}
