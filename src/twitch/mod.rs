pub mod api;
mod badges;
pub mod channels;
pub mod client;
pub mod context;
mod models;
pub mod oauth;
mod roomstate;

#[cfg(test)]
mod tests;

use api::{
    chat_settings::get_chat_settings,
    event_sub::{INITIAL_EVENT_SUBSCRIPTIONS, subscriptions, unsubscribe_from_events},
};
use badges::retrieve_user_badges;
use color_eyre::{
    Result,
    eyre::{Context, ContextCompat},
};
use context::TwitchWebsocketContext;
use futures::StreamExt;
use roomstate::handle_roomstate;
use tokio::sync::{broadcast::Receiver, mpsc::Sender};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Utf8Bytes, protocol::Message},
};
use tracing::{debug, error, info};

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
            event_sub::subscribe_to_events,
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
    let (ws_stream, _) = connect_async(url).await.unwrap_or_else(|_| {
        panic!(
            "Failed to connect to websocket server at {}",
            config.twitch.server
        )
    });

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

    let emotes_enabled = config.frontend.is_emotes_enabled();

    let mut context = TwitchWebsocketContext::default();
    context.set_emotes(emotes_enabled);
    context.set_token(config.twitch.token.clone());

    if stream.next().await.is_some() {
        debug!("Websocket server has pinged you to make sure you're here");
    }

    // Handle the welcome message, it should arrive after the initial ping
    let Some(Ok(Message::Text(message))) = stream.next().await else {
        panic!("First message was not a welcome message, something has gone terribly wrong");
    };
    if let Err(err) = handle_welcome_message(&mut config.twitch, &mut context, &tx, message).await {
        panic!("Failed to work with welcome message: {err}");
    }

    loop {
        tokio::select! {
            biased;

            Ok(action) = rx.recv() => {
                match action {
                    TwitchAction::SendMessage(message) => {
                        if let Err(err) = handle_send_message(&context, message).await {
                            error!("Failed to send Twitch message from terminal: {err}");
                        }
                    },
                    TwitchAction::JoinChannel(channel_name) => {
                        if let Err(err) = handle_channel_join(&mut config.twitch, &mut context, &tx, channel_name, false).await {
                            error!("Joining channel failed: {err}");
                        }
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
                                error!("Error when deserializing received message: {err}");
                                continue;
                            }
                        };

                        if let Err(err) = handle_incoming_message(
                            config.clone(),
                            &tx,
                            emotes_enabled,
                            received_message,
                        ).await {
                            error!("Error when handling incoming message: {err}");
                        }
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

/// Handle the user wanting to send a message from the terminal to the WebSocket server
async fn handle_send_message(context: &TwitchWebsocketContext, message: String) -> Result<()> {
    let twitch_client = context
        .twitch_client()
        .context("Twitch client could not be found when sending message")?;

    let channel_id = context
        .channel_id()
        .context("Channel ID could not be found when sending message")?;

    let twitch_oauth = context
        .oauth()
        .context("Twitch OAuth could not be found when sending message")?;

    let new_message = NewTwitchMessage::new(
        channel_id.to_string(),
        twitch_oauth.user_id.to_string(),
        message,
    );

    send_twitch_message(twitch_client, new_message).await?;

    Ok(())
}

/// Handling either the terminal joining a new channel, or the application just starting up
async fn handle_channel_join(
    twitch_config: &mut TwitchConfig,
    context: &mut TwitchWebsocketContext,
    tx: &Sender<TwitchToTerminalAction>,
    channel_name: String,
    first_channel: bool,
) -> Result<()> {
    let twitch_client = context.twitch_client().context("Twitch client not found")?;
    let twitch_oauth = context.oauth().context("No OAuth found")?;
    let chat_message_subscription = vec![subscriptions::CHANNEL_CHAT_MESSAGE];

    // Unsubscribe from previous channel
    if !first_channel {
        unsubscribe_from_events(
            twitch_client,
            context.event_subscriptions(),
            chat_message_subscription.clone(),
        )
        .await?;
    }

    // Subscribe to new channel
    let channel_id = if first_channel {
        context
            .channel_id()
            .context("Failed to get channel ID from context")?
    } else {
        &get_channel_id(twitch_client, &channel_name).await?
    };

    let new_subscriptions = subscribe_to_events(
        twitch_client,
        twitch_oauth,
        context.session_id().cloned(),
        channel_id.to_string(),
        chat_message_subscription,
    )
    .await
    .context(format!(
        "Failed to subscribe to new channel '{channel_name}'"
    ))?;

    // Set channel chat message event subscription to correct subscription ID
    let chat_event_subscription_id = new_subscriptions
        .get(subscriptions::CHANNEL_CHAT_MESSAGE)
        .context("Could not find chat message subscription ID in new subscriptions map")?;

    // TODO: Probably a better way to handle this
    let context_channel_id = channel_id.to_string();

    context.add_event_subscription(
        subscriptions::CHANNEL_CHAT_MESSAGE.to_owned(),
        chat_event_subscription_id.to_string(),
    );

    // Set old channel to new channel
    twitch_config.channel.clone_from(&channel_name);
    context.set_channel_id(Some(context_channel_id));

    // Notify frontend that new channel has been joined
    tx.send(DataBuilder::twitch(format!("Joined {channel_name}")))
        .await
        .context("Failed to send twitch join message")?;

    // Handle new chat settings with roomstate
    let chat_settings = get_chat_settings(context.twitch_client(), context.channel_id()).await?;
    handle_roomstate(&chat_settings, tx).await?;

    Ok(())
}

async fn handle_welcome_message(
    twitch_config: &mut TwitchConfig,
    context: &mut TwitchWebsocketContext,
    tx: &Sender<TwitchToTerminalAction>,
    message: Utf8Bytes,
) -> Result<()> {
    let received_message = serde_json::from_str::<ReceivedTwitchMessage>(&message)
        .context("Could not convert welcome message to received message")?;

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

    let initial_event_subscriptions = subscribe_to_events(
        &twitch_client,
        &twitch_oauth,
        session_id,
        channel_id.to_string(),
        INITIAL_EVENT_SUBSCRIPTIONS.to_vec(),
    )
    .await
    .context("Failed to subscribe to initial events")?;

    context.set_event_subscriptions(initial_event_subscriptions);

    handle_channel_join(
        twitch_config,
        context,
        tx,
        twitch_config.channel.clone(),
        true,
    )
    .await
    .context("Failed to join first channel")?;

    Ok(())
}

async fn handle_incoming_message(
    config: CoreConfig,
    tx: &Sender<TwitchToTerminalAction>,
    emotes_enabled: bool,
    received_message: ReceivedTwitchMessage,
) -> Result<()> {
    let Some(event) = received_message.event() else {
        return Ok(());
    };

    // TODO: Make this into a match statement to handle more cases
    if received_message
        .subscription_type()
        .is_some_and(|subscription_type| subscription_type == "channel.chat.notification")
    {
        if let Some(twitch_notification_message) = event.system_message() {
            tx.send(DataBuilder::twitch(twitch_notification_message.to_string()))
                .await?;
        }

        return Ok(());
    }

    let message_text = event
        .message_text()
        .context("Could not find message text")?;
    let (msg, highlight) = parse_message_action(&message_text);
    let received_emotes = emotes_enabled
        .then(|| event.emote_fragments())
        .unwrap_or_default()
        .unwrap_or_default();

    let emotes = futures::stream::iter(received_emotes.into_iter().map(
        |fragment_emote: models::ReceivedTwitchEventMessageFragment| async move {
            let emote = fragment_emote
                .emote()
                .context("Failed to get emote from emote fragment")?;
            let emote_id = emote
                .emote_id()
                .context("Failed to get emote ID from emote fragment")?
                .to_string();
            let emote_name = fragment_emote
                .emote_name()
                .context("Failed to get emote name from emote fragment")?
                .to_string();

            get_twitch_emote(&emote_id).await?;

            Ok((emote_name, (emote_id, false)))
        },
    ))
    .buffer_unordered(10)
    .collect::<Vec<Result<(String, (String, bool))>>>();

    let mut chatter_user_name = event
        .chatter_user_name()
        .context("Could not find chatter user name")?
        .to_string();
    let badges = event.badges().unwrap_or_default();
    if config.frontend.badges {
        retrieve_user_badges(&mut chatter_user_name, &badges);
    }

    let chatter_user_id = event
        .chatter_user_id()
        .context("could not find chatter user ID")?;
    let cleaned_message = clean_message(msg);
    let message_id = event
        .message_id()
        .context("Could not find message ID")?
        .to_string();

    let message_emotes = emotes.await.into_iter().flatten().collect();

    tx.send(DataBuilder::user(
        chatter_user_name.to_string(),
        Some(chatter_user_id.to_string()),
        cleaned_message,
        message_emotes,
        Some(message_id),
        highlight,
    ))
    .await?;

    Ok(())
}

//     Command::Raw(ref cmd, ref _items) => {
//         match cmd.as_ref() {
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
