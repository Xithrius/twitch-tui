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
}

impl FromStr for TwitchCommand {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.trim().to_lowercase();

        //TODO if the commands with one arg dont have that arg matched does it needed a different
        //error?
        let cmd = match parts.split_whitespace().collect::<Vec<&str>>().as_slice() {
            ["clear"] => Self::Clear,
            ["ban", args @ ..] => Self::handle_ban_command(args)?,
            ["unban", username] => Self::Unban((*username).to_string()),
            ["timeout", args @ ..] => Self::handle_timeout_command(args)?,
            ["raid", username] => Self::Raid((*username).to_string()),
            ["unraid"] => Self::Unraid,
            ["followers", args @ ..] => Self::handle_followers_command(args)?,
            ["followersoff"] => Self::FollowersOff,
            ["slow", args @ ..] => Self::handle_slow_commnad(args)?,
            ["slowoff"] => Self::SlowOff,
            ["subscribers"] => Self::Subscribers,
            ["subscribersoff"] => Self::SubscribersOff,
            ["emoteonly"] => Self::EmoteOnly,
            ["emoteonlyoff"] => Self::EmoteOnlyOff,
            ["mod", username] => Self::Mod((*username).to_string()),
            ["unmod", username] => Self::Unmod((*username).to_string()),
            ["vip", username] => Self::Vip((*username).to_string()),
            ["unvip", username] => Self::Unvip((*username).to_string()),
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

    #[test]
    fn test_twitch_command_unban_valid() {
        assert_eq!(
            TwitchCommand::from_str("unban username").unwrap(),
            TwitchCommand::Unban("username".to_string())
        );
    }

    #[test]
    fn test_twitch_command_unban_invalid() {
        assert!(TwitchCommand::from_str("unban").is_err());
        assert!(TwitchCommand::from_str("unban username unexpected").is_err());
    }

    #[test]
    fn test_twitch_command_raid_valid() {
        assert_eq!(
            TwitchCommand::from_str("raid username").unwrap(),
            TwitchCommand::Raid("username".to_string())
        )
    }

    #[test]
    fn test_twitch_command_raid_invalid() {
        assert!(TwitchCommand::from_str("raid").is_err());
        assert!(TwitchCommand::from_str("raid username unexpected").is_err());
    }

    #[test]
    fn test_twitch_command_unraid_valid() {
        assert_eq!(
            TwitchCommand::from_str("unraid").unwrap(),
            TwitchCommand::Unraid
        )
    }

    #[test]
    fn test_twitch_command_unraid_invalid() {
        assert!(TwitchCommand::from_str("unraid unexpected").is_err());
    }

    #[test]
    fn test_twitch_command_followers_valid() {
        assert_eq!(
            TwitchCommand::from_str("followers 10").unwrap(),
            TwitchCommand::Followers(Some(10))
        );
        assert_eq!(
            TwitchCommand::from_str("followers").unwrap(),
            TwitchCommand::Followers(None)
        );
    }

    #[test]
    fn test_twitch_command_followers_invalid() {
        assert!(TwitchCommand::from_str("followers string").is_err());
        assert!(TwitchCommand::from_str("followers 10 unexpected").is_err());
    }

    #[test]
    fn test_twitch_command_followersoff_valid() {
        assert_eq!(
            TwitchCommand::from_str("followersoff").unwrap(),
            TwitchCommand::FollowersOff
        )
    }

    #[test]
    fn test_twitch_command_followersoff_invalid() {
        assert!(TwitchCommand::from_str("followersoff unexpected").is_err());
    }

    #[test]
    fn test_twitch_command_slow_valid() {
        assert_eq!(
            TwitchCommand::from_str("slow").unwrap(),
            TwitchCommand::Slow(30)
        );
        assert_eq!(
            TwitchCommand::from_str("slow 10").unwrap(),
            TwitchCommand::Slow(10)
        );
    }

    #[test]
    fn test_twitch_command_slow_invalid() {
        assert!(TwitchCommand::from_str("slow 30 unexpected").is_err());
        assert!(TwitchCommand::from_str("slow string").is_err());
    }

    #[test]
    fn test_twitch_command_slowoff_valid() {
        assert_eq!(
            TwitchCommand::from_str("slowoff").unwrap(),
            TwitchCommand::SlowOff
        )
    }

    #[test]
    fn test_twitch_command_slowoff_invalid() {
        assert!(TwitchCommand::from_str("slowoff unexpected").is_err());
    }

    #[test]
    fn test_twitch_command_subscribers_valid() {
        assert_eq!(
            TwitchCommand::from_str("subscribers").unwrap(),
            TwitchCommand::Subscribers
        )
    }

    #[test]
    fn test_twitch_command_subscribers_invalid() {
        assert!(TwitchCommand::from_str("subscribers unexpected").is_err());
    }

    #[test]
    fn test_twitch_command_subscribersoff_valid() {
        assert_eq!(
            TwitchCommand::from_str("subscribersoff").unwrap(),
            TwitchCommand::SubscribersOff
        )
    }

    #[test]
    fn test_twitch_command_subscribersoff_invalid() {
        assert!(TwitchCommand::from_str("subscribersoff unexpected").is_err());
    }

    #[test]
    fn test_twitch_command_emoteonly_valid() {
        assert_eq!(
            TwitchCommand::from_str("emoteonly").unwrap(),
            TwitchCommand::EmoteOnly
        )
    }

    #[test]
    fn test_twitch_command_emoteonly_invalid() {
        assert!(TwitchCommand::from_str("emoteonly unexpected").is_err());
    }

    #[test]
    fn test_twitch_command_emoteonlyoff_valid() {
        assert_eq!(
            TwitchCommand::from_str("emoteonlyoff").unwrap(),
            TwitchCommand::EmoteOnlyOff
        )
    }

    #[test]
    fn test_twitch_command_emoteonlyoff_invalid() {
        assert!(TwitchCommand::from_str("emoteonlyoff unexpected").is_err());
    }

    #[test]
    fn test_twitch_command_vip_valid() {
        assert_eq!(
            TwitchCommand::from_str("vip username").unwrap(),
            TwitchCommand::Vip("username".to_string())
        )
    }

    #[test]
    fn test_twitch_command_vip_invalid() {
        assert!(TwitchCommand::from_str("vip").is_err());
        assert!(TwitchCommand::from_str("vip username unexpected").is_err());
    }

    #[test]
    fn test_twitch_command_unvip_valid() {
        assert_eq!(
            TwitchCommand::from_str("unvip username").unwrap(),
            TwitchCommand::Unvip("username".to_string())
        )
    }

    #[test]
    fn test_twitch_command_unvip_invalid() {
        assert!(TwitchCommand::from_str("unvip").is_err());
        assert!(TwitchCommand::from_str("unvip username unexpected").is_err());
    }

    #[test]
    fn test_twitch_command_mod_valid() {
        assert_eq!(
            TwitchCommand::from_str("mod username").unwrap(),
            TwitchCommand::Mod("username".to_string())
        )
    }

    #[test]
    fn test_twitch_command_mod_invalid() {
        assert!(TwitchCommand::from_str("mod").is_err());
        assert!(TwitchCommand::from_str("mod username unexpected").is_err());
    }

    #[test]
    fn test_twitch_command_unmod_valid() {
        assert_eq!(
            TwitchCommand::from_str("unmod username").unwrap(),
            TwitchCommand::Unmod("username".to_string())
        )
    }

    #[test]
    fn test_twitch_command_unmod_invalid() {
        assert!(TwitchCommand::from_str("unmod").is_err());
        assert!(TwitchCommand::from_str("unmod username unexpected").is_err());
    }

    #[test]
    fn test_twitch_command_title_valid() {
        assert_eq!(
            TwitchCommand::from_str("title name").unwrap(),
            TwitchCommand::Title("name".to_string())
        );
        assert_eq!(
            TwitchCommand::from_str("title name with space").unwrap(),
            TwitchCommand::Title("name with space".to_string())
        );
    }

    #[test]
    fn test_twitch_command_category_valid() {
        assert_eq!(
            TwitchCommand::from_str("category name").unwrap(),
            TwitchCommand::Category("name".to_string())
        );
        assert_eq!(
            TwitchCommand::from_str("category name with space").unwrap(),
            TwitchCommand::Category("name with space".to_string())
        );
    }
}
