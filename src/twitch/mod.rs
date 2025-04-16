mod badges;
pub mod channels;
mod connection;
mod messages;
pub mod oauth;

use std::{collections::HashMap, hash::BuildHasher};

use color_eyre::Result;
use futures::StreamExt;
use log::{debug, info};
use reqwest::Client;
use tokio::sync::{broadcast::Receiver, mpsc::Sender};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::{
    emotes::{DownloadedEmotes, get_twitch_emote},
    handlers::{
        config::CoreConfig,
        data::{DataBuilder, TwitchToTerminalAction},
        state::State,
    },
    twitch::{
        badges::retrieve_user_badges,
        connection::{client_stream_reconnect, wait_client_stream},
    },
    utils::{
        emotes::emotes_enabled,
        text::{clean_message, parse_message_action},
    },
};

#[derive(Debug, Clone)]
pub enum TwitchAction {
    Privmsg(String),
    Join(String),
    ClearMessages,
}

pub async fn twitch_websocket(
    mut config: CoreConfig,
    tx: Sender<TwitchToTerminalAction>,
    mut rx: Receiver<TwitchAction>,
) {
    let url = config.twitch.config_twitch_websocket_url();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    info!("Twitch websocket handshake successful");

    let (write, mut read) = ws_stream.split();

    let oauth_token = config.twitch.token.clone();

    let mut twitch_client: Option<Client> = None;
    let mut session_id: Option<String> = None;

    // If the dashboard is the start state, wait until the user has selected
    // a channel before connecting to Twitch's IRC.
    // if config.terminal.first_state == State::Dashboard {
    //     debug!("Waiting for user to select channel from debug screen");

    //     loop {
    //         if let Ok(TwitchAction::Join(channel)) = rx.recv().await {
    //             config.twitch.channel = channel;

    //             debug!("User has selected channel from start screen");
    //             break;
    //         }
    //     }
    // }

    let enable_emotes = emotes_enabled(&config.frontend);

    let data_builder = DataBuilder::new(&config.frontend.datetime_format);
    let mut room_state_startup = false;

    let (mut client, mut stream) =
        wait_client_stream(tx.clone(), data_builder, config.clone()).await;

    let sender = client.sender();

    // Request commands capabilities
    // if client
    //     .send_cap_req(&[
    //         Capability::Custom("twitch.tv/commands"),
    //         Capability::Custom("twitch.tv/tags"),
    //     ])
    //     .is_err()
    // {
    //     tx.send(
    //         data_builder.system(
    //             "Unable to request commands/tags capability, certain features may be affected."
    //                 .to_string(),
    //         ),
    //     )
    //     .await
    //     .unwrap();
    // }

    let mut connected = true;

    loop {
        tokio::select! {
            biased;

            Ok(action) = rx.recv() => {
                let current_channel = format!("#{}", config.twitch.channel);

                match action {
                    // TwitchAction::Privmsg(message) => {
                    //     debug!("Sending message to Twitch: {message}");

                    //     client
                    //         .send_privmsg(current_channel, message)
                    //         .unwrap();
                    // }
                    TwitchAction::Join(channel) => {
                        debug!("Switching to channel {channel}");

                        let channel_list = format!("#{channel}");

                        // Leave previous channel
                        if let Err(err) = sender.send_part(current_channel) {
                            tx.send(data_builder.twitch(err.to_string())).await.unwrap();
                        }

                        // Join specified channel
                        if let Err(err) = sender.send_join(&channel_list) {
                            tx.send(data_builder.twitch(err.to_string())).await.unwrap();
                        }

                        // Set old channel to new channel
                        config.twitch.channel = channel;
                    }
                    // TwitchAction::ClearMessages => {
                    //     client.send(Command::Raw("CLEARCHAT".to_string(), vec![])).unwrap();
                    // }
                    _ => {
                        panic!("Unsupported Twitch action triggered");
                    }
                }
            }
            Some(message) = stream.next() => {
                match message {
                    Ok(message) => {
                        if !connected {
                            tx.send(data_builder.system("Reconnect succcessful.".to_string())).await.unwrap();
                            connected = true;
                        }

                        if let Some(b) = handle_message_command(message, tx.clone(), data_builder, config.frontend.badges, room_state_startup, enable_emotes).await {
                            room_state_startup = b;
                        }
                    }
                    Err(err) => {
                        connected = false;

                        debug!("Twitch connection error encountered: {err}, attempting to reconnect.");

                        (client, stream) = client_stream_reconnect(err, tx.clone(), data_builder, &config).await;

                    }
                }
            }
            else => {}
        };
    }
}

