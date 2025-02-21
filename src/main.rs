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
    clippy::struct_field_names,
    clippy::too_many_arguments
)]

use std::thread;

use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
use log::{info, warn};
use tokio::sync::{broadcast, mpsc};

use crate::{
    handlers::{app::App, args::Cli, config::CompleteConfig},
    utils::emotes::emotes_enabled,
};

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

    let mut config = CompleteConfig::new(Cli::parse())
        .wrap_err("Configuration error.")
        .unwrap();

    initialize_logging(&config);

    info!("Logging system initialised");

    let (twitch_tx, terminal_rx) = mpsc::channel(100);
    let (terminal_tx, twitch_rx) = broadcast::channel(100);

    let app = App::new(config.clone(), startup_time);

    info!("Started tokio communication channels.");

    let decoded_rx = if emotes_enabled(&config.frontend) {
        // We need to probe the terminal for it's size before starting the tui,
        // as writing on stdout on a different thread can interfere.
        match crossterm::terminal::window_size() {
            Ok(size) => {
                app.emotes.cell_size.get_or_init(|| {
                    (
                        f32::from(size.width / size.columns),
                        f32::from(size.height / size.rows),
                    )
                });

                let (decoder_tx, decoder_rx) = mpsc::channel(100);
                emotes::DECODE_EMOTE_SENDER.get_or_init(|| decoder_tx);

                let (decoded_tx, decoded_rx) = mpsc::channel(100);

                // As decoding an image is a blocking task, spawn a separate thread to handle it.
                // We cannot use tokio tasks here as it will create noticeable freezes.
                thread::spawn(move || emotes::decoder(decoder_rx, &decoded_tx));

                Some(decoded_rx)
            }
            Err(e) => {
                config.frontend.twitch_emotes = false;
                config.frontend.betterttv_emotes = false;
                config.frontend.seventv_emotes = false;
                config.frontend.frankerfacez_emotes = false;
                warn!("Unable to query terminal for it's dimensions, disabling emotes. {e}");
                None
            }
        }
    } else {
        None
    };

    let cloned_config = config.clone();

    tokio::task::spawn(async move {
        twitch::twitch_irc(config, twitch_tx, twitch_rx).await;
    });

    terminal::ui_driver(cloned_config, app, terminal_tx, terminal_rx, decoded_rx).await;

    std::process::exit(0)
}
