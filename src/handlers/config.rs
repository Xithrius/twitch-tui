use std::str::FromStr;

use anyhow::{bail, Error, Result};
use serde::Deserialize;

use crate::utils::pathing::config_path;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Palette {
    Pastel,
    Vibrant,
    Warm,
    Cool,
}

impl FromStr for Palette {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "vibrant" => Ok(Palette::Vibrant),
            "warm" => Ok(Palette::Warm),
            "cool" => Ok(Palette::Cool),
            _ => Ok(Palette::Pastel),
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette::Pastel
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct CompleteConfig {
    /// Connecting to Twitch.
    pub twitch: TwitchConfig,
    /// Internal functionality.
    pub terminal: TerminalConfig,
    /// How everything looks to the user.
    pub frontend: FrontendConfig,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TwitchConfig {
    /// The username that this user has on Twitch.
    pub username: String,
    /// The streamer's channel name.
    pub channel: String,
    /// The IRC channel that they'd like to connect to.
    pub server: String,
    /// The OAuth token.
    pub token: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TerminalConfig {
    /// The delay between updates, in milliseconds.
    pub tick_delay: u64,
    /// The maximum amount of messages to be stored.
    pub maximum_messages: usize,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FrontendConfig {
    /// If the time and date is to be shown.
    pub date_shown: bool,
    /// The format of string that will show up in the terminal.
    pub date_format: String,
    /// The maximum length of a Twitch username.
    pub maximum_username_length: u16,
    /// Which side the username should be aligned to.
    pub username_alignment: String,
    /// The color palette.
    #[serde(default)]
    pub palette: Palette,
    /// Show Title with time and channel
    pub title_shown: bool,
    /// Show padding around chat frame
    pub padding: bool,
}

impl CompleteConfig {
    pub fn new() -> Result<Self, Error> {
        if let Ok(config_contents) = std::fs::read_to_string(config_path()) {
            let config: CompleteConfig = toml::from_str(config_contents.as_str()).unwrap();

            Ok(config)
        } else {
            bail!(
                "Configuration not found. Create a config file at '{}', and see '{}' for an example configuration.",
                config_path(),
                format!("{}/blob/main/default-config.toml", env!("CARGO_PKG_REPOSITORY"))
            )
        }
    }
}
