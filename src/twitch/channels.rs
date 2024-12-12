use std::{
    fmt::Display,
    string::{String, ToString},
    vec::Vec,
};

use color_eyre::Result;
use serde::Deserialize;

use crate::{handlers::config::TwitchConfig, ui::components::utils::SearchItemGetter};

use super::oauth::{get_twitch_client, get_twitch_client_id};

const FOLLOWER_COUNT: usize = 100;

#[derive(Deserialize, Debug, Clone, Default)]
#[allow(dead_code)]
pub struct FollowingUser {
    broadcaster_id: String,
    pub broadcaster_login: String,
    pub broadcaster_name: String,
    followed_at: String,
}

//    id 	String 	An ID that identifies the stream. You can use this ID later to look up the video on demand (VOD).
//    user_id 	String 	The ID of the user that’s broadcasting the stream.
//    user_login 	String 	The user’s login name.
//    user_name 	String 	The user’s display name.
//    game_id 	String 	The ID of the category or game being played.
//    game_name 	String 	The name of the category or game being played.
//    type 	String 	The type of stream. Possible values are:
//
//     live
//
// If an error occurs, this field is set to an empty string.
//    title 	String 	The stream’s title. Is an empty string if not set.
//    tags 	String[] 	The tags applied to the stream.
//    viewer_count 	Integer 	The number of users watching the stream.
//    started_at 	String 	The UTC date and time (in RFC3339 format) of when the broadcast began.
//    language 	String 	The language that the stream uses. This is an ISO 639-1 two-letter language code or other if the stream uses a language not in the list of supported stream languages.
//    thumbnail_url 	String 	A URL to an image of a frame from the last 5 minutes of the stream. Replace the width and height placeholders in the URL ({width}x{height}) with the size of the image you want, in pixels.
//    tag_ids 	String[] 	IMPORTANT As of February 28, 2023, this field is deprecated and returns only an empty array. If you use this field, please update your code to use the tags field.
//
// The list of tags that apply to the stream. The list contains IDs only when the channel is steaming live. For a list of possible tags, see List of All Tags. The list doesn’t include Category Tags.
//    is_mature 	Boolean 	A Boolean value that indicates whether the stream is meant for mature audiences.
#[derive(Deserialize, Debug, Clone, Default)]
#[allow(dead_code)]
pub struct StreamingUser {
    id: String,
    // pub user_id: String,
    // pub user_login: String,
    pub user_name: String,
    // pub game_id: String,
    // pub game_name: String,
    #[serde(rename = "name")]
    pub s_type: String,
}

impl Display for StreamingUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.user_name)
    }
}

// data 	Object[] 	The list of streams.
// pagination 	Object 	The information used to page through the list of results. The object is empty if there are no more pages left to page through. Read More
//    cursor 	String 	The cursor used to get the next page of results. Set the request’s after or before query parameter to this value depending on whether you’re paging forwards or backwards.

#[derive(Deserialize, Debug, Clone, Default)]
#[allow(dead_code)]
pub struct StreamingList {
    pub data: Vec<StreamingUser>,
    pagination: Pagination,
}

impl Display for FollowingUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.broadcaster_login)
    }
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
#[allow(dead_code)]
pub struct Following {
    // TODO: Don't re-create client on new requests
    // client: &Client,
    twitch_config: TwitchConfig,
    list: FollowingList,
}

// https://dev.twitch.tv/docs/api/reference/#get-followed-channels
pub async fn get_following(twitch_config: &TwitchConfig) -> Result<FollowingList> {
    let client = get_twitch_client(twitch_config.token.as_deref()).await?;
    let user_id = &get_twitch_client_id(None).await?.user_id;

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

pub async fn get_streams(twitch_config: &TwitchConfig) -> Result<StreamingList> {
    let client = get_twitch_client(twitch_config.token.as_deref()).await?;
    let user_id = &get_twitch_client_id(None).await?.user_id;

    Ok(client
        .get(format!(
            "https://api.twitch.tv/helix/streams?user_id={user_id}&type=live&first={FOLLOWER_COUNT}",
        ))
        .send()
        .await?
        .error_for_status()?
        .json::<StreamingList>()
        .await?)
}

impl Following {
    pub fn new(twitch_config: TwitchConfig) -> Self {
        Self {
            twitch_config,
            list: FollowingList::default(),
        }
    }
}

impl SearchItemGetter<String> for Following {
    async fn get_items(&mut self) -> Result<Vec<String>> {
        let following = get_following(&self.twitch_config).await;
        let streaming = get_streams(&self.twitch_config).await;

        following.map(|v| {
            v.data
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
        })
    }
}
