#![allow(clippy::use_self)]

use color_eyre::eyre::{bail, Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{create_dir_all, read_to_string, File},
    io::Write,
    path::Path,
    str::FromStr,
};

use crate::{
    emotes::{emotes_enabled, graphics_protocol},
    handlers::{
        app::State,
        args::{merge_args_into_config, Cli},
        interactive::interactive_config,
    },
    utils::pathing::{cache_path, config_path},
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
    pub token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct TerminalConfig {
    /// The delay in milliseconds between terminal updates.
    pub tick_delay: u64,
    /// The maximum amount of messages before truncation.
    pub maximum_messages: usize,
    /// The file path to log to.
    pub log_file: Option<String>,
    /// if debug logging should be enabled.
    pub verbose: bool,
    /// What state the application should start in.
    pub start_state: State,
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
    /// If the username should be shown.
    pub username_shown: bool,
    /// The color palette.
    pub palette: Palette,
    /// Show Title with time and channel.
    pub title_shown: bool,
    /// The amount of space between the chat window and the terminal border.
    pub margin: u16,
    /// Show twitch badges next to usernames.
    pub badges: bool,
    /// Theme, being either light or dark.
    pub theme: Theme,
    /// If the username should be highlighted when it appears in chat.
    pub username_highlight: bool,
    /// If there should be state tabs shown on the bottom of the terminal.
    pub state_tabs: bool,
    /// The shape of the cursor in insert boxes.
    pub cursor_shape: CursorType,
    /// If the cursor should be blinking.
    pub blinking_cursor: bool,
    /// If the scrolling should be inverted.
    pub inverted_scrolling: bool,
    /// If twitch emotes should be displayed (requires kitty terminal).
    pub twitch_emotes: bool,
    /// If betterttv emotes should be displayed (requires kitty terminal).
    pub betterttv_emotes: bool,
    /// If 7tv emotes should be displayed (requires kitty terminal).
    pub seventv_emotes: bool,
    /// Comma-separated channel names to be displayed at start screen.
    pub start_screen_channels: Vec<String>,
}

impl Default for TwitchConfig {
    fn default() -> Self {
        Self {
            username: String::new(),
            channel: String::new(),
            server: "irc.chat.twitch.tv".to_string(),
            token: None,
        }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            tick_delay: 30,
            maximum_messages: 150,
            log_file: None,
            verbose: false,
            start_state: State::Start,
        }
    }
}

impl Default for FrontendConfig {
    fn default() -> Self {
        Self {
            date_shown: true,
            date_format: "%a %b %e %T %Y".to_string(),
            username_shown: true,
            palette: Palette::default(),
            title_shown: true,
            margin: 0,
            badges: false,
            theme: Theme::default(),
            username_highlight: true,
            state_tabs: false,
            cursor_shape: CursorType::default(),
            blinking_cursor: false,
            inverted_scrolling: false,
            twitch_emotes: false,
            betterttv_emotes: false,
            seventv_emotes: false,
            start_screen_channels: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum CursorType {
    User,
    Line,
    Block,
    UnderScore,
}

impl FromStr for CursorType {
    type Err = Error;

    fn from_str(s: &str) -> Result<CursorType, Self::Err> {
        match s {
            "line" => Ok(CursorType::Line),
            "underscore" => Ok(CursorType::UnderScore),
            "block" => Ok(CursorType::Block),
            _ => Ok(CursorType::User),
        }
    }
}

impl Default for CursorType {
    fn default() -> Self {
        Self::User
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
        Self::Pastel
    }
}

impl FromStr for Palette {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "vibrant" => Ok(Self::Vibrant),
            "warm" => Ok(Self::Warm),
            "cool" => Ok(Self::Cool),
            _ => Ok(Self::Pastel),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Dark,
    Light,
    Custom,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Dark
    }
}

impl FromStr for Theme {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "light" => Ok(Self::Light),
            _ => Ok(Self::Dark),
        }
    }
}

fn persist_config(path: &Path, config: &CompleteConfig) -> Result<()> {
    let toml_string = toml::to_string(&config)?;
    let mut file = File::create(path)?;
    file.write_all(toml_string.as_bytes())?;
    drop(file);
    Ok(())
}

impl CompleteConfig {
    pub fn new(cli: Cli) -> Result<Self, Error> {
        let path_str = cache_path("");

        let p = Path::new(&path_str);
        if !p.exists() {
            create_dir_all(p).unwrap();
        }

        let path_str = config_path("config.toml");

        let p = Path::new(&path_str);

        if !p.exists() {
            create_dir_all(p.parent().unwrap()).unwrap();

            if let Some(config) = interactive_config() {
                persist_config(p, &config)?;
                Ok(config)
            } else {
                persist_config(p, &CompleteConfig::default())?;
                bail!("Configuration was generated at {path_str}, please fill it out with necessary information.")
            }
        } else if let Ok(config_contents) = read_to_string(p) {
            let mut config: CompleteConfig = toml::from_str(config_contents.as_str()).unwrap();

            merge_args_into_config(&mut config, cli);

            let token: Option<&'static str> = option_env!("TWT_TOKEN");

            if let Some(env_token) = token {
                if !env_token.is_empty() {
                    config.twitch.token = Some(env_token.to_string());
                }
            }

            {
                let t = &config.twitch;

                let check_token = t.token.as_ref().map_or("", |t| t);

                if t.username.is_empty() || t.channel.is_empty() || check_token.is_empty() {
                    bail!("Twitch config section is missing one or more of the following: username, channel, token.");
                }

                if emotes_enabled(&config.frontend)
                    && !graphics_protocol::support_graphics_protocol().unwrap_or(false)
                {
                    eprintln!("This terminal does not support the graphics protocol.\nUse a terminal such as kitty or WezTerm, or disable emotes.");
                    std::process::exit(1);
                }
            }

            // Channel names for the IRC connection can only be in lowercase.
            config.twitch.channel = config.twitch.channel.to_lowercase();

            Ok(config)
        } else {
            bail!(
                "Configuration could not be read correctly. See the following link for the example config: {}",
                format!("{}/blob/main/default-config.toml", env!("CARGO_PKG_REPOSITORY"))
            )
        }
    }
}
