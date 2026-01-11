pub mod channel_information;
pub mod channels;
pub mod chat_settings;
pub mod clear;
pub mod commercial;
pub mod event_sub;
pub mod following;
pub mod messages;
pub mod mods;
pub mod raids;
pub mod shoutouts;
pub mod subscriptions;
pub mod timeouts;
pub mod vips;

use color_eyre::Result;
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};

pub static TWITCH_API_BASE_URL: &str = "https://api.twitch.tv/helix";

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BroadcasterQuery {
    broadcaster_id: String,
    user_id: String,
}

impl BroadcasterQuery {
    pub const fn new(broadcaster_id: String, user_id: String) -> Self {
        Self {
            broadcaster_id,
            user_id,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ModeratorQuery {
    broadcaster_id: String,
    moderator_id: String,
}

impl ModeratorQuery {
    pub const fn new(broadcaster_id: String, moderator_id: String) -> Self {
        Self {
            broadcaster_id,
            moderator_id,
        }
    }
}

pub async fn request_bodiless<T: Serialize>(
    client: &Client,
    method: Method,
    url: String,
    query: T,
) -> Result<()> {
    client
        .request(method, url)
        .query(&query)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ResponseList<T> {
    data: Vec<T>,
}
