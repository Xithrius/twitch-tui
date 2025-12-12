use color_eyre::{Result, eyre::ContextCompat};

use crate::twitch::{
    api::messages::{NewTwitchMessage, send_twitch_message},
    context::TwitchWebsocketContext,
};

/// Handles the user wanting to send a message from the terminal to the WebSocket server
pub async fn handle_send_message(context: &TwitchWebsocketContext, message: String) -> Result<()> {
    let twitch_client = context
        .twitch_client()
        .context("Twitch client could not be found when sending message")?;

    let channel_id = context
        .channel_id()
        .context("Channel ID could not be found when sending message")?;

    let twitch_oauth = context
        .oauth()
        .context("Twitch OAuth could not be found when sending message")?;

    let new_message =
        NewTwitchMessage::new(channel_id.clone(), twitch_oauth.user_id.clone(), message);

    send_twitch_message(twitch_client, new_message).await?;

    Ok(())
}
