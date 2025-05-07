use std::string::ToString;

use color_eyre::{Result, eyre::ContextCompat};

use crate::twitch::{
    models::ReceivedTwitchMessagePayload,
    tests::{
        notifications::{EMOTE, MANY_EMOTES},
        utils::load_data,
    },
};

#[test]
fn test_deserialize_emote() -> Result<()> {
    let (raw, message) = load_data::<ReceivedTwitchMessagePayload>(&EMOTE)?;

    let raw_emote_ids: Vec<String> = raw
        .pointer("/event/message/fragments")
        .context("Could not find fragments in message")?
        .as_array()
        .context("Could not convert message fragments to array")?
        .iter()
        .filter_map(|fragment| {
            fragment.get("emote").and_then(|emote| {
                emote
                    .get("id")
                    .and_then(|emote_id| emote_id.as_str().map(ToString::to_string))
            })
        })
        .collect();

    let emote_ids: Vec<String> = message
        .event()
        .context("Could not find emotes deserialized event")?
        .emote_fragments()
        .context("Could not find emotes vector")?
        .iter()
        .filter_map(|fragment| fragment.emote().and_then(|emote| emote.emote_id()))
        .collect();

    assert_eq!(raw_emote_ids, emote_ids);

    Ok(())
}

#[test]
fn test_deserialize_many_emotes() -> Result<()> {
    let (raw, message) = load_data::<ReceivedTwitchMessagePayload>(&MANY_EMOTES)?;

    let raw_emote_ids: Vec<String> = raw
        .pointer("/event/message/fragments")
        .context("Could not find fragments in message")?
        .as_array()
        .context("Could not convert message fragments to array")?
        .iter()
        .filter_map(|fragment| {
            fragment.get("emote").and_then(|emote| {
                emote
                    .get("id")
                    .and_then(|emote_id| emote_id.as_str().map(ToString::to_string))
            })
        })
        .collect();

    let emote_ids: Vec<String> = message
        .event()
        .context("Could not find emotes deserialized event")?
        .emote_fragments()
        .context("Could not find emotes vector")?
        .iter()
        .filter_map(|fragment| fragment.emote().and_then(|emote| emote.emote_id()))
        .collect();

    assert_eq!(raw_emote_ids, emote_ids);

    Ok(())
}
