use std::fs;

use anyhow::Result;

use handlers::config::CompleteConfig;

use crate::utils::app::App;

mod handlers;
mod terminal;
mod twitch;
mod ui;
mod utils;

const CONFIG_PATH: &str = "config.toml";
const DEFAULT_CONFIG_PATH: &str = "default-config.toml";

fn main() -> Result<()> {
    if let Ok(config_contents) = fs::read_to_string(CONFIG_PATH) {
        let config: CompleteConfig = toml::from_str(config_contents.as_str())?;

        let app = App::new(config.terminal.maximum_messages as usize);

        let (tx, rx) = std::sync::mpsc::channel();
        let cloned_config = config.clone();

        std::thread::spawn(move || {
            twitch::twitch_irc(&config, &tx);
        });

        terminal::ui_driver(cloned_config, app, rx).unwrap();
    } else {
        println!(
            "Error: configuration not found. Please create a config file at '{}', and see '{}' for an example configuration.",
            CONFIG_PATH,
            DEFAULT_CONFIG_PATH,
        );
    }

    Ok(())
}
