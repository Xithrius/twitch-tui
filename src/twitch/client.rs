use color_eyre::Result;
use reqwest::Client;

use super::oauth::{TwitchOauth, get_twitch_client};
use crate::handlers::config::TwitchConfig;

pub struct TwitchClient {
    client: Client,
    oauth: TwitchOauth,
    session_id: Option<String>,
}

impl TwitchClient {
    pub async fn new(
        twitch_config: &TwitchConfig,
        oauth: TwitchOauth,
        session_id: String,
    ) -> Result<Self> {
        let client = get_twitch_client(&oauth, twitch_config.token.as_deref()).await?;

        Ok(Self {
            client,
            oauth,
            session_id: Some(session_id),
        })
    }
}
