use std::sync::Arc;

use color_eyre::{
    Result,
    eyre::{ContextCompat, bail},
};
use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::config::SharedCoreConfig;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TwitchOauthInner {
    client_id: String,
    login: String,
    scopes: Vec<String>,
    user_id: String,
    expires_in: i32,
}

#[derive(Debug, Clone, Default)]
pub struct TwitchOauth {
    inner_oauth: Option<Arc<TwitchOauthInner>>,
    inner_client: Option<Client>,
}

impl TwitchOauth {
    pub async fn init(&mut self, config: SharedCoreConfig) -> Result<Self> {
        let token = config.twitch.token.as_ref();
        self.init_oauth(token).await?;
        self.init_client(token).await?;

        Ok(self.to_owned())
    }

    async fn init_oauth(&mut self, token: Option<&String>) -> Result<()> {
        if self.inner_oauth.is_some() {
            warn!("Twitch OAuth tried to re-initialize. Maybe a second call happened somewhere?");
            return Ok(());
        }

        let token = token
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
            .error_for_status()?;

        let twitch_oauth = data.json::<TwitchOauthInner>().await?;

        info!(
            "Authentication successful. Enabled scopes: {:?}",
            twitch_oauth.scopes
        );

        self.inner_oauth = Some(Arc::new(twitch_oauth));

        Ok(())
    }

    async fn init_client(&mut self, token: Option<&String>) -> Result<()> {
        let Some(twitch_oauth) = self.inner_oauth.as_ref() else {
            bail!(
                "Twitch OAuth was not initialized (successfully?) before attempting to initialize the client."
            );
        };

        let token = token
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

        self.inner_client = Some(twitch_client);

        Ok(())
    }

    pub fn user_id(&self) -> Option<String> {
        self.inner_oauth.as_ref().map(|oauth| oauth.user_id.clone())
    }

    pub fn client(&self) -> Option<Client> {
        self.inner_client.clone()
    }

    pub fn client_id(&self) -> Option<String> {
        self.inner_oauth
            .as_ref()
            .map(|oauth| oauth.client_id.clone())
    }
}
