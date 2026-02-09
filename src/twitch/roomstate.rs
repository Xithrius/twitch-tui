use std::fmt::Write as _;

use color_eyre::{Result, eyre::Context};
use tokio::sync::mpsc::Sender;

use super::api::chat_settings::TwitchChatSettingsResponse;
use crate::{events::Event, handlers::data::DataBuilder};

pub async fn handle_roomstate(
    chat_settings: &TwitchChatSettingsResponse,
    event_tx: &Sender<Event>,
) -> Result<()> {
    let mut room_state = String::new();

    if let Some(slow_mode_wait_time) = chat_settings.slow_mode() {
        writeln!(
            room_state,
            "The channel has a {slow_mode_wait_time} second slowmode."
        )?;
    }

    if let Some(follower_mode_duration) = chat_settings.follower_mode() {
        writeln!(
            room_state,
            "The channel is followers-only. You must follow the channel for at least {follower_mode_duration} second(s) to chat."
        )?;
    }

    if let Some(non_moderator_chat_delay_duration) = chat_settings.non_moderator_chat() {
        writeln!(
            room_state,
            "The channel has a non-moderator message delay. It will take {non_moderator_chat_delay_duration} second(s) for your message to show after sending."
        )?;
    }

    if chat_settings.subscriber_mode() {
        writeln!(room_state, "The channel is subscribers-only.")?;
    }

    if chat_settings.emote_mode() {
        writeln!(room_state, "The channel is emote-only.")?;
    }

    if chat_settings.unique_chat_mode() {
        writeln!(room_state, "The channel accepts only unique messages.")?;
    }

    // Trim last newline
    room_state.pop();

    if room_state.is_empty() {
        return Ok(());
    }

    event_tx
        .send(DataBuilder::twitch(room_state).into())
        .await
        .context("Failed to send room state")
}
