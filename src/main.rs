#![warn(clippy::nursery, clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::module_name_repetitions,
    clippy::struct_excessive_bools,
    clippy::unused_self,
    clippy::future_not_send
)]

use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
use log::info;
use tokio::sync::mpsc;

use crate::handlers::{app::App, args::Cli, config::CompleteConfig};

mod handlers;
mod input;
mod terminal;
mod twitch;
mod ui;
mod utils;

fn initialize_logging(config: &CompleteConfig) {
    let logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ));
        })
        .level(if config.terminal.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        });

    if let Some(log_file_path) = config.terminal.log_file.clone() {
        if !log_file_path.is_empty() {
            logger
                .chain(fern::log_file(log_file_path).unwrap())
                .apply()
                .unwrap();
        }
    } else {
        logger.apply().unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().unwrap();

    let config = CompleteConfig::new(Cli::parse())
        .wrap_err("Configuration error.")
        .unwrap();

    initialize_logging(&config);

    info!("Logging system initialised");

    let app = App::new(&config);

    let (twitch_tx, terminal_rx) = mpsc::channel(100);
    let (terminal_tx, twitch_rx) = mpsc::channel(100);

    info!("Started tokio communication channels.");

    let cloned_config = config.clone();

    tokio::task::spawn(async move {
        twitch::twitch_irc(config, twitch_tx, twitch_rx).await;
    });

    terminal::ui_driver(cloned_config, app, terminal_tx, terminal_rx).await;

    std::process::exit(0)
}
