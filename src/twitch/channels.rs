use std::{string::String, vec::Vec};

use reqwest::Client;
use serde::Deserialize;
use tokio::{runtime::Handle, task};

use crate::handlers::config::TwitchConfig;

use super::oauth::{get_channel_id, get_twitch_client};

const FOLLOWER_COUNT: usize = 100;

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
    pub total: u64,
    pub data: Vec<FollowingUser>,
    pagination: Pagination,
}

#[derive(Debug, Clone)]
pub struct Following {
    client: Client,
    twitch_config: TwitchConfig,
    list: FollowingList,
}

pub trait ItemGetter<T>
where
    T: ToString,
{
    fn get_items(&mut self) -> Vec<T>;
}

impl Following {
    pub fn new(twitch_config: TwitchConfig) -> Self {
        let token = twitch_config.token.clone();

        let client = task::block_in_place(move || {
            Handle::current().block_on(async move { get_twitch_client(token).await.unwrap() })
        });

        Self {
            client,
            twitch_config,
            list: FollowingList::default(),
        }
    }

    // https://dev.twitch.tv/docs/api/reference/#get-followed-channels
    pub async fn get_user_following(&self, user_id: i32) -> FollowingList {
        self.client
        .get(format!(
            "https://api.twitch.tv/helix/channels/followed?user_id={user_id}&first={FOLLOWER_COUNT}",
        ))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json::<FollowingList>()
        .await
        .unwrap()
    }

    pub async fn get_following(&self) -> FollowingList {
        let app_user = self.twitch_config.username.clone();

        let user_id = get_channel_id(&self.client, &app_user).await.unwrap();

        self.get_user_following(user_id).await
    }

    pub fn get_followed_channels(self) -> FollowingList {
        task::block_in_place(move || {
            Handle::current().block_on(async move { self.get_following().await })
        })
    }
}
