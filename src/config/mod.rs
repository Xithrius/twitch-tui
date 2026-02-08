pub mod core;
pub mod filters;
pub mod frontend;
pub mod keybinds;
pub mod logs;
pub mod persistence;
pub mod storage;
pub mod terminal;
pub mod twitch;

pub use crate::config::{
    core::{CoreConfig, SharedCoreConfig},
    frontend::{CursorType, FrontendConfig, Palette, Theme},
    logs::LogLevel,
    twitch::TwitchConfig,
};
