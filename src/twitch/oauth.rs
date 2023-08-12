use anyhow::{Context, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client,
};
use serde::Deserialize;

use crate::handlers::config::TwitchConfig;

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

#[derive(Deserialize, Debug, Clone, Default)]
#[allow(dead_code)]
pub struct FollowingUser {
    broadcaster_id: String,
    pub broadcaster_login: String,
    pub broadcaster_name: String,
    followed_at: String,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[allow(dead_code)]
struct Pagination {
    cursor: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[allow(dead_code)]
pub struct FollowingList {
    total: u64,
    pub data: Vec<FollowingUser>,
    pagination: Pagination,
}

const FOLLOWER_COUNT: usize = 100;

// https://dev.twitch.tv/docs/api/reference/#get-followed-channels
pub async fn get_user_following(client: &Client, user_id: i32) -> Result<FollowingList> {
    Ok(client
        .get(format!(
            "https://api.twitch.tv/helix/channels/followed?user_id={user_id}&first={FOLLOWER_COUNT}",
        ))
        .send()
        .await?
        .error_for_status()?
        .json::<FollowingList>()
        .await?)
}

pub async fn get_following(twitch_config: &TwitchConfig) -> Result<FollowingList> {
    let oauth_token = twitch_config.token.clone();
    let app_user = twitch_config.username.clone();

    let client = get_twitch_client(oauth_token).await.unwrap();

    let user_id = get_channel_id(&client, &app_user).await.unwrap();

    get_user_following(&client, user_id).await
}
