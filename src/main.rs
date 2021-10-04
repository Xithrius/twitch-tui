use std::fs;

use anyhow::Result;
use tokio::sync::mpsc;

use handlers::config::CompleteConfig;

use crate::utils::{app::App, pathing::config_path};

mod handlers;
mod terminal;
mod twitch;
mod ui;
mod utils;

const CONF: &str = "https://github.com/Xithrius/terminal-twitch-chat/blob/main/default-config.toml";

#[tokio::main]
async fn main() -> Result<()> {
    if let Ok(config_contents) = fs::read_to_string(config_path()) {
        let config: CompleteConfig = toml::from_str(config_contents.as_str())?;

        let app = App::new(config.terminal.maximum_messages as usize);

        let (twitch_tx, terminal_rx) = mpsc::channel(100);
        let (terminal_tx, twitch_rx) = mpsc::channel(100);
        let cloned_config = config.clone();

        tokio::task::spawn(async move {
            twitch::twitch_irc(&config, twitch_tx, twitch_rx).await;
        });

        terminal::ui_driver(&cloned_config, app, terminal_tx, terminal_rx)
            .await
            .unwrap();
        std::process::exit(0);
    } else {
        println!(
            "Configuration not found. Create a config file at '{}', and see '{}' for an example configuration.",
            config_path(),
            CONF,
        );
    }

    Ok(())
}
