use std::collections::HashMap;

use ::std::hash::BuildHasher;
use color_eyre::{
    Result,
    eyre::{Context, ContextCompat},
};
use reqwest::{Client, StatusCode};
use tracing::{debug, error};

use super::TWITCH_API_BASE_URL;
use crate::twitch::{
    api::subscriptions::Subscription,
    models::{ReceivedTwitchSubscription, TwitchSubscriptionResponse},
    oauth::TwitchOauth,
};

/// Events that should be subscribed to when the first chat room is entered.
/// Channel chat messages are excluded since it's subscribed to on channel join.
pub static INITIAL_EVENT_SUBSCRIPTIONS: &[Subscription] = &[
    Subscription::Message,
    Subscription::Notification,
    Subscription::Clear,
    Subscription::ClearUserMessages,
    Subscription::MessageDelete,
];

/// Subscribe to a set of events, returning a hashmap of subscription types corresponding to their ID
///
/// <https://dev.twitch.tv/docs/api/reference/#create-eventsub-subscription>
///
/// Different subscription types
///
/// <https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#subscription-types>
///
/// No need to delete a subscription if/when session ends, since they're disabled automatically
///
/// <https://dev.twitch.tv/docs/eventsub/handling-websocket-events/#which-events-is-my-websocket-subscribed-to>
pub async fn subscribe_to_events(
    client: &Client,
    oauth: &TwitchOauth,
    session_id: Option<String>,
    channel_id: String,
    subscription_types: Vec<Subscription>,
) -> Result<HashMap<Subscription, String>> {
    let session_id = session_id.context("Session ID is empty")?;

    let url = format!("{TWITCH_API_BASE_URL}/eventsub/subscriptions");

    let user_id = oauth
        .user_id()
        .context("Faield to get user ID from twitch OAuth context")?;

    let mut subscription = ReceivedTwitchSubscription::new(channel_id, user_id, session_id);

    let mut subscription_map = HashMap::new();

    for subscription_type in subscription_types {
        subscription.set_subscription_type(subscription_type.clone());

        let response = client.post(&url).json(&subscription).send().await?;

        if response.status() == StatusCode::CONFLICT {
            error!("Conflict on event subscription: already subscribed to {subscription_type}");
        }

        let response_data = response
            .error_for_status()?
            .json::<TwitchSubscriptionResponse>()
            .await
            .context(format!(
                "Could not deserialize {subscription_type} event subscription response"
            ))?
            .data();
        let subscription_id = response_data
            .first()
            .context("Could not get channel subscription data")?
            .id()
            .context("Could not get ID from Twitch subscription data")?;

        debug!("Subscribed to event {subscription_type}");

        subscription_map.insert(subscription_type, subscription_id.clone());
    }

    Ok(subscription_map)
}

/// Removes a subscription from the current session
///
/// <https://dev.twitch.tv/docs/api/reference/#delete-eventsub-subscription>
pub async fn unsubscribe_from_events<S: BuildHasher>(
    client: &Client,
    subscriptions: &HashMap<Subscription, String, S>,
    remove_subscription_types: Vec<Subscription>,
) -> Result<()> {
    let url = format!("{TWITCH_API_BASE_URL}/eventsub/subscriptions");

    for subscription_type in remove_subscription_types {
        let Some(subscription_id) = subscriptions.get(&subscription_type) else {
            continue;
        };

        client
            .delete(&url)
            .query(&[("id", subscription_id)])
            .send()
            .await?
            .error_for_status()
            .context("Failed to build event unsubscribe request")?;

        debug!("Unsubscribed from event {subscription_type}");
    }

    Ok(())
}
