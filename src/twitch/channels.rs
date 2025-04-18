use std::{
    string::{String, ToString},
    vec::Vec,
};

use color_eyre::Result;

use super::{
    api::following::{FollowingList, get_user_following},
    oauth::{get_twitch_client, get_twitch_client_oauth},
};
use crate::{handlers::config::TwitchConfig, ui::components::utils::SearchItemGetter};

#[derive(Debug, Clone)]
pub struct Following {
    twitch_config: TwitchConfig,
}

// TODO: Authentication and requests in general should not be done on the UI side of things
pub async fn get_following(twitch_config: &TwitchConfig) -> Result<FollowingList> {
    let client_id = get_twitch_client_oauth(None).await?;
    let client = get_twitch_client(&client_id, twitch_config.token.as_ref()).await?;

    get_user_following(&client, &client_id.user_id).await
}

impl Following {
    pub const fn new(twitch_config: TwitchConfig) -> Self {
        Self { twitch_config }
    }
}

impl SearchItemGetter<String> for Following {
    async fn get_items(&mut self) -> Result<Vec<String>> {
        let following = get_following(&self.twitch_config).await;

        following.map(|v| {
            v.data
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
        })
    }
}
