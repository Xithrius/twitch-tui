use std::{
    env,
    fs::{create_dir_all, read_to_string},
    sync::Arc,
};

use color_eyre::eyre::{Error, Result, bail};
use serde::{Deserialize, Serialize};

use crate::{
    emotes::support_graphics_protocol,
    handlers::{
        args::{Cli, merge_args_into_config},
        config::{
            FiltersConfig, FrontendConfig, KeybindsConfig, StorageConfig, TerminalConfig,
            TwitchConfig, persist_config, persist_default_config,
            persistence::{get_cache_dir, get_config_dir},
        },
        interactive::interactive_config,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct CoreConfig {
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
    /// Keybinds for each state.
    pub keybinds: KeybindsConfig,
}

pub type SharedCoreConfig = Arc<CoreConfig>;

impl CoreConfig {
    pub fn new(cli: Cli) -> Result<Self, Error> {
        let cache_path = get_cache_dir();
        if !cache_path.exists() {
            create_dir_all(cache_path).unwrap();
        }

        let config_path = get_config_dir().join("config.toml");
        if !config_path.exists() {
            create_dir_all(config_path.parent().unwrap()).unwrap();

            if let Some(config) = interactive_config() {
                persist_config(&config_path, &config)?;
                Ok(config)
            } else {
                persist_default_config(&config_path);
                bail!(
                    "Default configuration was generated at {config_path:?}, please fill it out with necessary information."
                )
            }
        } else if let Ok(file_content) = read_to_string(config_path) {
            let mut config: Self = match toml::from_str(&file_content) {
                Ok(c) => c,
                Err(err) => bail!("Config could not be processed. Error: {:?}", err.message()),
            };

            merge_args_into_config(&mut config, cli);

            let token = env::var("TWT_TOKEN").ok();
            if let Some(env_token) = token {
                if !env_token.is_empty() {
                    config.twitch.token = Some(env_token);
                }
            }

            {
                let t = &config.twitch;

                let check_token = t.token.as_ref().map_or("", |t| t);

                if t.username.is_empty() || t.channel.is_empty() || check_token.is_empty() {
                    bail!(
                        "Twitch config section is missing one or more of the following: username, channel, token."
                    );
                }

                if config.frontend.is_emotes_enabled()
                    && !support_graphics_protocol().unwrap_or(false)
                {
                    eprintln!(
                        "This terminal does not support the graphics protocol.\nUse a terminal such as kitty, or disable emotes."
                    );
                    std::process::exit(1);
                }
            }

            // Channel names for the websocket connection can only be in lowercase.
            config.twitch.channel = config.twitch.channel.to_lowercase();

            Ok(config)
        } else {
            bail!(
                "Configuration could not be read correctly. See the following link for the example config: {}",
                format!(
                    "{}/blob/main/default-config.toml",
                    env!("CARGO_PKG_REPOSITORY")
                )
            )
        }
    }
}
