use std::str::FromStr;

use color_eyre::eyre::{Error, bail};
use serde::{Deserialize, Serialize};
use tracing::debug;

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
        debug!("Ban command received as {:?}", args);

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
        debug!("Timeout command received as {:?}", args);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twitch_command_clear_valid() {
        assert_eq!(
            TwitchCommand::from_str("clear").unwrap(),
            TwitchCommand::Clear
        );
    }

    #[test]
    fn test_twitch_command_clear_invalid() {
        assert!(TwitchCommand::from_str("clear asdf").is_err());
    }

    #[test]
    fn test_twitch_command_ban_valid() {
        assert_eq!(
            TwitchCommand::from_str("ban username").unwrap(),
            TwitchCommand::Ban("username".to_string(), None)
        );
        assert_eq!(
            TwitchCommand::from_str("ban username reason").unwrap(),
            TwitchCommand::Ban("username".to_string(), Some("reason".to_string()))
        );
    }

    #[test]
    fn test_twitch_command_ban_invalid() {
        assert!(TwitchCommand::from_str("ban").is_err());
        assert!(TwitchCommand::from_str("banasdf").is_err());
    }

    #[test]
    fn test_twitch_command_timeout_valid() {
        assert_eq!(
            TwitchCommand::from_str("timeout username 10 a reason").unwrap(),
            TwitchCommand::Timeout("username".to_string(), 10, Some("a reason".to_string()))
        );
        assert_eq!(
            TwitchCommand::from_str("timeout username 10").unwrap(),
            TwitchCommand::Timeout("username".to_string(), 10, None)
        );
    }

    #[test]
    fn test_twitch_command_timeout_invalid() {
        assert!(TwitchCommand::from_str("timeout").is_err());
        assert!(TwitchCommand::from_str("timeoutasdf").is_err());
        assert!(TwitchCommand::from_str("timeout asdf").is_err());
        assert!(TwitchCommand::from_str("timeout asdf asdf").is_err());
        assert!(TwitchCommand::from_str("timeout 10 asdf").is_err());
    }
}
