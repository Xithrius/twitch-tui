mod badges;
pub mod channels;
mod connection;
mod models;
pub mod oauth;

use std::{collections::HashMap, hash::BuildHasher};

use color_eyre::Result;
use connection::subscribe_to_channel;
use futures::StreamExt;
use log::{debug, error, info};
use models::ReceivedTwitchMessage;
use oauth::{get_channel_id, get_twitch_client, get_twitch_client_id, send_twitch_message};
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
    utils::{
        emotes::emotes_enabled,
        text::{clean_message, parse_message_action},
    },
};

#[derive(Debug, Clone)]
pub enum TwitchAction {
    SendMessage(String),
    JoinChannel(String),
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

    let (_, mut stream) = ws_stream.split();

    let oauth_token = config.twitch.token.clone();

    let mut twitch_client: Option<Client> = None;
    let mut session_id: Option<String> = None;

    // If the dashboard is the start state, wait until the user has selected
    // a channel before connecting to Twitch's websocket server.
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

    let oauth_token = config.twitch.token.clone();

    let mut twitch_client: Option<Client> = None;
    let mut session_id: Option<String> = None;

    loop {
        tokio::select! {
            biased;

            Ok(action) = rx.recv() => {
                match action {
                    TwitchAction::SendMessage(message) => {
                        if let Some(twitch_client) = twitch_client.as_ref() {
                            let _ = send_twitch_message(twitch_client, &message).await;
                        }
                    },
                    TwitchAction::JoinChannel(_) => todo!(),
                    TwitchAction::ClearMessages => todo!(),
                }
            }
            Some(message) = stream.next() => {
                match message {
                    Ok(message) => {
                        let message_text = match message {
                            Message::Text(message_text) => message_text,
                            Message::Ping(_) => {
                                // println!("Ping");
                                continue;
                            }
                            Message::Pong(_) => {
                                // println!("Pong");
                                continue;
                            }
                            Message::Close(close_frame) => {
                                // println!("Close frame: {close_frame:?}");
                                continue;
                            }
                            _ => continue,
                        };

                        let received_message = match serde_json::from_str::<ReceivedTwitchMessage>(&message_text) {
                            Ok(received_message) => received_message,
                            Err(err) => {
                                panic!("Could not deserialize received message into JSON: {err} -- {message_text}");
                            }
                        };

                        if received_message
                            .message_type()
                            .is_some_and(|message_type| message_type == "session_welcome" && session_id.is_none())
                        {
                            let client_id = get_twitch_client_id(oauth_token.as_deref()).await.unwrap();

                            let new_twitch_client = get_twitch_client(client_id, oauth_token.as_deref())
                                .await
                                .expect("failed to authenticate twitch client");
                            twitch_client = Some(new_twitch_client.clone());

                            let new_session_id = received_message.session_id();
                            session_id.clone_from(&new_session_id);

                            let channel_id = get_channel_id(&new_twitch_client, &config.twitch.channel)
                                .await
                                .unwrap();

                            let channel_subscription_response = subscribe_to_channel(
                                &new_twitch_client,
                                client_id,
                                new_session_id,
                                channel_id.to_string(),
                            )
                            .await
                            .unwrap();

                            continue;
                        }

                        if let Some(event) = received_message.event() {
                            tx.send(event.build_user_data()).await.unwrap();
                        }

                        // handle_incoming_message(message, tx.clone(), data_builder, config.frontend.badges, enable_emotes).await;
                    }
                    Err(err) => {
                        error!("Twitch connection error encountered: {err}, attempting to reconnect.");
                    }
                }
            }
            else => {}
        };
    }
}

async fn handle_incoming_message(
    message: Message,
    tx: Sender<TwitchToTerminalAction>,
    data_builder: DataBuilder<'_>,
    badges: bool,
    enable_emotes: bool,
) {

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
