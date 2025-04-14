use serde::{Deserialize, Serialize};

use crate::{
    emotes::DownloadedEmotes,
    handlers::data::{DataBuilder, TwitchToTerminalAction},
    utils::text::clean_message,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReceivedTwitchMessageMetadata {
    message_id: String,
    message_timestamp: String,
    message_type: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReceivedTwitchMessageSession {
    connected_at: String,
    id: String,
    keepalive_timeout_seconds: usize,
    reconnect_url: Option<String>,
    recovery_url: Option<String>,
    status: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ReceivedTwitchSubscriptionCondition {
    broadcaster_user_id: String,
    user_id: String,
}

impl ReceivedTwitchSubscriptionCondition {
    #[must_use]
    pub const fn new(broadcaster_user_id: String, user_id: String) -> Self {
        Self {
            broadcaster_user_id,
            user_id,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReceivedTwitchEventMessageFragmentCheermote {
    prefix: String,
    bits: usize,
    tier: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReceivedTwitchEventMessageFragmentEmote {
    id: Option<String>,
    emote_set_id: String,
    owner_id: String,
    format: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReceivedTwitchEventMessageFragmentMention {
    user_id: String,
    user_login: String,
    user_name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReceivedTwitchEventMessageFragment {
    #[serde(rename = "type")]
    sub_type: String,
    text: String,
    cheermote: Option<ReceivedTwitchEventMessageFragmentEmote>,
    emote: Option<ReceivedTwitchEventMessageFragmentEmote>,
    mention: Option<ReceivedTwitchEventMessageFragmentMention>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReceivedTwitchEventMessage {
    text: String,
    fragments: Vec<ReceivedTwitchEventMessageFragment>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReceivedTwitchEventBadges {
    id: String,
    set_id: String,
    info: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReceivedTwitchEventReply {
    parent_message_id: String,
    parent_message_body: String,
    parent_user_id: String,
    parent_user_name: String,
    parent_user_login: String,
    thread_message_id: String,
    thread_user_id: String,
    thread_user_name: String,
    thread_user_login: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReceivedTwitchEvent {
    broadcaster_user_id: String,
    broadcaster_user_login: String,
    broadcaster_user_name: String,
    chatter_user_id: String,
    chatter_user_login: String,
    chatter_user_name: String,
    message_id: String,
    message: ReceivedTwitchEventMessage,
    color: String,
    badges: Vec<ReceivedTwitchEventBadges>,
    message_type: String,
    cheer: Option<String>,
    reply: Option<ReceivedTwitchEventReply>,
    channel_points_custom_reward_id: Option<String>,
    source_broadcaster_user_id: Option<String>,
    source_broadcaster_user_login: Option<String>,
    source_broadcaster_user_name: Option<String>,
    source_message_id: Option<String>,
    source_badges: Option<String>,
}

impl ReceivedTwitchEvent {
    #[must_use]
    pub fn message_type(&self) -> String {
        self.message_type.clone()
    }

    #[must_use]
    pub fn chatter_information(&self) -> (String, String) {
        (self.chatter_user_name.clone(), self.message.text.clone())
    }

    pub fn build_user_data(&self) -> TwitchToTerminalAction {
        let name = self.chatter_user_name.clone();
        let user_id = self.chatter_user_id.clone();
        let cleaned_message = clean_message(&self.message.text);
        let message_id = self.message_id.clone();

        DataBuilder::user(
            name,
            Some(user_id),
            cleaned_message,
            DownloadedEmotes::default(),
            Some(message_id),
            false,
        )
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReceivedTwitchMessagePayload {
    session: Option<ReceivedTwitchMessageSession>,
    subscription: Option<ReceivedTwitchSubscription>,
    event: Option<ReceivedTwitchEvent>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ReceivedTwitchSubscriptionTransport {
    method: String,
    session_id: String,
}

impl ReceivedTwitchSubscriptionTransport {
    #[must_use]
    pub fn new(session_id: String) -> Self {
        Self {
            method: "websocket".to_string(),
            session_id,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ReceivedTwitchSubscription {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(rename = "type")]
    /// <https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#subscription-types>
    sub_type: String,
    version: String,
    condition: ReceivedTwitchSubscriptionCondition,
    transport: ReceivedTwitchSubscriptionTransport,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cost: Option<usize>,
}

impl ReceivedTwitchSubscription {
    #[must_use]
    pub fn new(sub_type: String, channel_id: String, user_id: String, session_id: String) -> Self {
        Self {
            sub_type,
            version: "1".to_string(),
            condition: ReceivedTwitchSubscriptionCondition::new(channel_id, user_id),
            transport: ReceivedTwitchSubscriptionTransport::new(session_id),
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TwitchSubscriptionResponse {
    data: Vec<ReceivedTwitchSubscription>,
    max_total_cost: usize,
    total: usize,
    total_cost: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReceivedTwitchMessage {
    metadata: Option<ReceivedTwitchMessageMetadata>,
    payload: Option<ReceivedTwitchMessagePayload>,
}

impl ReceivedTwitchMessage {
    #[must_use]
    pub fn message_type(&self) -> Option<String> {
        self.metadata
            .as_ref()
            .map(|metadata| metadata.message_type.clone())
    }

    #[must_use]
    pub fn session_id(&self) -> Option<String> {
        self.payload
            .as_ref()
            .map(|payload| payload.session.as_ref())
            .map(|session| session.map(|session| session.id.clone()))?
    }

    #[must_use]
    pub fn event(&self) -> Option<ReceivedTwitchEvent> {
        self.payload
            .as_ref()
            .and_then(|payload| payload.event.clone())
    }
}
