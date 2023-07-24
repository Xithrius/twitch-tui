use anyhow::{Context, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client,
};
use serde::Deserialize;

use crate::handlers::config::CompleteConfig;

#[derive(Deserialize)]
pub struct ClientId {
    client_id: String,
}

pub async fn get_twitch_client_id(token: &str) -> Result<String> {
    let client = Client::new();

    Ok(client
        .get("https://id.twitch.tv/oauth2/validate")
        .header(AUTHORIZATION, &format!("OAuth {token}"))
        .send()
        .await?
        .error_for_status()?
        .json::<ClientId>()
        .await?
        .client_id)
}

pub async fn get_twitch_client(config: &CompleteConfig) -> Result<Client> {
    let token = config
        .twitch
        .token
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
