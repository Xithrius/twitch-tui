use serde::{Deserialize, Serialize};

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
    fragment_type: String,
    text: String,
    cheermote: Option<ReceivedTwitchEventMessageFragmentCheermote>,
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

impl ReceivedTwitchEventBadges {
    pub fn set_id(&self) -> &str {
        self.set_id.as_ref()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReceivedTwitchEventCheer {
    bits: u64,
}

impl ReceivedTwitchEventCheer {
    #[cfg(test)]
    pub const fn bits(&self) -> u64 {
        self.bits
    }
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

impl ReceivedTwitchEventReply {
    #[cfg(test)]
    pub const fn parent_message_body(&self) -> &String {
        &self.parent_message_body
    }
}

/// All attributes that are to come through during a channel chat notification event
///
/// <https://dev.twitch.tv/docs/eventsub/eventsub-reference/#channel-chat-notification-event>
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReceivedTwitchEvent {
    broadcaster_user_id: String,
    broadcaster_user_login: String,
    broadcaster_user_name: String,
    chatter_user_id: Option<String>,
    chatter_user_login: Option<String>,
    chatter_user_name: Option<String>,
    color: Option<String>,
    message_id: Option<String>,
    message_type: Option<String>,
    message: Option<ReceivedTwitchEventMessage>,
    system_message: Option<String>,
    badges: Option<Vec<ReceivedTwitchEventBadges>>,
    cheer: Option<ReceivedTwitchEventCheer>,
    reply: Option<ReceivedTwitchEventReply>,
    channel_points_custom_reward_id: Option<String>,
    source_broadcaster_user_id: Option<String>,
    source_broadcaster_user_login: Option<String>,
    source_broadcaster_user_name: Option<String>,
    source_message_id: Option<String>,
    source_badges: Option<String>,
}

impl ReceivedTwitchEventMessageFragmentEmote {
    pub fn emote_id(&self) -> Option<String> {
        self.id.clone()
    }
}

impl ReceivedTwitchEventMessageFragment {
    pub fn emote(&self) -> Option<ReceivedTwitchEventMessageFragmentEmote> {
        self.emote.clone()
    }

    pub fn emote_name(&self) -> Option<&String> {
        self.emote.is_some().then_some(&self.text)
    }

    #[cfg(test)]
    pub fn text(&self) -> String {
        self.text.clone()
    }
}

impl ReceivedTwitchEvent {
    pub const fn chatter_user_id(&self) -> Option<&String> {
        self.chatter_user_id.as_ref()
    }

    pub const fn chatter_user_name(&self) -> Option<&String> {
        self.chatter_user_name.as_ref()
    }

    pub const fn message_id(&self) -> Option<&String> {
        self.message_id.as_ref()
    }

    pub fn message_text(&self) -> Option<String> {
        self.message.as_ref().map(|message| message.text.clone())
    }

    pub const fn system_message(&self) -> Option<&String> {
        self.system_message.as_ref()
    }

    pub fn badges(&self) -> Option<Vec<ReceivedTwitchEventBadges>> {
        self.badges.clone()
    }

    #[cfg(test)]
    pub const fn cheer(&self) -> Option<&ReceivedTwitchEventCheer> {
        self.cheer.as_ref()
    }

    #[cfg(test)]
    pub fn fragments(&self) -> Option<Vec<ReceivedTwitchEventMessageFragment>> {
        self.message
            .as_ref()
            .map(|message| message.fragments.clone())
    }

    pub fn emote_fragments(&self) -> Option<Vec<ReceivedTwitchEventMessageFragment>> {
        self.message.as_ref().map(|message| {
            message
                .fragments
                .iter()
                .filter(|fragment| fragment.emote.is_some())
                .cloned()
                .collect()
        })
    }

    #[cfg(test)]
    pub const fn reply(&self) -> Option<&ReceivedTwitchEventReply> {
        self.reply.as_ref()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReceivedTwitchMessagePayload {
    session: Option<ReceivedTwitchMessageSession>,
    subscription: Option<ReceivedTwitchSubscription>,
    event: Option<ReceivedTwitchEvent>,
}

impl ReceivedTwitchMessagePayload {
    #[cfg(test)]
    pub const fn event(&self) -> Option<&ReceivedTwitchEvent> {
        self.event.as_ref()
    }

    #[cfg(test)]
    pub const fn subscription(&self) -> Option<&ReceivedTwitchSubscription> {
        self.subscription.as_ref()
    }
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
    id: Option<String>,
    status: Option<String>,
    #[serde(rename = "type")]
    subscription_type: Option<String>,
    version: String,
    condition: ReceivedTwitchSubscriptionCondition,
    transport: ReceivedTwitchSubscriptionTransport,
    created_at: Option<String>,
    cost: Option<usize>,
}

impl ReceivedTwitchSubscription {
    #[must_use]
    pub fn new(
        maybe_subscription_type: Option<String>,
        channel_id: String,
        user_id: String,
        session_id: String,
    ) -> Self {
        Self {
            subscription_type: maybe_subscription_type,
            version: "1".to_string(),
            condition: ReceivedTwitchSubscriptionCondition::new(channel_id, user_id),
            transport: ReceivedTwitchSubscriptionTransport::new(session_id),
            ..Default::default()
        }
    }

    pub const fn subscription_type(&self) -> Option<&String> {
        self.subscription_type.as_ref()
    }

    pub fn set_subscription_type(&mut self, subscription_type: String) {
        self.subscription_type = Some(subscription_type);
    }

    pub const fn id(&self) -> Option<&String> {
        self.id.as_ref()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TwitchSubscriptionResponse {
    data: Vec<ReceivedTwitchSubscription>,
    max_total_cost: usize,
    total: usize,
    total_cost: usize,
}

impl TwitchSubscriptionResponse {
    pub fn data(&self) -> Vec<ReceivedTwitchSubscription> {
        self.data.clone()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReceivedTwitchMessage {
    metadata: Option<ReceivedTwitchMessageMetadata>,
    payload: Option<ReceivedTwitchMessagePayload>,
}

impl ReceivedTwitchMessage {
    pub fn subscription_type(&self) -> Option<String> {
        self.payload
            .as_ref()?
            .subscription
            .as_ref()?
            .subscription_type()
            .cloned()
    }

    #[must_use]
    pub fn session_id(&self) -> Option<String> {
        self.payload
            .as_ref()?
            .session
            .as_ref()
            .map(|session| session.id.clone())
    }

    #[must_use]
    pub fn event(&self) -> Option<ReceivedTwitchEvent> {
        self.payload.as_ref()?.event.clone()
    }
}
