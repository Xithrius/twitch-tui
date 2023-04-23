use std::{
    cmp::{Eq, PartialEq},
    str::FromStr,
};

use color_eyre::eyre::{bail, Error, Result};
use serde::Serialize;
use serde_with::DeserializeFromStr;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, DeserializeFromStr)]
pub enum State {
    Dashboard,
    Normal,
    Insert,
    Help,
    ChannelSwitch,
    MessageSearch,
}

impl State {
    pub const fn in_insert_mode(&self) -> bool {
        matches!(
            self,
            Self::Insert | Self::ChannelSwitch | Self::MessageSearch
        )
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
            Self::Dashboard => "Dashboard",
            Self::Normal => "Normal",
            Self::Insert => "Insert",
            Self::Help => "Help",
            Self::ChannelSwitch => "Channel",
            Self::MessageSearch => "Search",
        }
        .to_string()
    }
}

impl FromStr for State {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dashboard" | "dash" | "start" => Ok(Self::Dashboard),
            "normal" | "default" | "chat" => Ok(Self::Normal),
            "insert" | "input" => Ok(Self::Insert),
            "help" | "commands" => Ok(Self::Help),
            "channelswitcher" | "channels" => Ok(Self::ChannelSwitch),
            "messagesearch" | "search" => Ok(Self::MessageSearch),
            _ => bail!("State '{}' cannot be deserialized", s),
        }
    }
}
