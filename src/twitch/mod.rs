pub mod api;
mod badges;
pub mod channels;
pub mod client;
pub mod context;
mod models;
pub mod oauth;

use std::{fmt::Write as _, hash::BuildHasher};

use color_eyre::{Result, eyre::bail};
use context::TwitchWebsocketContext;
use futures::StreamExt;
use log::{debug, error, info};
use tokio::sync::{broadcast::Receiver, mpsc::Sender};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::{
    handlers::{
        config::{CoreConfig, TwitchConfig},
        data::{DataBuilder, TwitchToTerminalAction},
        state::State,
    },
    twitch::{
        api::{
            channels::get_channel_id,
            chat_settings::TwitchChatSettingsResponse,
            event_sub::{CHANNEL_CHAT_MESSAGE_EVENT_SUB, subscribe_to_events},
            messages::{NewTwitchMessage, send_twitch_message},
        },
        models::ReceivedTwitchMessage,
        oauth::{get_twitch_client, get_twitch_client_oauth},
    },
    utils::emotes::is_emotes_enabled,
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

    // If the dashboard is the start state, wait until the user has selected
    // a channel before connecting to Twitch's websocket server.
    if config.terminal.first_state == State::Dashboard {
        debug!("Waiting for user to select channel from debug screen");

        loop {
            if let Ok(TwitchAction::JoinChannel(channel)) = rx.recv().await {
                config.twitch.channel = channel;

                debug!("User has selected channel from start screen");
                break;
            }
        }
    }

    let data_builder = DataBuilder::new(&config.frontend.datetime_format);

    let emotes_enabled = is_emotes_enabled(&config.frontend);

    let mut context = TwitchWebsocketContext::default();
    context.set_emotes(emotes_enabled);
    context.set_token(config.twitch.token.clone());

    loop {
        tokio::select! {
            biased;

            Ok(action) = rx.recv() => {
                match action {
                    TwitchAction::SendMessage(message) => {
                        let Some(twitch_client) = context.twitch_client() else {
                            panic!("No twitch client at this stage");
                        };

                        let Some(channel_id) = context.channel_id() else {
                            panic!("No channel ID at this stage");
                        };

                        let Some(twitch_oauth) = context.oauth() else {
                            panic!("No user ID at this stage");
                        };

                        let new_message = NewTwitchMessage::new(channel_id.to_string(), twitch_oauth.user_id.to_string(), message);
                        let twitch_message_response = send_twitch_message(twitch_client, new_message).await;
                    },
                    TwitchAction::JoinChannel(_) => {
                        panic!("Joining channels is not implemented at this moment");
                    },
                    TwitchAction::ClearMessages => {
                        panic!("Clearning messages is not implemented at this moment");
                    },
                }
            }
            Some(message) = stream.next() => {
                match message {
                    Ok(message) => {
                        let Message::Text(message_text) = message else {
                            continue;
                        };

                        let received_message = match serde_json::from_str::<ReceivedTwitchMessage>(&message_text) {
                            Ok(received_message) => received_message,
                            Err(err) => {
                                panic!("Could not deserialize received message into JSON: {err} -- {message_text}");
                            }
                        };

                        let _ = handle_incoming_message(
                            &mut context,
                            &tx.clone(),
                            &received_message,
                            &config.twitch
                        ).await;
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

async fn handle_welcome_message(
    context: &mut TwitchWebsocketContext,
    received_message: &ReceivedTwitchMessage,
    twitch_config: &TwitchConfig,
) -> Result<()> {
    let oauth_token = context.clone().token();

    let twitch_oauth = get_twitch_client_oauth(oauth_token.as_ref()).await?;
    context.set_oauth(Some(twitch_oauth.clone()));

    let twitch_client = get_twitch_client(&twitch_oauth, oauth_token.as_ref())
        .await
        .expect("failed to authenticate twitch client");
    context.set_twitch_client(Some(twitch_client.clone()));

    let session_id = received_message.session_id();
    context.set_session_id(session_id.clone());

    let channel_id = get_channel_id(&twitch_client, &twitch_config.channel).await?;
    context.set_channel_id(Some(channel_id.clone()));

    // TODO: Do something with the subscriptions response data
    let _initial_subscriptions_response = subscribe_to_events(
        &twitch_client,
        &twitch_oauth,
        session_id,
        channel_id.to_string(),
        vec![CHANNEL_CHAT_MESSAGE_EVENT_SUB.to_string()],
    )
    .await?;

    Ok(())
}

async fn handle_incoming_message(
    context: &mut TwitchWebsocketContext,
    tx: &Sender<TwitchToTerminalAction>,
    received_message: &ReceivedTwitchMessage,
    twitch_config: &TwitchConfig,
) -> Result<()> {
    // handle_incoming_message(message, tx.clone(), data_builder, config.frontend.badges, enable_emotes).await;

    if received_message.message_type().is_some_and(|message_type| {
        message_type == "session_welcome" && context.session_id().is_none()
    }) {
        handle_welcome_message(context, received_message, twitch_config).await?;
    }

    let Some(event) = received_message.event() else {
        return Ok(());
    };

    tx.send(event.build_user_data()).await?;

    Ok(())
}

// fn handle_incoming_message(
//     message: Message,
//     tx: Sender<TwitchToTerminalAction>,
//     data_builder: DataBuilder<'_>,
//     badges: bool,
//     enable_emotes: bool,
// ) {
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
// }

async fn handle_roomstate(
    chat_settings: &TwitchChatSettingsResponse,
    tx: &Sender<TwitchToTerminalAction>,
) {
    let mut room_state = String::new();

    if let Some(slow_mode_wait_time) = chat_settings.slow_mode() {
        let _ = &writeln!(
            room_state,
            "The channel has a {slow_mode_wait_time} second slowmode."
        );
    }

    if let Some(follower_mode_duration) = chat_settings.follower_mode() {
        let _ = &writeln!(
            room_state,
            "The channel is followers-only. You must follow the channel for at least {follower_mode_duration} second(s) to chat."
        );
    }

    if let Some(non_moderator_chat_delay_duration) = chat_settings.non_moderator_chat() {
        let _ = &writeln!(
            room_state,
            "The channel has a non-moderator message delay. It will take {non_moderator_chat_delay_duration} second(s) for your message to show after sending."
        );
    }

    if chat_settings.subscriber_mode() {
        let _ = writeln!(room_state, "The channel is subscribers-only.");
    }

    if chat_settings.emote_mode() {
        let _ = writeln!(room_state, "The channel is emote-only.");
    }

    if chat_settings.unique_chat_mode() {
        let _ = writeln!(room_state, "The channel accepts only unique messages.");
    }

    // Trim last newline
    room_state.pop();

    if room_state.is_empty() {
        return;
    }

    tx.send(DataBuilder::twitch(room_state)).await.unwrap();
}
