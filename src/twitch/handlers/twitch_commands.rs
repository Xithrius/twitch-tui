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
    /// Turn unique chat mode on
    UniqueChat,
    /// Turn unique chat mode off
    UniqueChatOff,
    /// Vip username
    Vip(String),
    /// Unvip username
    Unvip(String),
    /// Mod username
    Mod(String),
    /// Unmod username
    Unmod(String),
    /// Shoutout username
    Shoutout(String),
    /// Start a commercial
    Commercial(usize),
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
    fn handle_slow_command(args: &[&str]) -> Result<Self, Error> {
        debug!("Slow command received as {:?}", args);
        let duration = match args.iter().as_slice() {
            [slow_duration] => slow_duration.parse::<usize>()?,
            [] => 30,
            _ => bail!("Invalid slow command arguments"),
        };
        Ok(Self::Slow(duration))
    }
    fn handle_commercial_command(args: &[&str]) -> Result<Self, Error> {
        debug!("Commercial command received as {:?}", args);
        let duration = match args.iter().as_slice() {
            [commercial_duration] => commercial_duration.parse::<usize>()?,
            [] => 30,
            _ => bail!("Invalid commercial command arguments"),
        };
        Ok(Self::Commercial(duration))
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

        let cmd = match parts.split_whitespace().collect::<Vec<&str>>().as_slice() {
            ["clear"] => Self::Clear,
            ["ban", args @ ..] => Self::handle_ban_command(args)?,
            ["unban", username] => Self::Unban((*username).to_string()),
            ["timeout", args @ ..] => Self::handle_timeout_command(args)?,
            ["raid", username] => Self::Raid((*username).to_string()),
            ["unraid"] => Self::Unraid,
            ["followers", args @ ..] => Self::handle_followers_command(args)?,
            ["followersoff"] => Self::FollowersOff,
            ["slow", args @ ..] => Self::handle_slow_command(args)?,
            ["slowoff"] => Self::SlowOff,
            ["subscribers"] => Self::Subscribers,
            ["subscribersoff"] => Self::SubscribersOff,
            ["emoteonly"] => Self::EmoteOnly,
            ["emoteonlyoff"] => Self::EmoteOnlyOff,
            ["uniquechat"] => Self::UniqueChat,
            ["uniquechatoff"] => Self::UniqueChatOff,
            ["mod", username] => Self::Mod((*username).to_string()),
            ["unmod", username] => Self::Unmod((*username).to_string()),
            ["vip", username] => Self::Vip((*username).to_string()),
            ["unvip", username] => Self::Unvip((*username).to_string()),
            ["shoutout", username] => Self::Shoutout((*username).to_string()),
            ["commercial", args @ ..] => Self::handle_commercial_command(args)?,
            ["title", args @ ..] => Self::handle_title_command(args),
            ["category", args @ ..] => Self::handle_category_command(args),
            _ => bail!("Twitch command {} is not supported", s),
        };

        Ok(cmd)
    }
}
