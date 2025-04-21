use std::sync::LazyLock;

#[allow(clippy::module_inception)]
mod messages;

pub static BADGES: LazyLock<&str> = LazyLock::new(|| include_str!("data/badges.json"));
pub static FULL_CHEER: LazyLock<&str> = LazyLock::new(|| include_str!("data/full_cheer.json"));
pub static FULL_EMOTE: LazyLock<&str> = LazyLock::new(|| include_str!("data/full_emote.json"));
pub static FULL_MESSAGE_ONE_WORD: LazyLock<&str> =
    LazyLock::new(|| include_str!("data/full_message_one_word.json"));
pub static FULL_MESSAGE: LazyLock<&str> = LazyLock::new(|| include_str!("data/full_message.json"));
pub static MANY_EMOTES: LazyLock<&str> = LazyLock::new(|| include_str!("data/many_emotes.json"));
pub static MULTIPLE_EMOTES: LazyLock<&str> =
    LazyLock::new(|| include_str!("data/multiple_emotes.json"));
pub static PARTIAL_EMOTE_MESSAGE: LazyLock<&str> =
    LazyLock::new(|| include_str!("data/partial_emote_message.json"));
pub static PARTIAL_MENTION: LazyLock<&str> =
    LazyLock::new(|| include_str!("data/partial_mention.json"));
pub static PARTIAL_MESSAGE_EMOTE_NO_ID: LazyLock<&str> =
    LazyLock::new(|| include_str!("data/partial_message_emote_no_id.json"));
pub static PARTIAL_REPLY: LazyLock<&str> =
    LazyLock::new(|| include_str!("data/partial_reply.json"));
