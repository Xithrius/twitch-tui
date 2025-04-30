use std::{fs::File, io};

use color_eyre::eyre::Result;
use tracing_subscriber::{EnvFilter, fmt::writer::BoxMakeWriter};

use crate::handlers::config::CoreConfig;

pub fn initialize_logging(config: &CoreConfig) -> Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(config.terminal.log_level.to_string().parse()?)
        .with_env_var("TWITCH_TUI_LOG")
        .from_env_lossy()
        .add_directive("hyper_util=off".parse()?);

    let writer: BoxMakeWriter = match &config.terminal.log_file {
        Some(path) if !path.trim().is_empty() => {
            let file = File::create(path)?;
            BoxMakeWriter::new(file)
        }
        _ => BoxMakeWriter::new(io::sink),
    };

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(writer)
        .with_ansi(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
