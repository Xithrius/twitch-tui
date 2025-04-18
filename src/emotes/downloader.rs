use std::{borrow::BorrowMut, collections::HashMap, path::Path};

use color_eyre::Result;
use futures::StreamExt;
use reqwest::{Client, Response};
use tokio::io::AsyncWriteExt;

use crate::{
    emotes::DownloadedEmotes,
    handlers::config::{CoreConfig, FrontendConfig},
    twitch::{
        api::channels::get_channel_id,
        oauth::{get_twitch_client, get_twitch_client_oauth},
    },
    utils::pathing::cache_path,
};

// HashMap of emote name, emote filename, emote url, and if the emote is an overlay
type EmoteMap = HashMap<String, (String, String, bool)>;

mod twitch {
    use color_eyre::Result;
    use log::warn;
    use reqwest::Client;
    use serde::Deserialize;

    use crate::{emotes::downloader::EmoteMap, twitch::api::TWITCH_API_BASE_URL};

    #[derive(Deserialize, Debug)]
    struct Emote {
        id: String,
        name: String,
        format: Vec<String>,
        scale: Vec<String>,
        theme_mode: Vec<String>,
    }

    #[derive(Deserialize, Debug)]
    struct Cursor {
        cursor: Option<String>,
    }

    #[derive(Deserialize, Debug)]
    struct EmoteList {
        data: Vec<Emote>,
        template: String,
        pagination: Cursor,
    }

    fn parse_emote_list(v: EmoteList) -> EmoteMap {
        let template = v.template;

        v.data
            .into_iter()
            .filter_map(|emote| {
                let url = template.replace("{{id}}", &emote.id);

                let url = url.replace(
                    "{{format}}",
                    if emote.format.contains(&String::from("animated")) {
                        "animated"
                    } else {
                        emote.format.first()?
                    },
                );

                let url = url.replace(
                    "{{theme_mode}}",
                    if emote.theme_mode.contains(&String::from("dark")) {
                        "dark"
                    } else {
                        emote.theme_mode.first()?
                    },
                );

                let url = url.replace(
                    "{{scale}}",
                    if emote.scale.contains(&String::from("1.0")) {
                        "1.0"
                    } else {
                        emote.scale.first()?
                    },
                );

                Some((emote.name, (emote.id, url, false)))
            })
            .collect()
    }

    // Twitch will not send all the emotes in one response, we use the cursor they return to query further emotes.
    pub async fn get_user_emotes(client: &Client, user_id: &str) -> Result<EmoteMap> {
        let mut user_emotes = client
            .get(format!(
                "{TWITCH_API_BASE_URL}/chat/emotes/user?user_id={user_id}"
            ))
            .send()
            .await?
            .error_for_status().inspect_err(|_| { warn!("Unable to get user emotes, please verify that the access token includes the user:read:emotes scope.");})?
            .json::<EmoteList>()
            .await?;

        while let Some(c) = user_emotes.pagination.cursor {
            let emotes =   client
            .get(format!(
                "{TWITCH_API_BASE_URL}/chat/emotes/user?user_id={user_id}&after={c}",
            ))
            .send()
            .await?
            .error_for_status().inspect_err(|_| { warn!("Unable to get user emotes, please verify that the access token includes the user:read:emotes scope.");})?
            .json::<EmoteList>().await?;

            user_emotes.pagination = emotes.pagination;
            user_emotes.data.extend(emotes.data);
        }

        Ok(parse_emote_list(user_emotes))
    }
}

mod betterttv {
    use color_eyre::Result;
    use reqwest::Client;
    use serde::Deserialize;

    use crate::emotes::downloader::EmoteMap;

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

    pub async fn get_emotes(channel_id: String) -> Result<EmoteMap> {
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
    use color_eyre::Result;
    use reqwest::Client;
    use serde::Deserialize;

    use crate::emotes::downloader::EmoteMap;

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

    pub async fn get_emotes(channel_id: String) -> Result<EmoteMap> {
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
    use color_eyre::Result;
    use futures::StreamExt;
    use reqwest::Client;
    use serde::Deserialize;

    use crate::emotes::downloader::EmoteMap;

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

    pub async fn get_emotes(channel_id: String) -> Result<EmoteMap> {
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

async fn save_emote(path: &Path, mut res: Response) -> Result<()> {
    let mut file = tokio::fs::File::create(&path).await?;

    while let Some(mut item) = res.chunk().await? {
        file.write_all_buf(item.borrow_mut()).await?;
    }

    Ok(())
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

                let res = client.get(&url).send().await?.error_for_status()?;

                save_emote(path, res).await?;

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

#[derive(Eq, PartialEq)]
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

pub async fn get_emotes(
    config: &CoreConfig,
    channel: &str,
) -> Result<(DownloadedEmotes, DownloadedEmotes)> {
    // Reuse the same client and headers for twitch requests
    let client_id = &get_twitch_client_oauth(None).await?;
    let twitch_client = get_twitch_client(client_id, config.twitch.token.as_ref()).await?;

    let channel_id = get_channel_id(&twitch_client, channel).await?;

    let enabled_emotes = get_enabled_emote_providers(&config.frontend);

    let user_emotes = if enabled_emotes.contains(&EmoteProvider::Twitch) {
        twitch::get_user_emotes(&twitch_client, &client_id.user_id)
            .await
            .unwrap_or_default()
    } else {
        HashMap::default()
    };

    // Concurrently get the list of emotes for each provider
    let global_emotes = futures::stream::iter(enabled_emotes.into_iter().map(|emote_provider| {
        let channel_id = channel_id.clone();

        async move {
            match emote_provider {
                EmoteProvider::Twitch => Ok(HashMap::new()),
                EmoteProvider::BetterTTV => betterttv::get_emotes(channel_id).await,
                EmoteProvider::SevenTV => seventv::get_emotes(channel_id).await,
                EmoteProvider::FrankerFaceZ => frankerfacez::get_emotes(channel_id).await,
            }
        }
    }))
    .buffer_unordered(4)
    .collect::<Vec<Result<EmoteMap>>>()
    .await
    .into_iter()
    .flatten()
    .flatten()
    .collect::<EmoteMap>();

    Ok((
        download_emotes(user_emotes).await,
        download_emotes(global_emotes).await,
    ))
}

pub async fn get_twitch_emote(name: &str) -> Result<()> {
    // Checks if emote is already downloaded.
    let path = cache_path(name);
    let path = Path::new(&path);

    if tokio::fs::metadata(&path).await.is_ok() {
        return Ok(());
    }

    // Download it if it is not in the cache, try the animated version first.
    let url = format!("https://static-cdn.jtvnw.net/emoticons/v2/{name}/animated/light/1.0");
    let client = Client::new();
    let res = client.get(&url).send().await?.error_for_status();

    let res = if res.is_err() {
        client
            .get(url.replace("animated", "static"))
            .send()
            .await?
            .error_for_status()
    } else {
        res
    }?;

    save_emote(path, res).await
}
