use std::marker::PhantomData;

use color_eyre::Result;
use reqwest::Client;

use super::oauth::{TwitchOauth, get_twitch_client, get_twitch_client_oauth};
use crate::handlers::config::TwitchConfig;

struct Authenticated;
struct Unauthenticated;

pub struct TwitchClient<State> {
    client: Client,
    oauth: TwitchOauth,
    session_id: String,
    state: PhantomData<State>,
}

impl TwitchClient<Unauthenticated> {
    pub async fn new(
        twitch_config: &TwitchConfig,
        session_id: String,
    ) -> Result<TwitchClient<Authenticated>> {
        let token = twitch_config.token.as_deref();

        let oauth = get_twitch_client_oauth(token).await?;
        let client = get_twitch_client(&oauth, token).await?;

        let twitch_client = TwitchClient {
            client,
            oauth,
            session_id,
            state: PhantomData,
        };

        Ok(twitch_client)
    }
}

impl TwitchClient<Authenticated> {}
