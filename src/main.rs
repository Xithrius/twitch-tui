use anyhow::Result;
use tokio::sync::mpsc;

use handlers::config::CompleteConfig;

use crate::handlers::app::App;

mod handlers;
mod terminal;
mod twitch;
mod ui;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    match CompleteConfig::new() {
        Ok(config) => {
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
