use std::fs;

use anyhow::Result;
use tokio::sync::mpsc;

use handlers::config::CompleteConfig;

use crate::utils::app::App;

mod handlers;
mod terminal;
mod twitch;
mod ui;
mod utils;

const CONFIG_PATH: &str = "config.toml";
const DEFAULT_CONFIG_PATH: &str = "default-config.toml";

#[tokio::main]
async fn main() -> Result<()> {
    if let Ok(config_contents) = fs::read_to_string(CONFIG_PATH) {
        let config: CompleteConfig = toml::from_str(config_contents.as_str())
            .expect("Could not read toml configuration file.");

        let app = App::new(config.terminal.maximum_messages as usize);

        let (twitch_tx, terminal_rx) = mpsc::channel(1);
        let (terminal_tx, twitch_rx) = mpsc::channel(1);
        let cloned_config = config.clone();

        tokio::task::spawn(async move {
            twitch::twitch_irc(&config, twitch_tx, twitch_rx).await;
        });

        terminal::ui_driver(cloned_config, app, terminal_tx, terminal_rx)
            .await
            .expect("Could not start user interface driver.");
        std::process::exit(0);
    } else {
        println!(
            "Error: configuration not found. Please create a config file at '{}', and see '{}' for an example configuration.",
            CONFIG_PATH,
            DEFAULT_CONFIG_PATH,
        );
    }

    Ok(())
}
