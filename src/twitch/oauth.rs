use std::sync::OnceLock;

use color_eyre::{Result, eyre::ContextCompat};
use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TwitchOauth {
    pub client_id: String,
    pub login: String,
    pub scopes: Vec<String>,
    pub user_id: String,
    pub expires_in: i32,
}

pub async fn get_twitch_client_oauth(oauth_token: Option<&String>) -> Result<TwitchOauth> {
    static TWITCH_CLIENT_OAUTH: OnceLock<TwitchOauth> = OnceLock::new();

    if let Some(twitch_oauth) = TWITCH_CLIENT_OAUTH.get() {
        return Ok(twitch_oauth.clone());
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

    let twitch_oauth = data.json::<TwitchOauth>().await?;

    Ok(TWITCH_CLIENT_OAUTH.get_or_init(|| twitch_oauth)).cloned()
}

pub async fn get_twitch_client(
    twitch_oauth: &TwitchOauth,
    oauth_token: Option<&String>,
) -> Result<Client> {
    static TWITCH_CLIENT: OnceLock<Client> = OnceLock::new();

    if let Some(twitch_client) = TWITCH_CLIENT.get() {
        return Ok(twitch_client.clone());
    }

    let token = oauth_token
        .context("Twitch token is empty")?
        .strip_prefix("oauth:")
        .context("token does not start with `oauth:`")?;

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}"))?,
    );
    headers.insert("Client-Id", HeaderValue::from_str(&twitch_oauth.client_id)?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json")?);

    let twitch_client = Client::builder().default_headers(headers).build()?;

    Ok(TWITCH_CLIENT.get_or_init(|| twitch_client)).cloned()
}
