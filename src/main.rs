mod handlers;
mod terminal;
mod twitch;
mod ui;
mod utils;

use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
use tokio::sync::mpsc;

use crate::handlers::{app::App, args::Cli, config::CompleteConfig};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().unwrap();

    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Trace);

    let config = CompleteConfig::new(Cli::parse())
        .wrap_err("Configuration error.")
        .unwrap();

    let app = App::new(config.clone());

    let (twitch_tx, terminal_rx) = mpsc::channel(100);
    let (terminal_tx, twitch_rx) = mpsc::channel(100);

    let cloned_config = config.clone();

    tokio::task::spawn(async move {
        twitch::twitch_irc(config, twitch_tx, twitch_rx).await;
    });

    terminal::ui_driver(cloned_config, app, terminal_tx, terminal_rx).await;

    std::process::exit(0)
}
