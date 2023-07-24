use anyhow::{Context, Result};
use futures::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use std::{borrow::BorrowMut, collections::HashMap, path::Path};
use tokio::io::AsyncWriteExt;

use crate::{
    emotes::DownloadedEmotes,
    handlers::config::{CompleteConfig, FrontendConfig},
    twitch::oauth::get_twitch_client,
    utils::pathing::cache_path,
};

// HashMap of emote name, emote filename, emote url, and if the emote is an overlay
type EmoteMap = HashMap<String, (String, String, bool)>;

mod twitch {
    use crate::emotes::downloader::EmoteMap;
    use anyhow::Result;
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Image {
        url_1x: String,
    }

    #[derive(Deserialize)]
    struct Emote {
        id: String,
        name: String,
        images: Image,
    }

    #[derive(Deserialize)]
    struct EmoteList {
        data: Vec<Emote>,
    }

    pub async fn get_emotes(client: &Client, channel_id: i32) -> Result<EmoteMap> {
        let channel_emotes = client
            .get(format!(
                "https://api.twitch.tv/helix/chat/emotes?broadcaster_id={channel_id}",
            ))
            .send()
            .await?
            .error_for_status()?
            .json::<EmoteList>()
            .await?
            .data;

        let global_emotes = client
            .get("https://api.twitch.tv/helix/chat/emotes/global")
            .send()
            .await?
            .error_for_status()?
            .json::<EmoteList>()
            .await?
            .data;

        Ok(channel_emotes
            .into_iter()
            .chain(global_emotes)
            .map(|emote| (emote.name, (emote.id, emote.images.url_1x, false)))
            .collect())
    }
}

