use std::string::ToString;

use color_eyre::{Result, eyre::ContextCompat};

use super::NO_BADGES;
use crate::twitch::{
    models::ReceivedTwitchMessagePayload,
    tests::messages::{
        BADGES, FULL_CHEER, FULL_EMOTE, FULL_MESSAGE, FULL_MESSAGE_ONE_WORD, MANY_EMOTES,
        MULTIPLE_EMOTES, PARTIAL_EMOTE_MESSAGE, PARTIAL_MENTION, PARTIAL_MESSAGE_EMOTE_NO_ID,
        PARTIAL_REPLY,
    },
};

fn deserialize_message(message: &str) -> Result<ReceivedTwitchMessagePayload, serde_json::Error> {
    serde_json::from_str::<ReceivedTwitchMessagePayload>(message)
}

#[test]
fn test_deserialize_badges() -> Result<()> {
    let raw_message: serde_json::Value = serde_json::from_str(&BADGES)?;
    let message = serde_json::from_str::<ReceivedTwitchMessagePayload>(&BADGES)?;

    let raw_badges: Vec<String> = raw_message
        .pointer("/event/badges")
        .context("Could not find badges")?
        .as_array()
        .context("Could not convert badges to array")?
        .iter()
        .filter_map(|item| {
            item.get("set_id")
                .and_then(|set_id| set_id.as_str().map(ToString::to_string))
        })
        .collect();

    let badges: Vec<String> = message
        .event()
        .clone()
        .context("Could not find badges deserialized event")?
        .badges()
        .iter()
        .map(|badge| badge.set_id().to_string())
        .collect();

    assert_eq!(raw_badges, badges);

    Ok(())
}

#[test]
#[should_panic(expected = "Missing badges field")]
fn test_deserialize_no_badges() {
    let _ = serde_json::from_str::<ReceivedTwitchMessagePayload>(&NO_BADGES)
        .expect("Missing badges field");
}

#[test]
fn test_deserialize_full_cheer() -> Result<()> {
    let raw_message: serde_json::Value = serde_json::from_str(&FULL_CHEER)?;
    let message = serde_json::from_str::<ReceivedTwitchMessagePayload>(&FULL_CHEER)?;

    let raw_bits = raw_message
        .pointer("/event/cheer/bits")
        .context("Could not find cheer bits")?
        .as_u64()
        .context("Cheer could not be converted to u64")?;

    let bits = message
        .event()
        .clone()
        .context("Could not find cheer deserialized event")?
        .cheer()
        .clone()
        .context("Could not find cheer in event")?
        .bits();

    assert_eq!(raw_bits, bits);

    Ok(())
}

#[test]
fn test_deserialize_full_emote() {
    assert!(deserialize_message(&FULL_EMOTE).is_ok());
}

#[test]
fn test_deserialize_full_message_one_word() {
    assert!(deserialize_message(&FULL_MESSAGE_ONE_WORD).is_ok());
}

#[test]
fn test_deserialize_full_message() {
    assert!(deserialize_message(&FULL_MESSAGE).is_ok());
}

#[test]
fn test_deserialize_many_emotes() {
    assert!(deserialize_message(&MANY_EMOTES).is_ok());
}

#[test]
fn test_deserialize_multiple_emotes() {
    assert!(deserialize_message(&MULTIPLE_EMOTES).is_ok());
}

#[test]
fn test_deserialize_partial_emote_message() {
    assert!(deserialize_message(&PARTIAL_EMOTE_MESSAGE).is_ok());
}

#[test]
fn test_deserialize_partial_mention() {
    assert!(deserialize_message(&PARTIAL_MENTION).is_ok());
}

#[test]
fn test_deserialize_partial_message_emote_no_id() {
    assert!(deserialize_message(&PARTIAL_MESSAGE_EMOTE_NO_ID).is_ok());
}

#[test]
fn test_deserialize_partial_reply() {
    assert!(deserialize_message(&PARTIAL_REPLY).is_ok());
}
