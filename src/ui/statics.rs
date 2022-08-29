use lazy_static::lazy_static;

lazy_static! {
    pub static ref HELP_COLUMN_TITLES: Vec<&'static str> = vec!["Keybind", "Description"];
    pub static ref HELP_KEYBINDS: Vec<Vec<&'static str>> = vec![
        vec!["c", "Chat window"],
        vec!["i", "Insert mode"],
        vec!["s", "Swap channels"],
        vec!["Ctrl + f", "Search messages"],
        vec!["?", "Bring up this window"],
        vec!["q", "Quit this application"],
        vec!["Ctrl + p", "Manually trigger a panic"],
        vec!["Esc", "Drop back to previous window layer"],
        vec!["Ctrl + f", "Move cursor to the right"],
        vec!["Ctrl + b", "Move cursor to the left"],
        vec!["Ctrl + a", "Move cursor to the start"],
        vec!["Ctrl + e", "Move cursor to the end"],
        vec!["Alt + f", "Move to the end of the next word"],
        vec!["Alt + b", "Move to the start of the previous word"],
        vec!["Ctrl + t", "Swap previous item with current item"],
        vec!["Alt + t", "Swap previous word with current word"],
        vec!["Ctrl + u", "Remove everything before the cursor"],
        vec!["Ctrl + k", "Remove everything after the cursor"],
        vec!["Ctrl + w", "Remove the previous word"],
        vec!["Ctrl + d", "Remove item to the right"],
        vec!["Ctrl + t", "Toggle the filter"],
        vec!["Ctrl + r", "Reverse the filter"],
        vec!["Tab", "Fill in suggestion, if available"],
        vec!["Enter", "Confirm the input text to go through"],
    ];

    // https://help.twitch.tv/s/article/chat-commands?language=en_US
    pub static ref COMMANDS: Vec<&'static str> = vec![
        "ban",
        "unban",
        "clear",
        "color",
        "commercial",
        "delete",
        "disconnect",
        "emoteonly",
        "emoteonlyoff",
        "followers",
        "followersoff",
        "help",
        "host",
        "unhost",
        "marker",
        "me",
        "mod",
        "unmod",
        "mods",
        "r9kbeta",
        "r9kbetaoff",
        "raid",
        "unraid",
        "slow",
        "slowoff",
        "subscribers",
        "subscribersoff",
        "timeout",
        "untimeout",
        "vip",
        "unvip",
        "vips",
        "w",
    ];

    // https://discuss.dev.twitch.tv/t/irc-bot-and-message-lengths/23327/4
    pub static ref TWITCH_MESSAGE_LIMIT: usize = 500;

    // https://www.reddit.com/r/Twitch/comments/32w5b2/username_requirements/
    pub static ref CHANNEL_NAME_REGEX: &'static str = "^[a-zA-Z0-9_]{4,25}$";
}
