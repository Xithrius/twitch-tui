mod handlers;
mod terminal;
mod twitch;
mod ui;
mod utils;

use anyhow::Result;
use structopt::StructOpt;
use tokio::sync::mpsc;

use handlers::{
    args::{merge_args_into_config, Args},
    config::CompleteConfig,
};

use crate::handlers::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    match CompleteConfig::new() {
        Ok(mut config) => {
            merge_args_into_config(&mut config, Args::from_args());

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
