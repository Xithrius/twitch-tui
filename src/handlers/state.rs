use std::{
    cmp::{Eq, PartialEq},
    str::FromStr,
};

use color_eyre::eyre::{bail, Error, Result};
use serde::Serialize;
use serde_with::DeserializeFromStr;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, DeserializeFromStr)]
pub enum NormalMode {
    Insert,
    Search,
}

impl ToString for NormalMode {
    fn to_string(&self) -> String {
        match self {
            Self::Insert => "insert",
            Self::Search => "search",
        }
        .to_string()
    }
}

impl FromStr for NormalMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "insert" | "input" => Ok(Self::Insert),
            "search" => Ok(Self::Search),
            _ => bail!("Normal mode '{}' cannot be deserialized", s),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, DeserializeFromStr)]
pub enum State {
    Dashboard,
    Normal,
    Help,
}

impl Default for State {
    fn default() -> Self {
        Self::Dashboard
    }
}

impl ToString for State {
    fn to_string(&self) -> String {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Normal => "Normal",
            Self::Help => "Help",
        }
        .to_string()
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
