use anyhow::{Context, Result};
use futures::StreamExt;
use log::warn;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client,
};
use serde::Deserialize;
use std::{borrow::BorrowMut, collections::HashMap, path::Path};
use tokio::io::AsyncWriteExt;

use crate::{handlers::config::CompleteConfig, utils::pathing::cache_path};

// HashMap of emote name, emote filename, emote url, and if the emote is an overlay
type EmoteMap = HashMap<String, (String, String, bool)>;

#[derive(Deserialize)]
struct StringAttribute {
    #[serde(rename = "id", alias = "client_id", alias = "url_1x")]
    value: String,
}

#[derive(Deserialize)]
struct VecAttribute<T> {
    #[serde(rename = "data", alias = "emotes")]
    value: Vec<T>,
}

#[derive(Deserialize)]
struct TwitchEmote {
    id: String,
    name: String,
    images: StringAttribute,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BetterTTVEmote {
    id: String,
    code: String,
    image_type: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BetterTTVEmotes {
    channel_emotes: Vec<BetterTTVEmote>,
    shared_emotes: Vec<BetterTTVEmote>,
}

#[derive(Deserialize)]
struct SevenTVEmoteSet {
    emote_set: StringAttribute,
}

#[derive(Deserialize)]
struct SevenTVEmote {
    name: String,
    id: String,
    flags: u64,
}

async fn get_twitch_client_id(token: &str) -> Result<String> {
    let client = Client::new();

    Ok(client
        .get("https://id.twitch.tv/oauth2/validate")
        .header(AUTHORIZATION, &format!("OAuth {token}"))
        .send()
        .await?
        .error_for_status()?
        .json::<StringAttribute>()
        .await?
        .value)
}

async fn get_twitch_client(config: &CompleteConfig) -> Result<Client> {
    let token = config
        .twitch
        .token
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

async fn get_channel_id(client: &Client, channel: &str) -> Result<i32> {
    Ok(client
        .get(format!("https://api.twitch.tv/helix/users?login={channel}",))
        .send()
        .await?
        .error_for_status()?
        .json::<VecAttribute<StringAttribute>>()
        .await?
        .value
        .first()
        .context("Could not get channel id.")?
        .value
        .parse()?)
}

async fn get_twitch_emotes(client: &Client, channel_id: i32) -> Result<EmoteMap> {
    let channel_emotes = client
        .get(format!(
            "https://api.twitch.tv/helix/chat/emotes?broadcaster_id={channel_id}",
        ))
        .send()
        .await?
        .error_for_status()?
        .json::<VecAttribute<TwitchEmote>>()
        .await?
        .value;

    let global_emotes = client
        .get("https://api.twitch.tv/helix/chat/emotes/global")
        .send()
        .await?
        .error_for_status()?
        .json::<VecAttribute<TwitchEmote>>()
        .await?
        .value;

    Ok(channel_emotes
        .into_iter()
        .chain(global_emotes)
        .map(|emote| (emote.name, (emote.id, emote.images.value, false)))
        .collect())
}

async fn get_betterttv_emotes(channel_id: i32) -> Result<EmoteMap> {
    let client = Client::new();

    let BetterTTVEmotes {
        channel_emotes,
        shared_emotes,
    } = client
        .get(format!(
            "https://api.betterttv.net/3/cached/users/twitch/{channel_id}",
        ))
        .send()
        .await?
        .error_for_status()?
        .json::<BetterTTVEmotes>()
        .await?;

    let global_emotes: Vec<BetterTTVEmote> = client
        .get("https://api.betterttv.net/3/cached/emotes/global")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(channel_emotes
        .into_iter()
        .chain(shared_emotes)
        .chain(global_emotes)
        .map(
            |BetterTTVEmote {
                 code,
                 id,
                 image_type,
             }| {
                (
                    code,
                    (
                        format!("{id}.{image_type}"),
                        format!("https://cdn.betterttv.net/emote/{id}/1x.{image_type}"),
                        false,
                    ),
                )
            },
        )
        .collect())
}

async fn get_7tv_emotes(channel_id: i32) -> Result<EmoteMap> {
    let client = Client::new();

    let set = client
        .get(format!("https://7tv.io/v3/users/twitch/{channel_id}",))
        .send()
        .await?
        .error_for_status()?
        .json::<SevenTVEmoteSet>()
        .await?
        .emote_set
        .value;

    let channel_emotes = client
        .get(format!("https://7tv.io/v3/emote-sets/{set}",))
        .send()
        .await?
        .error_for_status()?
        .json::<VecAttribute<SevenTVEmote>>()
        .await?
        .value;

    let global_emotes = client
        .get("https://7tv.io/v3/emote-sets/global")
        .send()
        .await?
        .error_for_status()?
        .json::<VecAttribute<SevenTVEmote>>()
        .await?
        .value;

    Ok(channel_emotes
        .into_iter()
        .chain(global_emotes)
        .map(|SevenTVEmote { name, id, flags }| {
            (
                name,
                (
                    format!("{id}.webp"),
                    format!("https://cdn.7tv.app/emote/{id}/1x.webp"),
                    flags == 1,
                ),
            )
        })
        .collect())
}

async fn download_emotes(emotes: EmoteMap) -> HashMap<String, (String, bool)> {
    let client = &Client::new();

    // We need to limit the number of concurrent connections, otherwise we might hit some system limits
    // ex: number of files/sockets open, etc.
    let stream = futures::stream::iter(emotes.into_iter().map(
        |(x, (filename, url, o))| async move {
            let path = cache_path(&filename);
            let path = Path::new(&path);

            if tokio::fs::metadata(&path).await.is_ok() {
                return Ok((x, (filename, o)));
            }

            let mut res = client.get(&url).send().await?.error_for_status()?;

            let mut file = tokio::fs::File::create(&path).await?;

            while let Some(mut item) = res.chunk().await? {
                file.write_all_buf(item.borrow_mut()).await?;
            }

            Ok((x, (filename, o)))
        },
    ))
    .buffer_unordered(100);

    stream
        .collect::<Vec<Result<(String, (String, bool))>>>()
        .await
        .into_iter()
        .filter_map(Result::ok)
        .collect()
}

pub async fn get_emotes(
    config: &CompleteConfig,
    channel: &str,
) -> Result<HashMap<String, (String, bool)>> {
    // Reuse the same client and headers for twitch requests
    let twitch_client = get_twitch_client(config).await?;

    let channel_id = get_channel_id(&twitch_client, channel).await?;

    let mut emotes = HashMap::new();

    if config.frontend.twitch_emotes {
        match get_twitch_emotes(&twitch_client, channel_id).await {
            Ok(e) => emotes.extend(e),
            Err(err) => warn!("Unable to get list of Twitch emotes: {err}"),
        }
    }

    if config.frontend.betterttv_emotes {
        match get_betterttv_emotes(channel_id).await {
            Ok(e) => emotes.extend(e),
            Err(err) => warn!("Unable to get list of BetterTTV emotes: {err}"),
        }
    }

    if config.frontend.seventv_emotes {
        match get_7tv_emotes(channel_id).await {
            Ok(e) => emotes.extend(e),
            Err(err) => warn!("Unable to get list of 7tv emotes: {err}"),
        }
    }

    Ok(download_emotes(emotes).await)
}
