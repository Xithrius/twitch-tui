use std::collections::HashMap;

use reqwest::Client;

use super::{api::subscriptions::Subscription, oauth::TwitchOauth};

#[derive(Debug, Clone, Default)]
pub struct TwitchWebsocketContext {
    /// Client for doing Twitch API requests
    client: Option<Client>,
    /// Data from the authentication endpoint for the current session
    oauth: Option<TwitchOauth>,
    /// The OAuth token for the user
    token: Option<String>,

    /// Current session ID
    session_id: Option<String>,
    /// Which channel ID the client is currently connected to
    channel_id: Option<String>,
    /// The current channel name
    channel_name: Option<String>,

    /// Are emotes enabled right now?
    emotes_enabled: bool,

    /// Events that are subscribed to in this session
    event_subscriptions: HashMap<Subscription, String>,
}

impl TwitchWebsocketContext {
    pub const fn twitch_client(&self) -> Option<&Client> {
        self.client.as_ref()
    }

    pub const fn oauth(&self) -> Option<&TwitchOauth> {
        self.oauth.as_ref()
    }

    pub fn token(self) -> Option<String> {
        self.token
    }

    pub const fn event_subscriptions(&self) -> &HashMap<Subscription, String> {
        &self.event_subscriptions
    }

    pub const fn session_id(&self) -> Option<&String> {
        self.session_id.as_ref()
    }

    pub const fn channel_id(&self) -> Option<&String> {
        self.channel_id.as_ref()
    }

    pub const fn channel_name(&self) -> Option<&String> {
        self.channel_name.as_ref()
    }

    pub fn set_twitch_client(&mut self, client: Option<Client>) {
        self.client = client;
    }

    pub fn set_oauth(&mut self, oauth: Option<TwitchOauth>) {
        self.oauth = oauth;
    }

    pub fn set_token(&mut self, token: Option<String>) {
        self.token = token;
    }

    pub fn set_session_id(&mut self, session_id: Option<String>) {
        self.session_id = session_id;
    }

    pub fn set_channel_id(&mut self, channel_id: Option<String>) {
        self.channel_id = channel_id;
    }

    pub fn set_channel_name(&mut self, channel_name: Option<String>) {
        self.channel_name = channel_name;
    }

    pub const fn set_emotes_state(&mut self, state: bool) {
        self.emotes_enabled = state;
    }

    pub const fn is_emotes_enabled(&self) -> bool {
        self.emotes_enabled
    }

    pub fn set_event_subscriptions(&mut self, event_subscriptions: HashMap<Subscription, String>) {
        self.event_subscriptions = event_subscriptions;
    }
}
