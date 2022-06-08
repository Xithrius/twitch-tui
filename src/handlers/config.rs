use std::{
    fs::{create_dir_all, read_to_string, File},
    io::Write,
    path::Path,
    str::FromStr,
};

use color_eyre::{eyre::bail, eyre::Error, eyre::Report};
use serde::Deserialize;

use crate::utils::pathing::config_path;

const CONFIG_URL: &str =
    "https://raw.githubusercontent.com/Xithrius/twitch-tui/main/default-config.toml";

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
pub struct TerminalConfig {
    /// The delay in milliseconds between terminal updates.
    pub tick_delay: u64,
    /// The maximum amount of messages before truncation.
    pub maximum_messages: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct StorageConfig {
    /// If previous channels switched to should be tracked.
    pub channels: bool,
    /// If previous username mentions should be tracked.
    pub mentions: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FiltersConfig {
    /// If filters should be enabled at all.
    pub enabled: bool,
    /// If the regex filters should be reversed.
    pub reversed: bool,
}

#[derive(Deserialize, Debug, Clone)]
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

fn download_file(url_source: &str, destination: &str) -> Result<(), ureq::Error> {
    let mut file = File::create(destination).unwrap();

    let body = ureq::get(url_source).call().unwrap().into_string().unwrap();

    file.write_all(body.as_bytes()).unwrap();

    Ok(())
}

impl CompleteConfig {
    pub fn new() -> Result<Self, Report> {
        let path_str = config_path("config.toml");

        let p = Path::new(&path_str);

        if !p.exists() {
            create_dir_all(p.parent().unwrap()).unwrap();

            download_file(CONFIG_URL, &path_str).unwrap();

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