async fn handle_message_command(
    message: Message,
    tx: Sender<TwitchToTerminalAction>,
    data_builder: DataBuilder<'_>,
    badges: bool,
    room_state_startup: bool,
    enable_emotes: bool,
) -> Option<bool> {
    // let mut tags: HashMap<&str, &str> = HashMap::new();

    // if let Some(ref ref_tags) = message.tags {
    //     for tag in ref_tags {
    //         if let Some(ref tag_value) = tag.1 {
    //             tags.insert(&tag.0, tag_value);
    //         }
    //     }
    // }

    // match message.command {
    //     Command::PRIVMSG(ref _target, ref msg) => {
    //         let (msg, highlight) = parse_message_action(msg);

    //         let emotes = enable_emotes
    //             .then(|| tags.get("emotes").map(|&e| retrieve_twitch_emotes(msg, e)))
    //             .unwrap_or_default()
    //             .unwrap_or_default();

    //         let emotes =
    //             futures::stream::iter(emotes.into_iter().map(|(name, filename)| async move {
    //                 get_twitch_emote(&filename).await?;
    //                 Ok((name, (filename, false)))
    //             }))
    //             .buffer_unordered(10)
    //             .collect::<Vec<Result<(String, (String, bool))>>>();

    //         let mut name = message.source_nickname().unwrap().to_string();

    //         retrieve_user_badges(&mut name, &message, badges);

    //         let cleaned_message = clean_message(msg);

    //         let message_id = tags.get("id").map(|&s| s.to_string());
    //         let user_id = tags.get("user-id").map(|&s| s.to_string());

    //         debug!("Message received from twitch: {name} - {cleaned_message:?}");

    //         let emotes = emotes.await.into_iter().flatten().collect();

    //         tx.send(DataBuilder::user(
    //             name,
    //             user_id,
    //             cleaned_message,
    //             emotes,
    //             message_id,
    //             highlight,
    //         ))
    //         .await
    //         .unwrap();
    //     }
    //     Command::NOTICE(ref _target, ref msg) => {
    //         tx.send(data_builder.twitch(msg.to_string())).await.unwrap();
    //     }
    //     Command::JOIN(ref channel, _, _) => {
    //         tx.send(data_builder.twitch(format!("Joined {}", *channel)))
    //             .await
    //             .unwrap();
    //     }
    //     Command::Raw(ref cmd, ref _items) => {
    //         match cmd.as_ref() {
    //             "ROOMSTATE" => {
    //                 if !room_state_startup {
    //                     handle_roomstate(&tx, &tags).await;
    //                 }

    //                 return Some(true);
    //             }
    //             "USERNOTICE" => {
    //                 if let Some(value) = tags.get("system-msg") {
    //                     tx.send(data_builder.twitch((*value).to_string()))
    //                         .await
    //                         .unwrap();
    //                 }
    //             }
    //             "CLEARCHAT" => {
    //                 let user_id = tags.get("target-user-id").map(|&s| s.to_string());

    //                 tx.send(TwitchToTerminalAction::ClearChat(user_id.clone()))
    //                     .await
    //                     .unwrap();

    //                 if user_id.is_some() {
    //                     let ban_duration = tags.get("ban-duration").map(|&s| s.to_string());

    //                     if let Some(duration) = ban_duration {
    //                         tx.send(
    //                             data_builder
    //                                 .twitch(format!("User was timed out for {duration} seconds")),
    //                         )
    //                         .await
    //                         .unwrap();
    //                     }
    //                     else {
    //                         tx.send(data_builder.twitch("User banned".to_string()))
    //                             .await
    //                             .unwrap();
    //                     }
    //                 } else {
    //                     tx.send(data_builder.twitch("Chat cleared by a moderator.".to_string()))
    //                         .await
    //                         .unwrap();
    //                 }
    //             }
    //             "CLEARMSG" => {
    //                 if let Some(id) = tags.get("target-msg-id") {
    //                     tx.send(TwitchToTerminalAction::DeleteMessage((*id).to_string()))
    //                         .await
    //                         .unwrap();
    //                 }
    //             }
    //             _ => (),
    //         }
    //     }
    //     _ => (),
    // }

    None
}

// pub async fn handle_roomstate<S: BuildHasher>(
//     tx: &Sender<TwitchToTerminalAction>,
//     tags: &HashMap<&str, &str, S>,
// ) {
//     let mut room_state = String::new();

//     for (name, value) in tags {
//         match *name {
//             "emote-only" if *value == "1" => {
//                 room_state.push_str("The channel is emote-only.\n");
//             }
//             "followers-only" if *value != "-1" => {
//                 room_state.push_str("The channel is followers-only.\n");
//             }
//             "subs-only" if *value == "1" => {
//                 room_state.push_str("The channel is subscribers-only.\n");
//             }
//             "slow" if *value != "0" => {
//                 room_state.push_str("The channel has a ");
//                 room_state.push_str(value);
//                 room_state.push_str("s slowmode.\n");
//             }
//             _ => (),
//         }
//     }

//     // Trim last newline
//     room_state.pop();

//     if room_state.is_empty() {
//         return;
//     }

//     let message_id = tags.get("target-msg-id").map(|&s| s.to_string());

//     tx.send(DataBuilder::user(
//         String::from("Info"),
//         None,
//         room_state,
//         DownloadedEmotes::default(),
//         message_id,
//         false,
//     ))
//     .await
//     .unwrap();
// }
