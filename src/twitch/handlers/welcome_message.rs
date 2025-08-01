use std::collections::HashMap;

use color_eyre::{
    Result,
    eyre::{Context, ContextCompat},
};
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::Utf8Bytes;

use crate::{
    handlers::{
        config::TwitchConfig,
        data::{DataBuilder, TwitchToTerminalAction},
    },
    twitch::{
        api::{
            channels::get_channel_id,
            chat_settings::get_chat_settings,
            event_sub::{
                INITIAL_EVENT_SUBSCRIPTIONS, subscribe_to_events, unsubscribe_from_events,
            },
            subscriptions::Subscription,
        },
        context::TwitchWebsocketContext,
        models::ReceivedTwitchMessage,
        oauth::{get_twitch_client, get_twitch_client_oauth},
        roomstate::handle_roomstate,
    },
};

/// Handling either the terminal joining a new channel, or the application just starting up
pub async fn handle_channel_join(
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

pub async fn handle_welcome_message(
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
