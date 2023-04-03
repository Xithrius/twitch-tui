use crate::handlers::config::CompleteConfig;
use crate::utils::pathing::cache_path;
use anyhow::{Context, Result};
use futures::StreamExt;
use log::warn;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{header, Client};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::path::Path;
use tokio::io::AsyncWriteExt;

// HashMap of emote name, emote filename, emote url
type EmoteMap = HashMap<String, (String, String)>;

fn get_twitch_client(config: &CompleteConfig) -> Result<Client> {
    let api_token = config
        .twitch
        .api
        .as_ref()
        .context("Twitch api token is empty")?;
    let api_id = config
        .twitch
        .id
        .as_ref()
        .context("Twitch api id is empty")?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {api_token}"))?,
    );
    headers.insert("Client-Id", HeaderValue::from_str(api_id)?);

    Ok(Client::builder().default_headers(headers).build()?)
}

async fn get_channel_id(client: &Client, channel: &str) -> Result<i32> {
    let res = client
        .get(format!("https://api.twitch.tv/helix/users?login={channel}",))
        .send()
        .await?
        .error_for_status()?;

    let json = json::parse(res.text().await?.as_str())?;
    Ok(json["data"][0]["id"].to_string().parse()?)
}

async fn get_twitch_emotes(client: &Client, channel_id: i32) -> Result<EmoteMap> {
    let res = client
        .get(format!(
            "https://api.twitch.tv/helix/chat/emotes?broadcaster_id={channel_id}",
        ))
        .send()
        .await?
        .error_for_status()?;

    let json = json::parse(&res.text().await?)?;
    let channel_emotes = &json["data"];

    let res = client
        .get("https://api.twitch.tv/helix/chat/emotes/global")
        .send()
        .await?
        .error_for_status()?;

    let json = json::parse(&res.text().await?)?;
    let global_emotes = &json["data"];

    let mut emotes_map = HashMap::new();

    for twitch_emote in [channel_emotes, global_emotes] {
        if let json::JsonValue::Array(emotes) = twitch_emote {
            emotes_map.extend(emotes.iter().map(|emote| {
                let id = emote["id"].to_string();
                let url =
                    format!("https://static-cdn.jtvnw.net/emoticons/v2/{id}/static/light/1.0");

                (emote["name"].to_string(), (id, url))
            }));
        }
    }

    Ok(emotes_map)
}

async fn get_betterttv_emotes(channel_id: i32) -> Result<EmoteMap> {
    let client = Client::new();

    let res = client
        .get(format!(
            "https://api.betterttv.net/3/cached/users/twitch/{channel_id}",
        ))
        .send()
        .await?
        .error_for_status()?;

    let json = json::parse(&res.text().await?)?;

    let channel_emotes = &json["channelEmotes"];
    let shared_emotes = &json["sharedEmotes"];

    let res = client
        .get("https://api.betterttv.net/3/cached/emotes/global")
        .send()
        .await?
        .error_for_status()?;

    let global_emotes = &json::parse(&res.text().await?)?;

    let mut emotes_map = HashMap::new();

    for bttv_emote in [channel_emotes, shared_emotes, global_emotes] {
        if let json::JsonValue::Array(emotes) = bttv_emote {
            emotes_map.extend(emotes.iter().map(|emote| {
                let id = emote["id"].to_string();
                let image_type = emote["imageType"].to_string();

                (
                    emote["code"].to_string(),
                    (
                        format!("{id}.{image_type}"),
                        format!("https://cdn.betterttv.net/emote/{id}/1x.{image_type}"),
                    ),
                )
            }));
        }
    }

    Ok(emotes_map)
}

async fn get_7tv_emotes(channel_id: i32) -> Result<EmoteMap> {
    let client = Client::new();

    let res = client
        .get(format!("https://7tv.io/v3/users/twitch/{channel_id}",))
        .send()
        .await?
        .error_for_status()?;

    let json = json::parse(&res.text().await?)?;
    let set = &json["emote_set"]["id"];

    let res = client
        .get(format!("https://7tv.io/v3/emote-sets/{set}",))
        .send()
        .await?
        .error_for_status()?;

    let json = json::parse(&res.text().await?)?;
    let channel_emotes = &json["emotes"];

    let res = client
        .get("https://7tv.io/v3/emote-sets/global")
        .send()
        .await?
        .error_for_status()?;

    let json = json::parse(&res.text().await?)?;
    let global_emotes = &json["emotes"];

    let mut emotes_map = HashMap::new();

    for seventv_emotes in [channel_emotes, global_emotes] {
        if let json::JsonValue::Array(emotes) = seventv_emotes {
            emotes_map.extend(emotes.iter().map(|emote| {
                let id = emote["id"].to_string();

                (
                    emote["name"].to_string(),
                    (
                        format!("{id}.webp"),
                        format!("https://cdn.7tv.app/emote/{id}/1x.webp"),
                    ),
                )
            }));
        }
    }

    Ok(emotes_map)
}

async fn download_emotes(emotes: EmoteMap) -> HashMap<String, String> {
    let client = &Client::new();

    // We need to limit the number of concurrent connections, otherwise we might hit some system limits
    // ex: number of files/sockets open, etc.
    let stream = futures::stream::iter(emotes.into_iter().map(|(x, (filename, url))| async move {
        let path = cache_path(&filename);
        let path = Path::new(&path);

        if tokio::fs::metadata(&path).await.is_ok() {
            return Ok((x, filename));
        }

        let mut res = client.get(&url).send().await?.error_for_status()?;

        let mut file = tokio::fs::File::create(&path).await?;

        while let Some(mut item) = res.chunk().await? {
            file.write_all_buf(item.borrow_mut()).await?;
        }

        Ok((x, filename))
    }))
    .buffer_unordered(100);

    stream
        .collect::<Vec<Result<(String, String)>>>()
        .await
        .into_iter()
        .filter_map(Result::ok)
        .collect()
}

pub async fn get_emotes(config: &CompleteConfig) -> Result<HashMap<String, String>> {
    // Reuse the same client and headers for twitch requests
    let twitch_client = get_twitch_client(config)?;

    let channel_id = get_channel_id(&twitch_client, &config.twitch.channel).await?;

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
