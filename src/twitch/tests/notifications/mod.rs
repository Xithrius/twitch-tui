mod badges;
mod bans;
mod cheers;
mod commands;
mod emotes;
mod message_fragments;
mod reply;

// Badges
static BADGES: &str = include_str!("data/badges.json");
static NO_BADGES: &str = include_str!("data/no_badges.json");
static INVALID_BADGES: &str = include_str!("data/invalid_badges.json");

// Cheers
static CHEER: &str = include_str!("data/cheer.json");
static INVALID_CHEER: &str = include_str!("data/invalid_cheer.json");

// Emotes
static EMOTE: &str = include_str!("data/emote.json");
static MANY_EMOTES: &str = include_str!("data/many_emotes.json");

// Commands
static CLEAR_COMMAND: &str = include_str!("data/clear_command.json");
static ME_COMMAND: &str = include_str!("data/me_command.json");

// Replies
static REPLY: &str = include_str!("data/reply.json");

// Bans (permanent/non-permanent timeouts)
static USER_BAN: &str = include_str!("data/user_ban.json");
static USER_TIMEOUT: &str = include_str!("data/user_timeout.json");

// Multiple message fragments (text with emotes, text with a mention, etc)
static MESSAGE_TEXT_FRAGMENT: &str = include_str!("data/message_text_fragment.json");
static MESSAGE_TEXT_EMOTE_FRAGMENTS: &str = include_str!("data/message_text_emote_fragments.json");
static MESSAGE_TEXT_MENTION_FRAGMENTS: &str =
    include_str!("data/message_text_mention_fragments.json");
