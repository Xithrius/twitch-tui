use std::str::FromStr;

use crate::twitch::handlers::twitch_commands::TwitchCommand;

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
    );
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
    );
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
    );
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
    );
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
    );
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
    );
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
    );
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
    );
}

#[test]
fn test_twitch_command_emoteonlyoff_invalid() {
    assert!(TwitchCommand::from_str("emoteonlyoff unexpected").is_err());
}

#[test]
fn test_twitch_command_uniquechat_valid() {
    assert_eq!(
        TwitchCommand::from_str("uniquechat").unwrap(),
        TwitchCommand::UniqueChat
    );
}

#[test]
fn test_twitch_command_uniquechat_invalid() {
    assert!(TwitchCommand::from_str("uniquechat unexpected").is_err());
}

#[test]
fn test_twitch_command_uniquechatoff_valid() {
    assert_eq!(
        TwitchCommand::from_str("uniquechatoff").unwrap(),
        TwitchCommand::UniqueChatOff
    );
}

#[test]
fn test_twitch_command_uniquechatoff_invalid() {
    assert!(TwitchCommand::from_str("uniquechatoff unexpected").is_err());
}

#[test]
fn test_twitch_command_vip_valid() {
    assert_eq!(
        TwitchCommand::from_str("vip username").unwrap(),
        TwitchCommand::Vip("username".to_string())
    );
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
    );
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
    );
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
    );
}

#[test]
fn test_twitch_command_unmod_invalid() {
    assert!(TwitchCommand::from_str("unmod").is_err());
    assert!(TwitchCommand::from_str("unmod username unexpected").is_err());
}

#[test]
fn test_twitch_command_shoutout_valid() {
    assert_eq!(
        TwitchCommand::from_str("shoutout username").unwrap(),
        TwitchCommand::Shoutout("username".to_string())
    );
}

#[test]
fn test_twitch_command_shoutout_invalid() {
    assert!(TwitchCommand::from_str("shoutout").is_err());
    assert!(TwitchCommand::from_str("shoutout username unexpected").is_err());
}

#[test]
fn test_twitch_command_commercial_valid() {
    assert_eq!(
        TwitchCommand::from_str("commercial").unwrap(),
        TwitchCommand::Commercial(30)
    );
    assert_eq!(
        TwitchCommand::from_str("commercial 10").unwrap(),
        TwitchCommand::Commercial(10)
    );
}

#[test]
fn test_twitch_command_commercial_invalid() {
    assert!(TwitchCommand::from_str("commercial 30 unexpected").is_err());
    assert!(TwitchCommand::from_str("commercial string").is_err());
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
