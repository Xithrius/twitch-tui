use std::fs::File;

use color_eyre::eyre::Result;
use tracing_subscriber::EnvFilter;

use crate::handlers::config::CoreConfig;

pub fn initialize_logging(config: &CoreConfig) -> Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(config.terminal.log_level.to_string().parse()?)
        .with_env_var("TWITCH_TUI_LOG")
        .from_env_lossy()
        .add_directive("hyper_util=off".parse()?);

    // TODO: Temporary, remove later. This is just for debugging.
    let log_path = File::create(config.terminal.log_file.clone().unwrap()).unwrap();

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(log_path)
        .with_ansi(true)
        .without_time()
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
