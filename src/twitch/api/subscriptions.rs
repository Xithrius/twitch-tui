use std::{fmt, str::FromStr};

use color_eyre::eyre::{Error, bail};
use serde::{Deserialize, Serialize};

/// Currently supported event subscription types
///
/// <https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types>/
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Subscription {
    /// Any user sends a message to a channel’s chat room
    ///
    /// <https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelchatmessage>
    #[serde(rename = "channel.chat.message")]
    Message,

    /// An event that appears in chat occurs, such as someone subscribing to the channel or a subscription is gifted
    ///
    /// <https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelchatnotification>
    #[serde(rename = "channel.chat.notification")]
    Notification,

    /// A moderator or bot clears all messages from the chat room
    ///
    /// <https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelchatclear>
    #[serde(rename = "channel.chat.clear")]
    Clear,

    /// A moderator or bot clears all messages for a specific user
    ///
    /// <https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelchatclear_user_messages>
    #[serde(rename = "channel.chat.clear_user_messages")]
    ClearUserMessages,

    /// A moderator removes a specific message
    ///
    /// <https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelchatmessage_delete>
    #[serde(rename = "channel.chat.message_delete")]
    MessageDelete,

    /// A viewer is timed out or banned from the channel.
    ///
    /// <https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#channelban>
    #[serde(rename = "channel.ban")]
    Ban,

    #[serde(other)]
    Unknown,
}

impl fmt::Display for Subscription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let subscription_str = match self {
            Self::Message => "channel.chat.message",
            Self::Notification => "channel.chat.notification",
            Self::Clear => "channel.chat.clear",
            Self::ClearUserMessages => "channel.chat.clear_user_messages",
            Self::MessageDelete => "channel.chat.message_delete",
            Self::Ban => "channel.ban",
            Self::Unknown => "unknown",
        }
        .to_string();

        write!(f, "{subscription_str}")
    }
}

impl FromStr for Subscription {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let subscription = match s.to_string().as_str() {
            "channel.chat.message" => Self::Message,
            "channel.chat.notification" => Self::Notification,
            "channel.chat.clear" => Self::Clear,
            "channel.chat.clear_user_messages" => Self::ClearUserMessages,
            "channel.chat.message_delete" => Self::MessageDelete,
            "channel.ban" => Self::Ban,
            _ => bail!("Subscription '{}' cannot be deserialized", s),
        };

        Ok(subscription)
    }
}

#[cfg(test)]
mod tests {
    use color_eyre::Result;

    use super::*;

    #[test]
    fn test_deserialize_subscription_message_type() -> Result<()> {
        let subscription_type: Subscription = serde_json::from_str("\"channel.chat.message\"")?;

        assert_eq!(subscription_type, Subscription::Message);

        Ok(())
    }

    #[test]
    fn test_serialize_subscription_message_type() -> Result<()> {
        let raw_subscription_type = serde_json::to_string(&Subscription::Message)?;

        assert_eq!(raw_subscription_type, "\"channel.chat.message\"");

        Ok(())
    }

    #[test]
    fn test_subscription_message_type_to_string() {
        let subscription_type_string = Subscription::Message.to_string();

        assert_eq!(subscription_type_string, "channel.chat.message");
    }
}
