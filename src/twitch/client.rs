#![allow(dead_code)]

use color_eyre::Result;
use reqwest::Client;

use super::oauth::{TwitchOauth, get_twitch_client, get_twitch_client_oauth};
use crate::handlers::config::TwitchConfig;

pub struct TwitchClient {
    client: Client,
    oauth: TwitchOauth,
    session_id: String,
}

impl TwitchClient {
    pub async fn new(twitch_config: &TwitchConfig, session_id: String) -> Result<Self> {
        let token = twitch_config.token.clone();

        let oauth = get_twitch_client_oauth(token.as_ref()).await?;
        let client = get_twitch_client(&oauth, token.as_ref()).await?;

        let twitch_client = Self {
            client,
            oauth,
            session_id,
        };

        Ok(twitch_client)
    }
}
