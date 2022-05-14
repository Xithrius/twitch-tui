mod handlers;
mod terminal;
mod twitch;
mod ui;
mod utils;

use anyhow::Result;
use clap::Parser;
use rusqlite::Connection as SqliteConnection;
use tokio::sync::mpsc;

use crate::{
    handlers::{
        app::App,
        args::{merge_args_into_config, Cli},
        config::CompleteConfig,
    },
    utils::pathing::config_path,
};

#[tokio::main]
async fn main() -> Result<()> {
    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Trace);

    let mut config = match CompleteConfig::new() {
        Ok(c) => c,
        Err(e) => panic!("Configuration error: {}", e),
    };

    merge_args_into_config(&mut config, Cli::parse());

    let sqlite_connection = SqliteConnection::open(&config_path("db.sqlite3")).unwrap();

    let app = App::new(config.clone(), sqlite_connection);

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
