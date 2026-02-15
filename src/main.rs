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

use std::{rc::Rc, sync::Arc};

use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
use logging::initialize_logging;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::{
    app::App,
    cli::args::Cli,
    config::CoreConfig,
    emotes::{Emotes, initialize_emote_decoder},
    events::Events,
    twitch::{oauth::TwitchOauth, websocket::TwitchWebsocket},
};

mod app;
mod cli;
mod commands;
mod config;
mod emotes;
mod events;
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

    let (event_tx, event_rx) = mpsc::channel(100);
    let (twitch_tx, twitch_rx) = mpsc::channel(100);

    let emotes = initialize_emote_decoder(&mut config);

    let config = Arc::new(config);

    let twitch_oauth = TwitchOauth::default().init(config.clone()).await?;
    let emotes_enabled = config.frontend.is_emotes_enabled();
    let context_emotes = Rc::new(Emotes::new(emotes_enabled));

    let context = App::new(
        config.clone(),
        twitch_oauth.clone(),
        event_tx.clone(),
        context_emotes.clone(),
    );

    let decoded_rx = if let Some((rx, cell_size)) = emotes {
        context_emotes.cell_size.get_or_init(|| cell_size);
        Some(rx)
    } else {
        None
    };

    TwitchWebsocket::new(
        config.clone(),
        twitch_oauth.clone(),
        event_tx.clone(),
        twitch_rx,
    );

    let events = Events::new(config.terminal.delay, event_tx, event_rx);
    terminal::ui_driver(config, context, events, twitch_oauth, twitch_tx, decoded_rx).await;

    std::process::exit(0)
}
