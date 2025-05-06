use std::str::FromStr;

use color_eyre::eyre::{Error, bail};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum TwitchCommand {
    /// Clear the chat
    Clear,
    /// Ban username with an optional reason
    Ban(String, Option<String>),
    /// Timeout for username, duration in seconds, and optional reason
    Timeout(String, usize, Option<String>),
}

impl TwitchCommand {
    fn handle_ban_command(args: &[&str]) -> Result<Self, Error> {
        match args.iter().as_slice() {
            [username] => Ok(Self::Ban((*username).to_string(), None)),
            [username, ban_reason @ ..] => {
                let reason = ban_reason.join(" ");

                Ok(Self::Ban((*username).to_string(), Some(reason)))
            }
            _ => bail!("Invalid ban command arguments"),
        }
    }

    fn handle_timeout_command(args: &[&str]) -> Result<Self, Error> {
        match args.iter().as_slice() {
            [username, timeout_duration] => {
                let duration = timeout_duration.parse::<usize>()?;

                Ok(Self::Timeout((*username).to_string(), duration, None))
            }
            [username, timeout_duration, timeout_reason @ ..] => {
                let duration = timeout_duration.parse::<usize>()?;
                let reason = timeout_reason.join(" ");

                Ok(Self::Timeout(
                    (*username).to_string(),
                    duration,
                    Some(reason),
                ))
            }
            _ => bail!("Invalid timeout command arguments"),
        }
    }
}

impl FromStr for TwitchCommand {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.trim().to_lowercase();

        let cmd = match parts.split_whitespace().collect::<Vec<&str>>().as_slice() {
            ["clear"] => Self::Clear,
            ["ban", args @ ..] => Self::handle_ban_command(args)?,
            ["timeout", args @ ..] => Self::handle_timeout_command(args)?,
            _ => bail!("Twitch command {} is not supported", s),
        };

        Ok(cmd)
    }
}
