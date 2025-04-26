#![forbid(unsafe_code)]
#![warn(clippy::nursery, clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::module_name_repetitions,
    clippy::struct_excessive_bools,
    clippy::future_not_send,
    clippy::suboptimal_flops,
    clippy::too_many_lines,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::struct_field_names,
    clippy::too_many_arguments,
    clippy::unused_self
)]

use std::thread;

use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
use logging::initialize_logging;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info, warn};

use crate::handlers::{app::App, args::Cli, config::CoreConfig};

mod commands;
mod emotes;
mod handlers;
mod logging;
mod terminal;
mod twitch;
mod ui;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let startup_time = chrono::Local::now();

    color_eyre::install()?;

    let mut config = CoreConfig::new(Cli::parse()).wrap_err("Configuration error.")?;

    initialize_logging(&config).wrap_err("Failed to initialize logger")?;

    info!("Logging system initialised");

    let (twitch_tx, terminal_rx) = mpsc::channel(100);
    let (terminal_tx, twitch_rx) = broadcast::channel(100);

    let app = App::new(config.clone(), startup_time);

    let decoded_rx = if config.frontend.is_emotes_enabled() {
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
        if let Err(err) = twitch::twitch_websocket(config, twitch_tx, twitch_rx).await {
            error!("Error when running Twitch websocket client: {err}");
        }
    });

    terminal::ui_driver(cloned_config, app, terminal_tx, terminal_rx, decoded_rx).await;

    std::process::exit(0)
}
