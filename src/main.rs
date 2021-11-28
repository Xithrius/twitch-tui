extern crate clap;

use anyhow::Result;
use tokio::sync::mpsc;

use handlers::config::{CompleteConfig, Palette};

use crate::handlers::app::App;

mod handlers;
mod terminal;
mod twitch;
mod ui;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let arg_matches = clap::App::new("twitch-tui")
        .version("1.4.1")
        .author("Xithrius")
        .about("Twitch chat in the terminal.")
        .args_from_usage("-c, --channel=[CHANNEL] 'The streamer's name'
                          -t, --tick-delay=[DELAY] 'The delay in milliseconds between terminal updates'
                          -m, --max-messages=[MESSAGES] 'The maximum amount of messages to be stored'
                          -s, --date-shown=[true/false] 'If the time and date is to be shown'
                          -u, --maximum-username-length=[LENGTH] 'The longest a username can be'
                          -a, --username-alignment=[left/center/right] 'Side the username should be aligned to'
                          -p, --palette=[PALETTE] 'The color palette for the username column: pastel (default), vibrant, warm, cool'")
        .get_matches();

    match CompleteConfig::new() {
        Ok(mut config) => {
            // twitch section of config
            if let Some(ch) = arg_matches.value_of("channel") {
                config.twitch.channel = ch.to_string();
            }
            // terminal section of config
            if let Some(tick_delay) = arg_matches.value_of("tick-delay") {
                config.terminal.tick_delay = tick_delay.parse().unwrap();
            }
            if let Some(max_messages) = arg_matches.value_of("maximum-messages") {
                config.terminal.maximum_messages = max_messages.parse().unwrap();
            }
            // frontend section of config
            if let Some(date_shown) = arg_matches.value_of("date-shown") {
                config.frontend.date_shown = date_shown.parse().unwrap();
            }
            if let Some(maximum_username_length) = arg_matches.value_of("maximum-username-length") {
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
