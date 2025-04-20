use std::{collections::HashMap, sync::LazyLock};

use ::std::hash::BuildHasher;
use color_eyre::{
    Result,
    eyre::{Context, ContextCompat},
};
use reqwest::Client;
use subscriptions::{
    CHANNEL_CHAT_CLEAR, CHANNEL_CHAT_CLEAR_USER_MESSAGES, CHANNEL_CHAT_MESSAGE_DELETE,
    CHANNEL_CHAT_NOTIFICATION,
};

use super::TWITCH_API_BASE_URL;
use crate::twitch::{
    models::{ReceivedTwitchSubscription, TwitchSubscriptionResponse},
    oauth::TwitchOauth,
};

/// Currently supported event subscription types
///
/// <https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types>/
pub mod subscriptions {
    /// Any user sends a message to a channel’s chat room
    ///
    /// <https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelchatmessage>
    pub const CHANNEL_CHAT_MESSAGE: &str = "channel.chat.message";

    /// An event that appears in chat occurs, such as someone subscribing to the channel or a subscription is gifted
    ///
    /// <https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelchatnotification>
    pub const CHANNEL_CHAT_NOTIFICATION: &str = "channel.chat.notification";

    /// A moderator or bot clears all messages from the chat room
    ///
    /// <https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelchatclear>
    pub const CHANNEL_CHAT_CLEAR: &str = "channel.chat.clear";

    /// A moderator or bot clears all messages for a specific user
    ///
    /// <https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelchatclear_user_messages>
    pub const CHANNEL_CHAT_CLEAR_USER_MESSAGES: &str = "channel.chat.clear_user_messages";

    /// A moderator removes a specific message
    ///
    /// <https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelchatmessage_delete>
    pub const CHANNEL_CHAT_MESSAGE_DELETE: &str = "channel.chat.message_delete";
}

/// Events that should be subscribed to when the first chat room is entered.
/// Channel chat messages are excluded since it's subscribed to on channel join.
pub static INITIAL_EVENT_SUBSCRIPTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| {
    vec![
        CHANNEL_CHAT_NOTIFICATION,
        CHANNEL_CHAT_CLEAR,
        CHANNEL_CHAT_CLEAR_USER_MESSAGES,
        CHANNEL_CHAT_MESSAGE_DELETE,
    ]
});

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
    subscription_types: Vec<&str>,
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
        subscription.set_subscription_type(subscription_type.to_owned());

        let response_data = client
            .post(&url)
            .json(&subscription)
            .send()
            .await?
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

        subscription_map.insert(subscription_type.to_string(), subscription_id.to_string());
    }

    Ok(subscription_map)
}

/// Removes a subscription from the current session
///
/// <https://dev.twitch.tv/docs/api/reference/#delete-eventsub-subscription>
pub async fn unsubscribe_from_events<S: BuildHasher>(
    client: &Client,
    subscriptions: &HashMap<String, String, S>,
    remove_subscription_types: Vec<&str>,
) -> Result<()> {
    let url = format!("{TWITCH_API_BASE_URL}/eventsub/subscriptions");

    for subscription_type in remove_subscription_types {
        let Some(subscription_id) = subscriptions.get(subscription_type) else {
            continue;
        };

        client
            .delete(&url)
            .query(&[("id", subscription_id)])
            .send()
            .await?
            .error_for_status()
            .context("Failed to build event unsubscribe request")?;
    }

    Ok(())
}
