use crate::emotes::{Error, Result};
use crate::handlers::config::CompleteConfig;
use crate::utils::pathing::cache_path;
use futures::future;
use log::warn;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;

async fn get_user_id(config: &CompleteConfig) -> Result<String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "https://api.twitch.tv/helix/users?login={}",
            config.twitch.channel
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", config.twitch.api.as_ref().unwrap()),
        )
        .header("Client-Id", config.twitch.id.as_ref().unwrap())
        .send()
        .await?;

    let json = json::parse(resp.text().await?.as_str())?;
    Ok(json["data"][0]["id"].to_string())
}

async fn get_twitch_emotes(
    config: &CompleteConfig,
    id: &str,
) -> Result<HashMap<String, (String, String)>> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "https://api.twitch.tv/helix/chat/emotes?broadcaster_id={id}",
        ))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", config.twitch.api.as_ref().unwrap()),
        )
        .header("Client-Id", config.twitch.id.as_ref().unwrap())
        .send()
        .await?;

    let json = json::parse(resp.text().await?.as_str())?;
    let channel_emotes = json["data"].clone();

    let resp = client
        .get("https://api.twitch.tv/helix/chat/emotes/global")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", config.twitch.api.as_ref().unwrap()),
        )
        .header("Client-Id", config.twitch.id.as_ref().unwrap())
        .send()
        .await?;
    let json = json::parse(resp.text().await?.as_str())?;
    let global_emotes = json["data"].clone();

    let mut emotes = HashMap::new();

    for twitch_emote in &[channel_emotes, global_emotes] {
        if let json::JsonValue::Array(emote) = twitch_emote {
            for emote in emote {
                let id = emote["id"].to_string();
                emotes.insert(
                    emote["name"].to_string(),
                    (
                        id.clone(),
                        format!("https://static-cdn.jtvnw.net/emoticons/v2/{id}/static/light/1.0"),
                    ),
                );
            }
        }
    }

    Ok(emotes)
}

async fn get_betterttv_emotes(id: &str) -> Result<HashMap<String, (String, String)>> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "https://api.betterttv.net/3/cached/users/twitch/{id}",
        ))
        .send()
        .await?;

    let json = json::parse(resp.text().await?.as_str())?;
    let channel_emotes = json["channelEmotes"].clone();
    let shared_emotes = json["sharedEmotes"].clone();

    let resp = client
        .get("https://api.betterttv.net/3/cached/emotes/global")
        .send()
        .await?;
    let global_emotes = json::parse(resp.text().await?.as_str())?;

    let mut emotes = HashMap::new();

    for bttv_emote in &[channel_emotes, shared_emotes, global_emotes] {
        if let json::JsonValue::Array(emote) = bttv_emote {
            for emote in emote {
                let id = emote["id"].to_string();
                let image_type = emote["imageType"].to_string();
                emotes.insert(
                    emote["code"].to_string(),
                    (
                        format!("{id}.{image_type}"),
                        format!("https://cdn.betterttv.net/emote/{id}/1x.{image_type}"),
                    ),
                );
            }
        }
    }

    Ok(emotes)
}

async fn get_7tv_emotes(id: &str) -> Result<HashMap<String, (String, String)>> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("https://7tv.io/v3/users/twitch/{id}",))
        .send()
        .await?;

    let json = json::parse(resp.text().await?.as_str())?;
    let set = json["emote_set"]["id"].clone();

    let resp = client
        .get(format!("https://7tv.io/v3/emote-sets/{set}",))
        .send()
        .await?;
    let json = json::parse(resp.text().await?.as_str())?;
    let channel_emotes = json["emotes"].clone();

    let resp = client
        .get("https://7tv.io/v3/emote-sets/global")
        .send()
        .await?;
    let json = json::parse(resp.text().await?.as_str())?;
    let global_emotes = json["emotes"].clone();

    let mut emotes = HashMap::new();

    for sevent_tv in &[channel_emotes, global_emotes] {
        if let json::JsonValue::Array(emote) = sevent_tv {
            for emote in emote {
                let id = emote["id"].to_string();
                emotes.insert(
                    emote["name"].to_string(),
                    (
                        format!("{id}.webp"),
                        format!("https://cdn.7tv.app/emote/{id}/1x.webp"),
                    ),
                );
            }
        }
    }

    Ok(emotes)
}

async fn download_emotes(emotes: impl Iterator<Item = &(String, String)>) {
    let client = reqwest::Client::new();

    future::join_all(emotes.map(|(filename, url)| {
        let client = &client;

        let path = cache_path(filename);

        async move {
            if tokio::fs::metadata(path.clone()).await.is_ok() {
                return Ok::<(), Error>(());
            }

            let response = client.get(url).send().await?;

            let mut res = response.error_for_status()?;
            let mut file = tokio::fs::File::create(path).await?;

            while let Some(mut item) = res.chunk().await? {
                file.write_all_buf(item.borrow_mut()).await?;
            }
            Ok(())
        }
    }))
    .await;
}

pub async fn get_emotes(config: &CompleteConfig) -> Result<HashMap<String, String>> {
    match get_user_id(config).await {
        Ok(id) => {
            let mut emotes = if config.frontend.twitch_emotes {
                match get_twitch_emotes(config, &id).await {
                    Ok(emotes) => emotes,
                    Err(err) => {
                        warn!("Unable to get list of twitch emotes: {err:?}");
                        HashMap::new()
                    }
                }
            } else {
                HashMap::new()
            };

            if config.frontend.betterttv_emotes {
                match get_betterttv_emotes(&id).await {
                    Ok(bttv_emotes) => {
                        for (name, data) in bttv_emotes {
                            emotes.insert(name, data);
                        }
                    }
                    Err(err) => warn!("Unable to get list of BetterTTV emotes: {err:?}"),
                }
            }

            if config.frontend.seventv_emotes {
                match get_7tv_emotes(&id).await {
                    Ok(seventv_emotes) => {
                        for (name, data) in seventv_emotes {
                            emotes.insert(name, data);
                        }
                    }
                    Err(err) => warn!("Unable to get list of 7tv emotes: {err:?}"),
                }
            }

            download_emotes(emotes.values()).await;

            Ok(emotes
                .iter()
                .map(|(key, (value, _))| (key.clone(), value.clone()))
                .collect())
        }
        Err(err) => Err(err),
    }
}
