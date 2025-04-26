use std::{
    string::{String, ToString},
    vec::Vec,
};

use color_eyre::Result;

use super::api::following::{Following, get_following};
use crate::ui::components::utils::SearchItemGetter;

impl SearchItemGetter<String> for Following {
    async fn get_items(&mut self) -> Result<Vec<String>> {
        let following = get_following(
            &self.config.twitch,
            self.config.frontend.only_get_live_followed_channels,
        )
        .await;

        following.map(|v| {
            v.data
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
        })
    }
}
