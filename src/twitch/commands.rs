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
    /// Unban username
    Unban(String),
    /// Raid a username
    Raid(String),
    /// Cancel a raid
    Unraid,
    /// Turn followers only mode on, with optional minimum follow duration in seconds
    Followers(Option<usize>),
    /// Turn followers only mode off
    FollowersOff,
    /// Turn slowmode on with duration in seconds
    Slow(usize),
    /// Turn slowmode off
    SlowOff,
    /// Turn subscribers only mode on
    Subscribers,
    /// Turn subscribers only mode off
    SubscribersOff,
    /// Turn emote only mode on
    EmoteOnly,
    /// Turn emote only mode off
    EmoteOnlyOff,
    /// Vip username
    Vip(String),
    /// Unvip username
    Unvip(String),
    /// Mod username
    Mod(String),
    /// Unmod username
    Unmod(String),
    /// Set the title of the stream
    Title(String),
    /// Set the category of a stream
    Category(String),
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

    fn handle_unban_command(args: &[&str]) -> Result<Self, Error> {
        debug!("Unban command received as {:?}", args);
        match args.iter().as_slice() {
            [username] => Ok(Self::Unban((*username).to_string())),
            _ => bail!("Invalid unban command arguments"),
        }
    }

    fn handle_raid_command(args: &[&str]) -> Result<Self, Error> {
        debug!("Raid command received as {:?}", args);
        match args.iter().as_slice() {
            [username] => Ok(Self::Raid((*username).to_string())),
            _ => bail!("Invalid raid command arguments"),
        }
    }
    fn handle_followers_command(args: &[&str]) -> Result<Self, Error> {
        debug!("Followers command received as {:?}", args);
        match args.iter().as_slice() {
            [followed_duration] => {
                let duration = followed_duration.parse::<usize>()?;

                Ok(Self::Followers(Some(duration)))
            }
            [] => Ok(Self::Followers(None)),
            _ => bail!("Invalid followers command arguments"),
        }
    }
    fn handle_slow_commnad(args: &[&str]) -> Result<Self, Error> {
        debug!("Slow command received as {:?}", args);
        match args.iter().as_slice() {
            [slow_duration] => {
                let duration = slow_duration.parse::<usize>()?;

                Ok(Self::Slow(duration))
            }
            //TODO uh does it make sense to structure it here
            [] => Ok(Self::Slow(30)),
            _ => bail!("Invalid slow command arguments"),
        }
    }
    fn handle_title_command(args: &[&str]) -> Self {
        let title = args.join(" ");
        Self::Title(title)
    }
    fn handle_category_command(args: &[&str]) -> Self {
        let game_name = args.join(" ");
        Self::Category(game_name)
    }
    fn handle_mod_command(args: &[&str]) -> Result<Self, Error> {
        debug!("Mod command received as {:?}", args);
        match args.iter().as_slice() {
            [username] => Ok(Self::Mod((*username).to_string())),
            _ => bail!("Invalid mod command arguments"),
        }
    }
    fn handle_unmod_command(args: &[&str]) -> Result<Self, Error> {
        debug!("Unmod command received as {:?}", args);
        match args.iter().as_slice() {
            [username] => Ok(Self::Unmod((*username).to_string())),
            _ => bail!("Invalid unmod command arguments"),
        }
    }
    fn handle_vip_command(args: &[&str]) -> Result<Self, Error> {
        debug!("Vip command received as {:?}", args);
        match args.iter().as_slice() {
            [username] => Ok(Self::Vip((*username).to_string())),
            _ => bail!("Invalid vip command arguments"),
        }
    }
    fn handle_unvip_command(args: &[&str]) -> Result<Self, Error> {
        debug!("Unvip command received as {:?}", args);
        match args.iter().as_slice() {
            [username] => Ok(Self::Unvip((*username).to_string())),
            _ => bail!("Invalid unvip command arguments"),
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
            ["unban", args @ ..] => Self::handle_unban_command(args)?,
            ["timeout", args @ ..] => Self::handle_timeout_command(args)?,
            ["raid", args @ ..] => Self::handle_raid_command(args)?,
            ["unraid"] => Self::Unraid,
            ["followers", args @ ..] => Self::handle_followers_command(args)?,
            ["followersoff"] => Self::FollowersOff,
            ["slow", args @ ..] => Self::handle_slow_commnad(args)?,
            ["slowoff"] => Self::SlowOff,
            ["subscribers"] => Self::Subscribers,
            ["subscribersoff"] => Self::SubscribersOff,
            ["emoteonly"] => Self::EmoteOnly,
            ["emoteonlyoff"] => Self::EmoteOnlyOff,
            ["mod", args @ ..] => Self::handle_mod_command(args)?,
            ["unmod", args @ ..] => Self::handle_unmod_command(args)?,
            ["vip", args @ ..] => Self::handle_vip_command(args)?,
            ["unvip", args @ ..] => Self::handle_unvip_command(args)?,
            ["title", args @ ..] => Self::handle_title_command(args),
            ["category", args @ ..] => Self::handle_category_command(args),
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
