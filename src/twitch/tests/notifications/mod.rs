use std::sync::LazyLock;

mod badges;
mod cheers;
mod emotes;
#[allow(clippy::module_inception)]
mod notifications;

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

// Messages
static FULL_MESSAGE_ONE_WORD: LazyLock<&str> =
    LazyLock::new(|| include_str!("data/full_message_one_word.json"));
static FULL_MESSAGE: LazyLock<&str> = LazyLock::new(|| include_str!("data/full_message.json"));

// Partials
static PARTIAL_EMOTE_MESSAGE: LazyLock<&str> =
    LazyLock::new(|| include_str!("data/partial_emote_message.json"));
static PARTIAL_MENTION: LazyLock<&str> =
    LazyLock::new(|| include_str!("data/partial_mention.json"));
static PARTIAL_MESSAGE_EMOTE_NO_ID: LazyLock<&str> =
    LazyLock::new(|| include_str!("data/partial_message_emote_no_id.json"));
static PARTIAL_REPLY: LazyLock<&str> = LazyLock::new(|| include_str!("data/partial_reply.json"));