mod betterttv {
    use crate::emotes::downloader::EmoteMap;
    use anyhow::Result;
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Emote {
        id: String,
        code: String,
        image_type: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct EmoteList {
        channel_emotes: Vec<Emote>,
        shared_emotes: Vec<Emote>,
    }

    pub async fn get_emotes(channel_id: i32) -> Result<EmoteMap> {
        let client = Client::new();

        let EmoteList {
            channel_emotes,
            shared_emotes,
        } = client
            .get(format!(
                "https://api.betterttv.net/3/cached/users/twitch/{channel_id}",
            ))
            .send()
            .await?
            .error_for_status()?
            .json::<EmoteList>()
            .await?;

        let global_emotes = client
            .get("https://api.betterttv.net/3/cached/emotes/global")
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Emote>>()
            .await?;

        Ok(channel_emotes
            .into_iter()
            .chain(shared_emotes)
            .chain(global_emotes)
            .map(
                |Emote {
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
}

mod seventv {
    use crate::emotes::downloader::EmoteMap;
    use anyhow::Result;
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Emote {
        name: String,
        id: String,
        flags: u64,
    }

    #[derive(Deserialize)]
    struct EmoteList {
        emotes: Vec<Emote>,
    }

    #[derive(Deserialize)]
    struct EmoteSet {
        emote_set: EmoteList,
    }

    pub async fn get_emotes(channel_id: i32) -> Result<EmoteMap> {
        let client = Client::new();

        let channel_emotes = client
            .get(format!("https://7tv.io/v3/users/twitch/{channel_id}",))
            .send()
            .await?
            .error_for_status()?
            .json::<EmoteSet>()
            .await?
            .emote_set
            .emotes;

        let global_emotes = client
            .get("https://7tv.io/v3/emote-sets/global")
            .send()
            .await?
            .error_for_status()?
            .json::<EmoteList>()
            .await?
            .emotes;

        Ok(channel_emotes
            .into_iter()
            .chain(global_emotes)
            .map(|Emote { name, id, flags }| {
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
}

mod frankerfacez {
    use crate::emotes::downloader::EmoteMap;
    use anyhow::Result;
    use futures::StreamExt;
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Emote {
        name: String,
        id: u64,
        modifier_flags: u64,
    }

    #[derive(Deserialize)]
    struct EmoteList {
        emoticons: Vec<Emote>,
    }

    #[derive(Deserialize)]
    struct EmoteSet {
        set: EmoteList,
    }

    #[derive(Deserialize)]
    struct GlobalSets {
        default_sets: Vec<u64>,
    }

    #[derive(Deserialize)]
    struct SetId {
        set: u64,
    }

    #[derive(Deserialize)]
    struct Room {
        room: SetId,
    }

    pub async fn get_emotes(channel_id: i32) -> Result<EmoteMap> {
        let client = &Client::new();

        let mut sets = client
            .get("https://api.frankerfacez.com/v1/_set/global")
            .send()
            .await?
            .error_for_status()?
            .json::<GlobalSets>()
            .await?
            .default_sets;

        sets.push(
            client
                .get(format!(
                    "https://api.frankerfacez.com/v1/_room/id/{channel_id}",
                ))
                .send()
                .await?
                .error_for_status()?
                .json::<Room>()
                .await?
                .room
                .set,
        );

        let emotes = futures::stream::iter(sets.into_iter().map(|set| async move {
            Ok(client
                .get(format!("https://api.frankerfacez.com/v1/_set/{set}",))
                .send()
                .await?
                .error_for_status()?
                .json::<EmoteSet>()
                .await?
                .set
                .emoticons)
        }))
        .buffer_unordered(10)
        .collect::<Vec<Result<_>>>()
        .await
        .into_iter()
        .flatten()
        .flatten();

        Ok(emotes
            .map(
                |Emote {
                     name,
                     id,
                     modifier_flags,
                 }| {
                    (
                        name,
                        (
                            format!("ffz_{id}"),
                            format!("https://cdn.frankerfacez.com/emote/{id}/1"),
                            modifier_flags != 0,
                        ),
                    )
                },
            )
            .collect())
    }
}

#[derive(Deserialize)]
struct Channel {
    id: String,
}

#[derive(Deserialize)]
struct ChannelList {
    data: Vec<Channel>,
}

async fn get_channel_id(client: &Client, channel: &str) -> Result<i32> {
    Ok(client
        .get(format!("https://api.twitch.tv/helix/users?login={channel}",))
        .send()
        .await?
        .error_for_status()?
        .json::<ChannelList>()
        .await?
        .data
        .first()
        .context("Could not get channel id.")?
        .id
        .parse()?)
}

async fn download_emotes(emotes: EmoteMap) -> DownloadedEmotes {
    let client = &Client::new();

    // We need to limit the number of concurrent connections, otherwise we might hit some system limits
    // ex: number of files/sockets open, etc.
    futures::stream::iter(
        emotes
            .into_iter()
            .map(|(x, (filename, url, o))| async move {
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
            }),
    )
    .buffer_unordered(100)
    .collect::<Vec<Result<(String, (String, bool))>>>()
    .await
    .into_iter()
    .flatten()
    .collect()
}

enum EmoteProvider {
    Twitch,
    BetterTTV,
    SevenTV,
    FrankerFaceZ,
}

fn get_enabled_emote_providers(config: &FrontendConfig) -> Vec<EmoteProvider> {
    let mut providers = Vec::with_capacity(4);

    if config.twitch_emotes {
        providers.push(EmoteProvider::Twitch);
    }
    if config.betterttv_emotes {
        providers.push(EmoteProvider::BetterTTV);
    }
    if config.seventv_emotes {
        providers.push(EmoteProvider::SevenTV);
    }
    if config.frankerfacez_emotes {
        providers.push(EmoteProvider::FrankerFaceZ);
    }

    providers
}

pub async fn get_emotes(config: &CompleteConfig, channel: &str) -> Result<DownloadedEmotes> {
    // Reuse the same client and headers for twitch requests
    let twitch_client = get_twitch_client(config).await?;

    let channel_id = get_channel_id(&twitch_client, channel).await?;

    let enabled_emotes = get_enabled_emote_providers(&config.frontend);

    let twitch_get_emotes = |c: i32| twitch::get_emotes(&twitch_client, c);

    // Concurrently get the list of emotes for each provider
    let emotes =
        futures::stream::iter(enabled_emotes.into_iter().map(|emote_provider| async move {
            match emote_provider {
                EmoteProvider::Twitch => twitch_get_emotes(channel_id).await,
                EmoteProvider::BetterTTV => betterttv::get_emotes(channel_id).await,
                EmoteProvider::SevenTV => seventv::get_emotes(channel_id).await,
                EmoteProvider::FrankerFaceZ => frankerfacez::get_emotes(channel_id).await,
            }
        }))
        .buffer_unordered(4)
        .collect::<Vec<Result<EmoteMap>>>()
        .await
        .into_iter()
        .flatten()
        .flatten()
        .collect::<EmoteMap>();

    Ok(download_emotes(emotes).await)
}
