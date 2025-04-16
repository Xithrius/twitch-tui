use reqwest::Client;

#[derive(Debug, Clone, Default)]
pub struct TwitchWebsocketContext {
    /// Client for doing Twitch API requests
    twitch_client: Option<Client>,

    /// Current session data
    session_id: Option<String>,
    /// Which channel ID the client is currently connected to
    channel_id: Option<String>,
    /// User ID of the user using the client
    user_id: Option<String>,

    /// Are emotes enabled right now?
    emotes_enabled: bool,
}

impl TwitchWebsocketContext {
    pub const fn twitch_client(&self) -> Option<&Client> {
        self.twitch_client.as_ref()
    }

    pub const fn session_id(&self) -> Option<&String> {
        self.session_id.as_ref()
    }

    pub const fn channel_id(&self) -> Option<&String> {
        self.channel_id.as_ref()
    }

    pub const fn user_id(&self) -> Option<&String> {
        self.user_id.as_ref()
    }

    pub fn set_twitch_client(&mut self, client: Option<Client>) {
        self.twitch_client = client;
    }

    pub fn set_session_id(&mut self, session_id: Option<String>) {
        self.session_id = session_id;
    }

    pub fn set_channel_id(&mut self, channel_id: Option<String>) {
        self.channel_id = channel_id;
    }

    pub fn set_user_id(&mut self, user_id: Option<String>) {
        self.user_id = user_id;
    }

    pub const fn set_emotes(&mut self, state: bool) {
        self.emotes_enabled = state;
    }
}
