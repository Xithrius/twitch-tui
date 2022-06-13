mod handlers;
mod terminal;
mod twitch;
mod ui;
mod utils;

use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
use tokio::sync::mpsc;

use crate::handlers::{
    app::App,
    args::{merge_args_into_config, Cli},
    config::CompleteConfig,
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().unwrap();

    let mut config = CompleteConfig::new()
        .wrap_err("Configuration error.")
        .unwrap();

    merge_args_into_config(&mut config, Cli::parse());

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

    std::process::exit(0)
}
