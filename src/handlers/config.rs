use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Palette {
    Pastel,
    Vibrant,
    Warm,
    Cool,
}

impl Default for Palette {
    fn default() -> Self {
        Palette::Pastel
    }
}

#[derive(Deserialize, Clone)]
pub struct CompleteConfig {
    /// Connecting to Twitch.
    pub twitch: TwitchConfig,
    /// Internal functionality.
    pub terminal: TerminalConfig,
    /// How everything looks to the user.
    pub frontend: FrontendConfig,
    /// All the keybinds on the keyboard.
    pub keybinds: KeybindsConfig,
}

#[derive(Deserialize, Clone)]
pub struct TwitchConfig {
    /// The username that this user has on Twitch.
    pub username: String,
    /// The streamer's channel name.
    pub channel: String,
    /// The IRC channel that they'd like to connect to.
    pub server: String,
}

#[derive(Deserialize, Clone)]
pub struct TerminalConfig {
    /// The delay between updates, in milliseconds.
    pub tick_delay: u64,
    /// The maximum amount of messages to be stored.
    pub maximum_messages: u64,
}

#[derive(Deserialize, Clone)]
pub struct FrontendConfig {
    // If the time and date is to be shown.
    pub date_shown: bool,
    /// The format of string that will show up in the terminal.
    pub date_format: String,
    /// The maximum length of a Twitch username.
    pub maximum_username_length: u16,
    /// Which side the username should be aligned to.
    pub username_alignment: String,
    /// The color palette
    #[serde(default)]
    pub palette: Palette,
}

#[derive(Deserialize, Clone)]
pub struct KeybindsConfig {
    /// Chat table.
    pub chat: char,
    /// Keybinds table.
    pub keybinds: char,
    /// Users in chat.
    pub users: char,
    /// Quit application (the ESC key will always be enabled).
    pub quit: char,
}
