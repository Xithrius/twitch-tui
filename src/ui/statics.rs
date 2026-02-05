pub static HELP_COLUMN_TITLES: &[&str] = &["State", "Keybind", "Description"];

// https://help.twitch.tv/s/article/chat-commands?language=en_US
pub static SUPPORTED_COMMANDS: &[&str] = &[
    "clear",
    "ban",
    "unban",
    "timeout",
    "raid",
    "unraid",
    "followers",
    "followersoff",
    "slow",
    "slowoff",
    "subscribers",
    "subscribersoff",
    "emoteonly",
    "emoteonlyoff",
    "mod",
    "unmod",
    "vip",
    "unvip",
    "shoutout",
    "commercial",
    "uniquechat",
    "uniquechatoff",
    // "color",
    // "delete",
    // "disconnect",
    // "help",
    // "host",
    // "unhost",
    // "marker",
    // "me",
    // "mods",
    // "r9kbeta",
    // "r9kbetaoff",
    // "untimeout",
    // "vips",
    // "w",

    // The following commands are not normally chat commands so they're separated for now
    "title",
    "category",
];

pub static LINE_BUFFER_CAPACITY: usize = 4096;

// https://discuss.dev.twitch.tv/t/irc-bot-and-message-lengths/23327/4
pub static TWITCH_MESSAGE_LIMIT: usize = 500;

// https://www.reddit.com/r/Twitch/comments/32w5b2/username_requirements/
// This thread is from 8 years ago, so this regex match may be outdated.
// It is now possible to have channel names be 3 characters, such as "ppy".
pub static NAME_MAX_CHARACTERS: usize = 25;
pub static NAME_RESTRICTION_REGEX: &str = "^[a-zA-Z0-9_]{3,25}$";
