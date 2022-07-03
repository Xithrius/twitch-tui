use std::{
    fs::{create_dir_all, read_to_string, File},
    io::Write,
    path::Path,
    str::FromStr,
};

use color_eyre::eyre::{bail, Error, Result};
use serde::{Deserialize, Serialize};

use crate::{
    handlers::args::{merge_args_into_config, Cli},
    utils::pathing::config_path,
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct CompleteConfig {
    /// Connecting to Twitch.
    pub twitch: TwitchConfig,
    /// Internal functionality.
    pub terminal: TerminalConfig,
    /// If anything should be recorded for future use.
    pub storage: StorageConfig,
    /// Filtering out messages.
    pub filters: FiltersConfig,
    /// How everything looks to the user.
    pub frontend: FrontendConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct TwitchConfig {
    /// The current user's name.
    pub username: String,
    /// The streamer's channel name.
    pub channel: String,
    /// The IRC channel to connect to.
    pub server: String,
    /// The authentication token for the IRC.
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct TerminalConfig {
    /// The delay in milliseconds between terminal updates.
    pub tick_delay: u64,
    /// The maximum amount of messages before truncation.
    pub maximum_messages: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct StorageConfig {
    /// If previous channels switched to should be tracked.
    pub channels: bool,
    /// If previous username mentions should be tracked.
    pub mentions: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct FiltersConfig {
    /// If filters should be enabled at all.
    pub enabled: bool,
    /// If the regex filters should be reversed.
    pub reversed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
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
    pub palette: Palette,
    /// Show Title with time and channel.
    pub title_shown: bool,
    /// Show padding around chat frame.
    pub padding: bool,
    /// Show twitch badges next to usernames.
    pub badges: bool,
    /// The state of the logging window.
    pub logging: bool,
}

impl Default for TwitchConfig {
    fn default() -> Self {
        Self {
            username: "".to_string(),
            channel: "".to_string(),
            server: "irc.chat.twitch.tv".to_string(),
            token: "".to_string(),
        }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            tick_delay: 30,
            maximum_messages: 150,
        }
    }
}

impl Default for FrontendConfig {
    fn default() -> Self {
        Self {
            date_shown: true,
            date_format: "%a %b %e %T %Y".to_string(),
            maximum_username_length: 26,
            username_alignment: "right".to_string(),
            palette: Palette::default(),
            title_shown: true,
            padding: true,
            badges: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

impl CompleteConfig {
    pub fn new(cli: Cli) -> Result<Self, Error> {
        let path_str = config_path("config.toml");

        let p = Path::new(&path_str);

        if !p.exists() {
            create_dir_all(p.parent().unwrap()).unwrap();

            let default_toml_string = toml::to_string(&CompleteConfig::default()).unwrap();
            let mut file = File::create(path_str.clone()).unwrap();
            file.write_all(default_toml_string.as_bytes()).unwrap();

            bail!("Configuration was generated at {path_str}, please fill it out with necessary information.")
        } else if let Ok(config_contents) = read_to_string(&p) {
            let mut config: CompleteConfig = toml::from_str(config_contents.as_str()).unwrap();

            merge_args_into_config(&mut config, cli);

            {
                let t = &config.twitch;

                if t.username.is_empty() || t.channel.is_empty() || t.token.is_empty() {
                    bail!("Twitch config section is missing one or more of the following: username, channel, token.");
                }
            }

            Ok(config)
        } else {
            bail!(
                "Configuration could not be read correctly. See the following link for the example config: {}",
                format!("{}/blob/main/default-config.toml", env!("CARGO_PKG_REPOSITORY"))
            )
        }
    }
}
