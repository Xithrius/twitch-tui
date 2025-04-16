use color_eyre::{Result, eyre::ContextCompat};
use reqwest::Client;

use super::TWITCH_API_BASE_URL;
use crate::twitch::{
    models::{ReceivedTwitchSubscription, TwitchSubscriptionResponse},
    oauth::TwitchOauth,
};

pub const CHANNEL_CHAT_MESSAGE_EVENT_SUB: &str = "channel.chat.message";

/// Subscribe to a certain event
/// <https://dev.twitch.tv/docs/api/reference/#create-eventsub-subscription>
///
/// Different subscription types
/// <https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#subscription-types>
///
/// No need to delete a subscription if/when session ends, since they're disabled automatically
/// <https://dev.twitch.tv/docs/eventsub/handling-websocket-events/#which-events-is-my-websocket-subscribed-to>
pub async fn subscribe_to_event(
    client: &Client,
    client_id: &TwitchOauth,
    session_id: Option<String>,
    channel_id: String,
    subscription_type: String,
) -> Result<TwitchSubscriptionResponse> {
    let session_id = session_id.context("Session ID is empty")?;

    let url = format!("{TWITCH_API_BASE_URL}/eventsub/subscriptions");

    let subscription = ReceivedTwitchSubscription::new(
        subscription_type,
        channel_id,
        client_id.user_id.clone(),
        session_id,
    );

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&subscription)
        .send()
        .await?;

    // Example of a bad response:
    // Object {
    //     "error": String("Bad Request"),
    //     "message": String("missing or unparseable subscription condition"),
    //     "status": Number(400),
    // }

    let response_data: TwitchSubscriptionResponse = response.json().await?;

    Ok(response_data)
}
