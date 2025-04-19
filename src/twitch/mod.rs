pub mod api;
mod badges;
pub mod channels;
pub mod client;
pub mod context;
mod models;
pub mod oauth;
mod roomstate;

use api::chat_settings::get_chat_settings;
use badges::retrieve_user_badges;
use color_eyre::Result;
use context::TwitchWebsocketContext;
use futures::StreamExt;
use log::{debug, error, info};
use roomstate::handle_roomstate;
use tokio::sync::{broadcast::Receiver, mpsc::Sender};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::{
    emotes::get_twitch_emote,
    handlers::{
        config::{CoreConfig, TwitchConfig},
        data::{DataBuilder, TwitchToTerminalAction},
        state::State,
    },
    twitch::{
        api::{
            channels::get_channel_id,
            event_sub::{CHANNEL_CHAT_MESSAGE_EVENT_SUB, subscribe_to_events},
            messages::{NewTwitchMessage, send_twitch_message},
        },
        models::ReceivedTwitchMessage,
        oauth::{get_twitch_client, get_twitch_client_oauth},
    },
    utils::text::{clean_message, parse_message_action},
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

    let emotes_enabled = config.frontend.is_emotes_enabled();

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
                        let chat_settings = get_chat_settings(context.twitch_client(), context.channel_id()).await.unwrap();
                        handle_roomstate(&chat_settings, &tx).await;

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
                            &config,
                            &mut context,
                            &tx,
                            emotes_enabled,
                            &received_message,
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

/// Handling either the terminal joining a new channel, or the application just starting up
async fn handle_channel_join() {
    todo!()
}

async fn handle_welcome_message(
    twitch_config: &TwitchConfig,
    context: &mut TwitchWebsocketContext,
    received_message: &ReceivedTwitchMessage,
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

async fn handle_user_message(
    config: &CoreConfig,
    context: &mut TwitchWebsocketContext,
    tx: &Sender<TwitchToTerminalAction>,
    emotes_enabled: bool,
    received_message: &ReceivedTwitchMessage,
) -> Result<()> {
    let Some(event) = received_message.event() else {
        return Ok(());
    };

    let message_text = event.message_text();
    let (msg, highlight) = parse_message_action(message_text);
    let received_emotes = emotes_enabled
        .then(|| event.emote_fragments())
        .unwrap_or_default();

    let emotes = futures::stream::iter(received_emotes.into_iter().map(
        |fragment_emote| async move {
            // TODO: Remove unwraps
            let emote = fragment_emote.emote().unwrap();
            let emote_id = emote.emote_id().unwrap().to_string();
            let emote_name = fragment_emote.emote_name().unwrap().to_string();
            get_twitch_emote(&emote_id).await?;
            Ok((emote_name, (emote_id, false)))
        },
    ))
    .buffer_unordered(10)
    .collect::<Vec<Result<(String, (String, bool))>>>();

    let mut chatter_user_name = event.chatter_user_name().to_string();
    let badges = event.badges();
    if config.frontend.badges {
        retrieve_user_badges(&mut chatter_user_name, badges);
    }

    let chatter_user_id = event.chatter_user_id();
    let cleaned_message = clean_message(message_text);
    let message_id = event.message_id();

    let message_emotes = emotes.await.into_iter().flatten().collect();

    tx.send(DataBuilder::user(
        chatter_user_name.to_string(),
        Some(chatter_user_id.to_string()),
        cleaned_message,
        message_emotes,
        Some(message_id.to_string()),
        highlight,
    ))
    .await
    .unwrap();

    tx.send(event.build_user_data()).await?;

    Ok(())
}

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

async fn handle_incoming_message(
    config: &CoreConfig,
    context: &mut TwitchWebsocketContext,
    tx: &Sender<TwitchToTerminalAction>,
    emotes_enabled: bool,
    received_message: &ReceivedTwitchMessage,
) -> Result<()> {
    // Welcome messages only happen once, when a websocket connects to a server
    if received_message.message_type().is_some_and(|message_type| {
        message_type == "session_welcome" && context.session_id().is_none()
    }) {
        handle_welcome_message(&config.twitch, context, received_message).await?;
    } else {
        handle_user_message(config, context, tx, emotes_enabled, received_message).await?;
    }

    Ok(())
}
