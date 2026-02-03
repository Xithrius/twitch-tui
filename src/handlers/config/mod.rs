pub mod core;
pub mod filters;
pub mod frontend;
pub mod keybinds;
pub mod logs;
pub mod persistence;
pub mod storage;
pub mod terminal;
pub mod twitch;

pub use crate::handlers::config::{
    core::{CoreConfig, SharedCoreConfig},
    filters::FiltersConfig,
    frontend::{CursorType, FrontendConfig, Palette, Theme},
    keybinds::KeybindsConfig,
    logs::LogLevel,
    persistence::{persist_config, persist_default_config},
    storage::StorageConfig,
    terminal::TerminalConfig,
    twitch::TwitchConfig,
};
