use color_eyre::{Result, eyre::ContextCompat};
use futures::StreamExt;
use tokio::sync::mpsc::Sender;

use crate::{
    emotes::get_twitch_emote,
    handlers::{
        config::SharedCoreConfig,
        data::{DataBuilder, TwitchToTerminalAction},
    },
    twitch::{
        api::subscriptions::Subscription,
        badges::retrieve_user_badges,
        context::TwitchWebsocketContext,
        models::{ReceivedTwitchEvent, ReceivedTwitchEventMessageFragment, ReceivedTwitchMessage},
    },
    utils::text::{clean_message, parse_message_action},
};

async fn handle_chat_notification(
    tx: &Sender<TwitchToTerminalAction>,
    event: ReceivedTwitchEvent,
    subscription_type: Subscription,
) -> Result<()> {
    match subscription_type {
        Subscription::Notification => {
            if let Some(twitch_notification_message) = event.system_message() {
                tx.send(DataBuilder::twitch(twitch_notification_message.clone()))
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
                    target_user_id.clone(),
                )))
                .await?;
            }
        }
        Subscription::MessageDelete => {
            if let Some(message_id) = event.message_id() {
                tx.send(TwitchToTerminalAction::DeleteMessage(message_id.clone()))
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

pub async fn handle_incoming_message(
    config: SharedCoreConfig,
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
        |fragment_emote: ReceivedTwitchEventMessageFragment| async move {
            let emote = fragment_emote
                .emote()
                .context("Failed to get emote from emote fragment")?;
            let emote_id = emote
                .emote_id()
                .context("Failed to get emote ID from emote fragment")?
                .clone();
            let emote_name = fragment_emote
                .emote_name()
                .context("Failed to get emote name from emote fragment")?
                .clone();

            get_twitch_emote(&emote_id).await?;

            Ok((emote_name, (emote_id, false)))
        },
    ))
    .buffer_unordered(10)
    .collect::<Vec<Result<(String, (String, bool))>>>();

    let chatter_user_name = event
        .chatter_user_name()
        .context("Could not find chatter user name")?
        .clone();
    let badges = event.badges().unwrap_or_default();
    let badges = if config.frontend.badges {
        Some(retrieve_user_badges(&badges))
    } else {
        None
    };

    let chatter_user_id = event
        .chatter_user_id()
        .context("could not find chatter user ID")?;
    let cleaned_message = clean_message(msg);
    let message_id = event
        .message_id()
        .context("Could not find message ID")?
        .clone();

    let message_emotes = emotes.await.into_iter().flatten().collect();

    tx.send(DataBuilder::user(
        chatter_user_name,
        Some(chatter_user_id.clone()),
        cleaned_message,
        message_emotes,
        Some(message_id),
        highlight,
        badges,
    ))
    .await?;

    Ok(())
}
