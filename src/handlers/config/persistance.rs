use std::{fs::File, io::Write, path::Path};

use color_eyre::eyre::Result;
use tokio::{runtime::Handle, task};

use crate::handlers::config::CoreConfig;

pub fn persist_config(path: &Path, config: &CoreConfig) -> Result<()> {
    let toml_string = toml::to_string(&config)?;
    let mut file = File::create(path)?;

    file.write_all(toml_string.as_bytes())?;
    drop(file);

    Ok(())
}

const RAW_DEFAULT_CONFIG_URL: &str =
    "https://raw.githubusercontent.com/Xithrius/twitch-tui/main/default-config.toml";

pub fn persist_default_config(path: &Path) {
    let default_config = task::block_in_place(move || {
        Handle::current().block_on(async move {
            reqwest::get(RAW_DEFAULT_CONFIG_URL)
                .await
                .unwrap()
                .text()
                .await
                .unwrap()
        })
    });

    let mut file = File::create(path).unwrap();

    file.write_all(default_config.as_bytes()).unwrap();
    drop(file);
}
