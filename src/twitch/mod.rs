pub mod api;
mod badges;
pub mod channels;
pub mod client;
mod commands;
pub mod context;
mod models;
pub mod oauth;
mod roomstate;

#[cfg(test)]
mod tests;

use std::{collections::HashMap, str::FromStr};

use api::{
    channel_information::{
        UpdateChannelInformationPayload, get_game_id, update_channel_information,
    },
    chat_settings::{
        UpdateTwitchChatSettingsPayload, UpdateTwitchChatSettingsQuery, get_chat_settings,
        update_chat_settings,
    },
    clear::{DeleteMessageQuery, delete_twitch_messages},
    event_sub::{INITIAL_EVENT_SUBSCRIPTIONS, unsubscribe_from_events},
    mods::{ModQuery, mod_twitch_user, unmod_twitch_user},
    raids::{RaidQuery, raid_twitch_user, unraid_twitch_user},
    subscriptions::Subscription,
    timeouts::{TimeoutPayload, TimeoutQuery, UnbanQuery, timeout_twitch_user, unban_twitch_user},
    vips::{VipQuery, unvip_twitch_user, vip_twitch_user},
};
use badges::retrieve_user_badges;
use color_eyre::{
    Result,
    eyre::{Context, ContextCompat, bail},
};
use commands::TwitchCommand;
use context::TwitchWebsocketContext;
use futures::StreamExt;
use models::ReceivedTwitchEvent;
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
            commercial::{CommercialPayload, start_commercial},
            event_sub::subscribe_to_events,
            messages::{NewTwitchMessage, send_twitch_message},
            shoutouts::{ShoutoutQuery, shoutout_twitch_user},
        },
        models::ReceivedTwitchMessage,
        oauth::{get_twitch_client, get_twitch_client_oauth},
    },
    utils::text::{clean_message, parse_message_action},
};

#[derive(Debug, Clone)]
pub enum TwitchAction {
    Message(String),
    JoinChannel(String),
}

