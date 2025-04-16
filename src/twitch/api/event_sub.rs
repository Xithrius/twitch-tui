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
pub async fn subscribe_to_events(
    client: &Client,
    client_id: &TwitchOauth,
    session_id: Option<String>,
    channel_id: String,
    subscription_types: Vec<String>,
) -> Result<Vec<TwitchSubscriptionResponse>> {
    let session_id = session_id.context("Session ID is empty")?;

    let url = format!("{TWITCH_API_BASE_URL}/eventsub/subscriptions");

    let mut responses = Vec::<TwitchSubscriptionResponse>::new();

    let mut subscription = ReceivedTwitchSubscription::new(
        // Set to None here so we can modify otherwise in the loop
        None,
        channel_id,
        client_id.user_id.clone(),
        session_id,
    );

    for subscription_type in subscription_types {
        subscription.set_subscription_type(subscription_type);

        let response = client.post(&url).json(&subscription).send().await?;

        let response_data: TwitchSubscriptionResponse = response.json().await?;

        responses.push(response_data);
    }

    // Example of a bad response:
    // Object {
    //     "error": String("Bad Request"),
    //     "message": String("missing or unparseable subscription condition"),
    //     "status": Number(400),
    // }

    Ok(responses)
}
