use color_eyre::{Result, eyre::ContextCompat};

use super::super::oauth::TwitchOauth;
use crate::twitch::{
    api::messages::{NewTwitchMessage, send_twitch_message},
    context::TwitchWebsocketContext,
};

/// Handles the user wanting to send a message (or a reply to `reply_parent_message_id`) from the terminal
pub async fn handle_send_message(
    context: &TwitchWebsocketContext,
    message: String,
    reply_parent_message_id: Option<String>,
) -> Result<()> {
    let twitch_client = context
        .twitch_client()
        .context("Twitch client could not be found when sending message")?;

    let channel_id = context
        .channel_id()
        .context("Channel ID could not be found when sending message")?;

    let user_id = context
        .oauth()
        .and_then(TwitchOauth::user_id)
        .context("Twitch OAuth could not be found when sending message")?;

    let new_message = NewTwitchMessage::new(
        channel_id.clone(),
        user_id.clone(),
        message,
        reply_parent_message_id,
    );

    send_twitch_message(&twitch_client, new_message).await?;

    Ok(())
}
