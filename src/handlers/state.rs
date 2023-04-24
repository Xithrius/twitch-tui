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
    Normal(Option<NormalMode>),
    Help,
    ChannelSwitch,
}

impl State {
    pub const fn in_insert_mode(&self) -> bool {
        match self {
            Self::Normal(mode) => mode.is_some(),
            State::ChannelSwitch => true,
            _ => false,
        }
    }

    /// What general category the state can be identified with.
    pub fn category(&self) -> String {
        if self.in_insert_mode() {
            "Insert modes".to_string()
        } else {
            self.to_string()
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::Dashboard
    }
}

impl ToString for State {
    fn to_string(&self) -> String {
        match self {
            Self::Dashboard => "Dashboard".to_string(),
            Self::Normal(mode) => mode
                .as_ref()
                .map_or("Normal".to_string(), |m| format!("Normal[{m:?}]")),
            Self::Help => "Help".to_string(),
            Self::ChannelSwitch => "Channel".to_string(),
        }
    }
}

impl FromStr for State {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "normal" | "default" | "chat" => Ok(Self::Normal(None)),
            "insert" | "input" => Ok(Self::Normal(Some(NormalMode::Insert))),
            "messagesearch" | "search" => Ok(Self::Normal(Some(NormalMode::Search))),

            "dashboard" | "dash" | "start" => Ok(Self::Dashboard),
            "help" | "commands" => Ok(Self::Help),
            "channelswitcher" | "channels" => Ok(Self::ChannelSwitch),
            _ => bail!("State '{}' cannot be deserialized", s),
        }
    }
}
