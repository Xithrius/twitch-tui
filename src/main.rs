use std::fs;

use anyhow::Result;

use handlers::config::CompleteConfig;

use crate::utils::app::App;

mod handlers;
mod tui;
mod twitch;
mod utils;

fn main() -> Result<()> {
    if let Ok(config_contents) = fs::read_to_string("config.toml") {
        let config: CompleteConfig = toml::from_str(config_contents.as_str()).unwrap();

        let app = App::default();

        let (tx, rx) = std::sync::mpsc::channel();
        let cloned_config = config.clone();

        std::thread::spawn(move || {
            twitch::twitch_irc(&config, &tx);
        });

        tui::tui(cloned_config, app, rx).unwrap();
    }

    Ok(())
}