#[allow(clippy::cognitive_complexity)]
pub async fn twitch_websocket(
    mut config: CoreConfig,
    tx: Sender<TwitchToTerminalAction>,
    mut rx: Receiver<TwitchAction>,
) -> Result<()> {
    let url = config.twitch.config_twitch_websocket_url();
    let (ws_stream, _) = match connect_async(url).await {
        Ok(websocket_connection) => websocket_connection,
        Err(err) => {
            bail!(
                "Failed to connect to websocket server at {}: {}",
                config.twitch.server,
                err
            )
        }
    };

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
        let error_message = "Welcome message from websocket server was not found, something has gone terribly wrong";
        tx.send(DataBuilder::system(error_message.to_string()))
            .await?;
        bail!(error_message);
    };
    if let Err(err) = handle_welcome_message(&mut config.twitch, &mut context, &tx, message).await {
        let error_message = format!("Failed to handle welcome message: {err}");
        tx.send(DataBuilder::system(error_message.to_string()))
            .await?;
        bail!(error_message);
    }

    loop {
        tokio::select! {
            biased;

            Ok(action) = rx.recv() => {
                match action {
                    TwitchAction::Message(message) => {
                        if let Some(command) = message.strip_prefix('/') {
                            if let Err(err) = handle_command_message(&context, &tx, command).await {
                                error!("Failed to handle Twitch message command from terminal: {err}");
                                tx.send(DataBuilder::twitch(format!("Failed to handle Twitch message command from terminal: {err}"))).await?;
                            }
                        }
                        else if let Err(err) = handle_send_message(&context, message).await {
                            error!("Failed to send Twitch message from terminal: {err}");
                        }
                    },
                    TwitchAction::JoinChannel(channel_name) => {
                        let channel = if config.frontend.only_get_live_followed_channels {
                            channel_name.split(':').next().map_or(channel_name.clone(), ToString::to_string)
                        } else {
                            channel_name
                        };

                        if let Err(err) = handle_channel_join(&mut config.twitch, &mut context, &tx, channel, false).await {
                            error!("Joining channel failed: {err}");
                        }
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
                            &context,
                            &tx,
                            received_message,
                            emotes_enabled,
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

#[allow(clippy::cognitive_complexity)]
async fn handle_command_message(
    context: &TwitchWebsocketContext,
    tx: &Sender<TwitchToTerminalAction>,
    user_command: &str,
) -> Result<()> {
    let Ok(command) = TwitchCommand::from_str(user_command) else {
        tx.send(DataBuilder::system(format!(
            "Command /{user_command} either does not exist, or is not supported"
        )))
        .await?;

        return Ok(());
    };

    let twitch_client = context
        .twitch_client()
        .context("Twitch client could not be found when sending command")?;

    let channel_id = context
        .channel_id()
        .context("Channel ID could not be found when sending command")?;

    let twitch_oauth = context
        .oauth()
        .context("Twitch OAuth could not be found when sending command")?;

    let user_id = twitch_oauth.user_id.clone();

    let command_message = match command {
        TwitchCommand::Clear => {
            let delete_message_query =
                DeleteMessageQuery::new(channel_id.to_string(), user_id, None);
            delete_twitch_messages(twitch_client, delete_message_query).await?;

            "Chat was cleared for non-Moderators viewing this room".to_string()
        }
        TwitchCommand::Ban(username, reason) => {
            let target_user_id = get_channel_id(twitch_client, &username).await?;

            let ban_query = TimeoutQuery::new(channel_id.to_string(), user_id);
            let ban_payload = TimeoutPayload::new(target_user_id, None, reason.clone());

            timeout_twitch_user(twitch_client, ban_query, ban_payload).await?;

            reason.map_or_else(
                || format!("User {username} banned."),
                |reason| format!("User {username} banned. Reason: {reason}"),
            )
        }
        TwitchCommand::Timeout(username, duration, reason) => {
            let target_user_id = get_channel_id(twitch_client, &username).await?;

            let timeout_query = TimeoutQuery::new(channel_id.to_string(), user_id);
            let timeout_payload =
                TimeoutPayload::new(target_user_id, Some(duration), reason.clone());

            timeout_twitch_user(twitch_client, timeout_query, timeout_payload).await?;

            reason.map_or_else(
                || format!("User {username} timed out for {duration} seconds."),
                |reason| {
                    format!("User {username} timed out for {duration} seconds. Reason: {reason}")
                },
            )
        }
        TwitchCommand::Unban(username) => {
            let target_user_id = get_channel_id(twitch_client, &username).await?;

            let unban_query = UnbanQuery::new(channel_id.to_string(), user_id, target_user_id);

            unban_twitch_user(twitch_client, unban_query).await?;

            format!("User {username} unbanned")
        }
        TwitchCommand::Raid(username) => {
            let target_user_id = get_channel_id(twitch_client, &username).await?;

            let raid_query = RaidQuery::new(channel_id.to_string(), target_user_id);

            raid_twitch_user(twitch_client, raid_query).await?;

            format!("Raid to {username} created")
        }
        TwitchCommand::Unraid => {
            unraid_twitch_user(twitch_client, channel_id.to_string()).await?;

            "Raid cancelled".to_string()
        }
        TwitchCommand::Followers(duration) => {
            let update_query = UpdateTwitchChatSettingsQuery::new(channel_id.to_string(), user_id);
            let update_payload = UpdateTwitchChatSettingsPayload::new_follower_mode(true, duration);

            update_chat_settings(twitch_client, update_query, update_payload).await?;

            duration.map_or_else(
                || "Enabled followers-only mode for this room".to_string(),
                |duration| format!("Enabled {duration} minutes followers-only mode for this room"),
            )
        }
        TwitchCommand::FollowersOff => {
            let update_query = UpdateTwitchChatSettingsQuery::new(channel_id.to_string(), user_id);
            let update_payload = UpdateTwitchChatSettingsPayload::new_follower_mode(false, None);

            update_chat_settings(twitch_client, update_query, update_payload).await?;

            "Disabled followers-only mode for this room".to_string()
        }
        TwitchCommand::Slow(duration) => {
            let update_query = UpdateTwitchChatSettingsQuery::new(channel_id.to_string(), user_id);
            let update_payload =
                UpdateTwitchChatSettingsPayload::new_slow_mode(true, Some(duration));

            update_chat_settings(twitch_client, update_query, update_payload).await?;

            format!("Enabled {duration}-second slow mode for this room")
        }
        TwitchCommand::SlowOff => {
            let update_query = UpdateTwitchChatSettingsQuery::new(channel_id.to_string(), user_id);
            let update_payload = UpdateTwitchChatSettingsPayload::new_slow_mode(false, None);

            update_chat_settings(twitch_client, update_query, update_payload).await?;

            "Disabled slow mode for this room".to_string()
        }
        TwitchCommand::Subscribers => {
            let update_query = UpdateTwitchChatSettingsQuery::new(channel_id.to_string(), user_id);
            let update_payload = UpdateTwitchChatSettingsPayload::new_subscriber_mode(true);

            update_chat_settings(twitch_client, update_query, update_payload).await?;

            "Enabled subscribers-only mode for this room".to_string()
        }
        TwitchCommand::SubscribersOff => {
            let update_query = UpdateTwitchChatSettingsQuery::new(channel_id.to_string(), user_id);
            let update_payload = UpdateTwitchChatSettingsPayload::new_subscriber_mode(false);

            update_chat_settings(twitch_client, update_query, update_payload).await?;

            "Disabled subscribers-only mode for this room".to_string()
        }
        TwitchCommand::EmoteOnly => {
            let update_query = UpdateTwitchChatSettingsQuery::new(channel_id.to_string(), user_id);
            let update_payload = UpdateTwitchChatSettingsPayload::new_emote_only_mode(true);

            update_chat_settings(twitch_client, update_query, update_payload).await?;

            "Enabled emote-only mode for this room".to_string()
        }
        TwitchCommand::EmoteOnlyOff => {
            let update_query = UpdateTwitchChatSettingsQuery::new(channel_id.to_string(), user_id);
            let update_payload = UpdateTwitchChatSettingsPayload::new_emote_only_mode(false);

            update_chat_settings(twitch_client, update_query, update_payload).await?;

            "Disabled emote-only mode for this room".to_string()
        }
        TwitchCommand::UniqueChat => {
            let update_query = UpdateTwitchChatSettingsQuery::new(channel_id.to_string(), user_id);
            let update_payload = UpdateTwitchChatSettingsPayload::new_unique_chat_mode(true);

            update_chat_settings(twitch_client, update_query, update_payload).await?;

            "Enabled unique-chat mode for this room".to_string()
        }
        TwitchCommand::UniqueChatOff => {
            let update_query = UpdateTwitchChatSettingsQuery::new(channel_id.to_string(), user_id);
            let update_payload = UpdateTwitchChatSettingsPayload::new_unique_chat_mode(false);

            update_chat_settings(twitch_client, update_query, update_payload).await?;

            "Disabled unique-chat mode for this room".to_string()
        }
        TwitchCommand::Vip(username) => {
            let target_user_id = get_channel_id(twitch_client, &username).await?;

            let vip_query = VipQuery::new(channel_id.to_string(), target_user_id);

            vip_twitch_user(twitch_client, vip_query).await?;

            format!("Added {username} as a VIP of the channel")
        }
        TwitchCommand::Unvip(username) => {
            let target_user_id = get_channel_id(twitch_client, &username).await?;

            let unvip_query = VipQuery::new(channel_id.to_string(), target_user_id);

            unvip_twitch_user(twitch_client, unvip_query).await?;

            format!("Removed {username} as a VIP of the channel")
        }
        TwitchCommand::Mod(username) => {
            let target_user_id = get_channel_id(twitch_client, &username).await?;

            let mod_query = ModQuery::new(channel_id.to_string(), target_user_id);

            mod_twitch_user(twitch_client, mod_query).await?;

            format!("Granted moderator privledges to {username}")
        }
        TwitchCommand::Unmod(username) => {
            let target_user_id = get_channel_id(twitch_client, &username).await?;

            let unmod_query = ModQuery::new(channel_id.to_string(), target_user_id);

            unmod_twitch_user(twitch_client, unmod_query).await?;

            format!("Removed {username} as a moderator of this channel")
        }
        TwitchCommand::Shoutout(username) => {
            let target_user_id = get_channel_id(twitch_client, &username).await?;

            let shoutout_query =
                ShoutoutQuery::new(channel_id.to_string(), target_user_id, user_id);

            shoutout_twitch_user(twitch_client, shoutout_query).await?;

            format!("Gave a shoutout to {username}")
        }
        TwitchCommand::Commercial(duration) => {
            let commercial_payload = CommercialPayload::new(channel_id.to_string(), duration);
            start_commercial(twitch_client, commercial_payload).await?;

            format!("Started a commercial for {duration} seconds")
        }

        TwitchCommand::Title(title) => {
            let update_payload = UpdateChannelInformationPayload::new_title(&title);

            update_channel_information(twitch_client, channel_id.to_string(), update_payload)
                .await?;

            format!("The title of the stream was changed to {title}")
        }
        TwitchCommand::Category(game_name) => {
            let game_id = get_game_id(twitch_client, &game_name).await?;

            let update_payload = UpdateChannelInformationPayload::new_category(&game_id);

            update_channel_information(twitch_client, channel_id.to_string(), update_payload)
                .await?;

            format!("The category of the stream was changed to {game_name}")
        }
    };

    debug!("Sending command message: {command_message}");
    tx.send(DataBuilder::twitch(command_message)).await?;

    Ok(())
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
    let current_subscriptions: Vec<Subscription> = context
        .event_subscriptions()
        .keys()
        .map(std::borrow::ToOwned::to_owned)
        .collect();

    // Unsubscribe from previous channel
    if !first_channel {
        unsubscribe_from_events(
            twitch_client,
            context.event_subscriptions(),
            current_subscriptions.clone(),
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
        current_subscriptions,
    )
    .await
    .context(format!(
        "Failed to subscribe to new channel '{channel_name}'"
    ))?;

    let context_channel_id = channel_id.to_string();

    context.set_event_subscriptions(new_subscriptions);

    // Set old channel to new channel
    twitch_config.channel.clone_from(&channel_name);
    context.set_channel_id(Some(context_channel_id));

    // Notify frontend that new channel has been joined
    tx.send(DataBuilder::twitch(format!("Joined #{channel_name}")))
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

    let initial_event_subscriptions: HashMap<_, _> = INITIAL_EVENT_SUBSCRIPTIONS
        .iter()
        .cloned()
        .map(|item| (item, String::new()))
        .collect();

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

async fn handle_chat_notification(
    tx: &Sender<TwitchToTerminalAction>,
    event: ReceivedTwitchEvent,
    subscription_type: Subscription,
) -> Result<()> {
    match subscription_type {
        Subscription::Notification => {
            if let Some(twitch_notification_message) = event.system_message() {
                tx.send(DataBuilder::twitch(twitch_notification_message.to_string()))
                    .await?;
            }
        }
        Subscription::Clear => {
            tx.send(TwitchToTerminalAction::ClearChat(None)).await?;
            tx.send(DataBuilder::twitch(
                "Chat was cleared for non-Moderators viewing this room".to_string(),
            ))
            .await?;
        }
        Subscription::ClearUserMessages => {
            if let Some(target_user_id) = event.target_user_id() {
                tx.send(TwitchToTerminalAction::ClearChat(Some(
                    target_user_id.to_string(),
                )))
                .await?;
            }
        }
        Subscription::MessageDelete => {
            if let Some(message_id) = event.message_id() {
                tx.send(TwitchToTerminalAction::DeleteMessage(
                    message_id.to_string(),
                ))
                .await?;
            }
        }
        Subscription::Ban => {
            let affected_user = event
                .user_name()
                .map_or("Unknown Twitch user", |user| user.as_str());

            let timeout_message = event.timeout_duration().map_or_else(
                || format!("User {affected_user} banned"),
                |timeout_duration| {
                    format!("User {affected_user} was timed out for {timeout_duration} second(s)")
                },
            );

            tx.send(DataBuilder::twitch(timeout_message)).await?;
        }
        _ => {}
    }

    Ok(())
}

async fn handle_incoming_message(
    config: CoreConfig,
    context: &TwitchWebsocketContext,
    tx: &Sender<TwitchToTerminalAction>,
    received_message: ReceivedTwitchMessage,
    emotes_enabled: bool,
) -> Result<()> {
    // Don't allow messages from other channels go through
    if let Some(condition) = received_message.subscription_condition() {
        if context
            .channel_id()
            .is_some_and(|channel_id| channel_id != condition.broadcaster_user_id())
        {
            return Ok(());
        }
    }

    let Some(event) = received_message.event() else {
        return Ok(());
    };

    if let Some(subscription_type) = received_message.subscription_type() {
        if subscription_type != Subscription::Message {
            return handle_chat_notification(tx, event, subscription_type).await;
        }
    }

    let message_text = event
        .message_text()
        .context("Could not find message text")?;
    let (msg, highlight) = parse_message_action(&message_text);
    let received_emotes = if emotes_enabled {
        event.emote_fragments()
    } else {
        Option::default()
    }
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
