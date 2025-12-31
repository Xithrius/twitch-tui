use std::{
    cell::RefCell,
    env,
    fs::{create_dir_all, read_to_string},
    path::Path,
    rc::Rc,
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
        },
        interactive::interactive_config,
    },
    utils::pathing::{cache_path, config_path},
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

pub type SharedCoreConfig = Rc<RefCell<CoreConfig>>;

impl CoreConfig {
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
                persist_default_config(p);
                bail!(
                    "Default configuration was generated at {path_str}, please fill it out with necessary information."
                )
            }
        } else if let Ok(file_content) = read_to_string(p) {
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
