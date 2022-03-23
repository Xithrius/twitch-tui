use std::{
    fs::{copy, create_dir_all, read_to_string},
    path::Path,
    str::FromStr,
};

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
    /// If anything should be recorded for future use.
    pub database: DatabaseConfig,
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
    /// Show Title with time and channel.
    pub title_shown: bool,
    /// Show padding around chat frame.
    pub padding: bool,
    /// Show twitch badges next to usernames.
    pub badges: bool,
}

#[allow(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct DatabaseConfig {
    /// If filters should be enabled at all.
    filters: bool,
    /// If previous channels switched to should be tracked.
    channels: bool,
    /// If previous username mentions should be tracked.
    mentions: bool,
}

impl CompleteConfig {
    pub fn new() -> Result<Self, Error> {
        let path_str = config_path("config.toml");

        let p = Path::new(&path_str);

        if !p.exists() {
            create_dir_all(p.parent().unwrap()).unwrap();

            copy("default-config.toml", Path::new(&path_str)).unwrap();

            bail!("Configuration was generated at {path_str}, please fill it out with necessary information.")
        } else if let Ok(config_contents) = read_to_string(&p) {
            let config: CompleteConfig = toml::from_str(config_contents.as_str()).unwrap();

            Ok(config)
        } else {
            bail!(
                "Configuration could not be read correctly. See the following link for the example config: {}",
                format!("{}/blob/main/default-config.toml", env!("CARGO_PKG_REPOSITORY"))
            )
        }
    }
}
