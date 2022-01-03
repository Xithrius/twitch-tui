mod handlers;
mod terminal;
mod twitch;
mod ui;
mod utils;

use anyhow::Result;
use tokio::sync::mpsc;

use handlers::config::{CompleteConfig, Palette};

use crate::handlers::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    let yaml = clap::load_yaml!("../cli.yml");
    let arg_matches = clap::App::from_yaml(yaml).get_matches();

    match CompleteConfig::new() {
        Ok(mut config) => {
            // Twitch section of the config
            if let Some(ch) = arg_matches.value_of("channel") {
                config.twitch.channel = ch.to_string();
            }
            // Terminal section of the config
            if let Some(tick_delay) = arg_matches.value_of("tick-delay") {
                config.terminal.tick_delay = tick_delay.parse().unwrap();
            }
            if let Some(max_messages) = arg_matches.value_of("max-messages") {
                config.terminal.maximum_messages = max_messages.parse().unwrap();
            }
            // Frontend section of the config
            if let Some(date_shown) = arg_matches.value_of("date-shown") {
                config.frontend.date_shown = date_shown.parse().unwrap();
            }
            if let Some(maximum_username_length) = arg_matches.value_of("max-username-length") {
                config.frontend.maximum_username_length = maximum_username_length.parse().unwrap();
            }
            if let Some(username_alignment) = arg_matches.value_of("username-alignment") {
                config.frontend.username_alignment = username_alignment.to_string();
            }
            if let Some(palette) = arg_matches.value_of("palette") {
                config.frontend.palette = match palette {
                    "vibrant" => Palette::Vibrant,
                    "warm" => Palette::Warm,
                    "cool" => Palette::Cool,
                    _ => Palette::Pastel,
                };
            }

            let app = App::new(config.clone());

            let (twitch_tx, terminal_rx) = mpsc::channel(100);
            let (terminal_tx, twitch_rx) = mpsc::channel(100);

            let cloned_config = config.clone();

            tokio::task::spawn(async move {
                twitch::twitch_irc(config, twitch_tx, twitch_rx).await;
            });

            terminal::ui_driver(cloned_config, app, terminal_tx, terminal_rx)
                .await
                .unwrap();

            std::process::exit(0);
        }
        Err(err) => {
            println!("{}", err);
        }
    }

    Ok(())
}
