#![forbid(unsafe_code)]
#![warn(clippy::nursery, clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::module_name_repetitions,
    clippy::struct_excessive_bools,
    clippy::unused_self,
    clippy::future_not_send,
    clippy::suboptimal_flops,
    clippy::too_many_lines,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::struct_field_names
)]

use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
use log::{info, warn};
use tokio::sync::{broadcast, mpsc};

use crate::handlers::{app::App, args::Cli, config::CompleteConfig};

mod commands;
mod emotes;
mod handlers;
mod terminal;
pub mod twitch;
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
    let startup_time = chrono::Local::now();

    color_eyre::install().unwrap();

    let config = CompleteConfig::new(Cli::parse())
        .wrap_err("Configuration error.")
        .unwrap();

    initialize_logging(&config);

    info!("Logging system initialised");

    let (twitch_tx, terminal_rx) = mpsc::channel(100);
    let (terminal_tx, twitch_rx) = broadcast::channel(100);
    let (emotes_tx, emotes_rx) = mpsc::channel(1);

    let mut app = App::new(config.clone(), startup_time);

    info!("Started tokio communication channels.");

    if emotes::emotes_enabled(&config.frontend) {
        let cloned_config = config.clone();
        let twitch_rx = twitch_rx.resubscribe();

        // We need to probe the terminal for it's size before starting the tui,
        // as writing on stdout on a different thread can interfere.
        match crossterm::terminal::window_size() {
            Ok(size) => {
                app.emotes.cell_size = (
                    f32::from(size.width / size.columns),
                    f32::from(size.height / size.rows),
                );
                info!("{:?}", app.emotes.cell_size);
                tokio::task::spawn(async move {
                    emotes::emotes(cloned_config, emotes_tx, twitch_rx).await;
                });
            }
            Err(e) => {
                warn!("Unable to query terminal for it's dimensions, disabling emotes. {e}");
            }
        }
    }

    let cloned_config = config.clone();

    tokio::task::spawn(async move {
        twitch::twitch_irc(config, twitch_tx, twitch_rx).await;
    });

    terminal::ui_driver(cloned_config, app, terminal_tx, terminal_rx, emotes_rx).await;

    std::process::exit(0)
}
