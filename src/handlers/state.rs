use std::{
    cmp::{Eq, PartialEq},
    fmt::Display,
    str::FromStr,
};

use color_eyre::eyre::{Error, Result, bail};
use serde::Serialize;
use serde_with::DeserializeFromStr;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, DeserializeFromStr, Default)]
pub enum State {
    #[default]
    Dashboard,
    Normal,
    Help,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Dashboard => "dashboard",
                Self::Normal => "normal",
                Self::Help => "help",
            }
        )
    }
}

impl FromStr for State {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "normal" | "default" | "chat" => Ok(Self::Normal),
            "dashboard" | "dash" | "start" => Ok(Self::Dashboard),
            "help" | "commands" => Ok(Self::Help),
            _ => bail!("State '{}' cannot be deserialized", s),
        }
    }
}
