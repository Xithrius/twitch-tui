use ::std::hash::BuildHasher;
use std::collections::HashMap;

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
///
/// Function returns a hashmap of subscription types to their ID
pub async fn subscribe_to_events(
    client: &Client,
    oauth: &TwitchOauth,
    session_id: Option<String>,
    channel_id: String,
    subscription_types: Vec<String>,
) -> Result<HashMap<String, String>> {
    let session_id = session_id.context("Session ID is empty")?;

    let url = format!("{TWITCH_API_BASE_URL}/eventsub/subscriptions");

    let mut subscription = ReceivedTwitchSubscription::new(
        // Set to None here so we can modify otherwise in the loop
        None,
        channel_id,
        oauth.user_id.clone(),
        session_id,
    );

    let mut subscription_map = HashMap::new();

    for subscription_type in subscription_types {
        subscription.set_subscription_type(subscription_type.clone());

        let response_data = client
            .post(&url)
            .json(&subscription)
            .send()
            .await?
            .error_for_status()?
            .json::<TwitchSubscriptionResponse>()
            .await?
            .data();
        let subscription_id = response_data
            .first()
            .context("Could not get channel subscription data")?
            .id()
            .context("Could not get ID from Twitch subscription data")?;

        subscription_map.insert(subscription_type, subscription_id.to_string());
    }

    Ok(subscription_map)
}

/// Removes a subscription from the current session
///
/// <https://dev.twitch.tv/docs/api/reference/#delete-eventsub-subscription>
pub async fn unsubscribe_from_events<S: BuildHasher>(
    client: &Client,
    subscriptions: &HashMap<String, String, S>,
    remove_subscription_types: Vec<String>,
) -> Result<()> {
    let url = format!("{TWITCH_API_BASE_URL}/eventsub/subscriptions");

    for subscription_type in remove_subscription_types {
        let Some(subscription_id) = subscriptions.get(&subscription_type) else {
            continue;
        };

        let response = client
            .delete(&url)
            .query(&["id", subscription_id])
            .send()
            .await?
            .error_for_status()?;

        if response.status().is_success() {}
    }

    Ok(())
}
