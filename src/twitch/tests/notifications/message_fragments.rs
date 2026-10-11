use std::string::ToString;

use color_eyre::{Result, eyre::ContextCompat};

use crate::twitch::{
    models::{ReceivedTwitchEventMessageFragment, ReceivedTwitchMessagePayload},
    tests::{
        notifications::{
            MESSAGE_TEXT_EMOTE_FRAGMENTS, MESSAGE_TEXT_FRAGMENT, MESSAGE_TEXT_MENTION_FRAGMENTS,
        },
        utils::load_data,
    },
};

fn build_message_text_vec(
    raw: &serde_json::Value,
    message: &ReceivedTwitchMessagePayload,
) -> Result<(Vec<String>, Vec<String>)> {
    let raw_message_text: Vec<String> = raw
        .pointer("/event/message/fragments")
        .context("Could not find fragments in message")?
        .as_array()
        .context("Could not convert message fragments to array")?
        .iter()
        .filter_map(|fragment| {
            fragment
                .get("text")
                .and_then(|text| text.as_str().map(ToString::to_string))
        })
        .collect();

    let message_text: Vec<String> = message
        .event()
        .context("Could not find message text deserialized event")?
        .fragments()
        .context("Could not get fragments from event")?
        .iter()
        .map(ReceivedTwitchEventMessageFragment::text)
        .collect();

    Ok((raw_message_text, message_text))
}

#[test]
fn test_deserialize_message_text_fragment() -> Result<()> {
    let (raw, message) = load_data::<ReceivedTwitchMessagePayload>(MESSAGE_TEXT_FRAGMENT)?;

    let (raw_message_text, message_text) = build_message_text_vec(&raw, &message)?;

    assert_eq!(raw_message_text, message_text);

    Ok(())
}

#[test]
fn test_deserialize_message_text_emote_fragments() -> Result<()> {
    let (raw, message) = load_data::<ReceivedTwitchMessagePayload>(MESSAGE_TEXT_EMOTE_FRAGMENTS)?;

    let (raw_message_text, message_text) = build_message_text_vec(&raw, &message)?;

    assert_eq!(raw_message_text, message_text);

    Ok(())
}

#[test]
fn test_deserialize_message_text_mention_fragments() -> Result<()> {
    let (raw, message) = load_data::<ReceivedTwitchMessagePayload>(MESSAGE_TEXT_MENTION_FRAGMENTS)?;

    let (raw_message_text, message_text) = build_message_text_vec(&raw, &message)?;

    assert_eq!(raw_message_text, message_text);

    Ok(())
}
