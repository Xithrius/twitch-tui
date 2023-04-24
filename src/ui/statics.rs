use lazy_static::lazy_static;

use crate::handlers::state::{NormalMode, State};

lazy_static! {
    pub static ref HELP_COLUMN_TITLES: Vec<&'static str> = vec!["State", "Keybind", "Description"];
    pub static ref HELP_KEYBINDS: Vec<(State, Vec<(&'static str, &'static str)>)> = vec![
        (State::Normal(None), vec![
            ("c", "Main chat window"),
            ("i", "Insert mode"),
            ("@", "Insert mode with mention symbol"),
            ("/", "Insert mode with command symbol"),
            ("s", "Swap channels"),
            ("?", "* You are here!"),
            ("Ctrl + f", "Search messages"),
            ("Ctrl + p", "Manual crash"),
            ("q", "Quit"),
            ("Esc", "Go back a window layer")
        ]),
        (State::Normal(Some(NormalMode::Insert)), vec![
            ("Tab", "Fill in suggestion, if available"),
            ("Enter", "Confirm the input text to go through"),
            ("Ctrl + f", "Move cursor to the right"),
            ("Ctrl + b", "Move cursor to the left"),
            ("Ctrl + a", "Move cursor to the start"),
            ("Ctrl + e", "Move cursor to the end"),
            ("Ctrl + t", "Swap previous item with current item"),
            ("Ctrl + k", "Remove everything after the cursor"),
            ("Ctrl + u", "Remove everything before the cursor"),
            ("Ctrl + w", "Remove the previous word"),
            ("Ctrl + d", "Remove item to the right"),
            ("Ctrl + t", "Toggle the filter"),
            ("Ctrl + r", "Reverse the filter"),
            ("Alt + f", "Move to the end of the next word"),
            ("Alt + b", "Move to the start of the previous word"),
            ("Alt + t", "Swap previous word with current word"),
        ])
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

    pub static ref LINE_BUFFER_CAPACITY: usize = 4096;

    // https://discuss.dev.twitch.tv/t/irc-bot-and-message-lengths/23327/4
    pub static ref TWITCH_MESSAGE_LIMIT: usize = 500;

    // https://www.reddit.com/r/Twitch/comments/32w5b2/username_requirements/
    pub static ref NAME_RESTRICTION_REGEX: &'static str = "^[a-zA-Z0-9_]{4,25}$";
}
