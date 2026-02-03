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
    clippy::too_many_arguments
)]

use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
use logging::initialize_logging;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info, warn};

use crate::{
    emotes::initialize_emote_decoder,
    handlers::{args::Cli, config::CoreConfig, context::Context},
};

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
    color_eyre::install()?;

    let mut config = CoreConfig::new(Cli::parse()).wrap_err("Configuration error.")?;

    initialize_logging(&config).wrap_err("Failed to initialize logger")?;

    info!("Logging system initialised");

    let (twitch_tx, terminal_rx) = mpsc::channel(100);
    let (terminal_tx, twitch_rx) = broadcast::channel(100);

    let context = Context::new(config.clone());

    let decoded_rx = initialize_emote_decoder(&mut config, &context);

    let cloned_config = config.clone();

    tokio::task::spawn(async move {
        if let Err(err) = Box::pin(twitch::twitch_websocket(config, twitch_tx, twitch_rx)).await {
            error!("Error when running Twitch websocket client: {err}");
        }
    });

    terminal::ui_driver(cloned_config, context, terminal_tx, terminal_rx, decoded_rx).await;

    std::process::exit(0)
}
