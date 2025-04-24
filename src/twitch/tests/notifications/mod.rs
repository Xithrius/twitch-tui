use std::sync::LazyLock;

mod badges;
mod cheers;
mod commands;
mod emotes;
mod message_fragments;
mod reply;

// Badges
static BADGES: LazyLock<&str> = LazyLock::new(|| include_str!("data/badges.json"));
static NO_BADGES: LazyLock<&str> = LazyLock::new(|| include_str!("data/no_badges.json"));
static INVALID_BADGES: LazyLock<&str> = LazyLock::new(|| include_str!("data/invalid_badges.json"));

// Cheers
static CHEER: LazyLock<&str> = LazyLock::new(|| include_str!("data/cheer.json"));
static INVALID_CHEER: LazyLock<&str> = LazyLock::new(|| include_str!("data/invalid_cheer.json"));

// Emotes
static EMOTE: LazyLock<&str> = LazyLock::new(|| include_str!("data/emote.json"));
static MANY_EMOTES: LazyLock<&str> = LazyLock::new(|| include_str!("data/many_emotes.json"));

// Commands
static CLEAR_COMMAND: LazyLock<&str> = LazyLock::new(|| include_str!("data/clear_command.json"));
static ME_COMMAND: LazyLock<&str> = LazyLock::new(|| include_str!("data/me_command.json"));

// Replies
static REPLY: LazyLock<&str> = LazyLock::new(|| include_str!("data/reply.json"));

// Multiple message fragments (text with emotes, text with a mention, etc)
static MESSAGE_TEXT_FRAGMENT: LazyLock<&str> =
    LazyLock::new(|| include_str!("data/message_text_fragment.json"));
static MESSAGE_TEXT_EMOTE_FRAGMENTS: LazyLock<&str> =
    LazyLock::new(|| include_str!("data/message_text_emote_fragments.json"));
static MESSAGE_TEXT_MENTION_FRAGMENTS: LazyLock<&str> =
    LazyLock::new(|| include_str!("data/message_text_mention_fragments.json"));
