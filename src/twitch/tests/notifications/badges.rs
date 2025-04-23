use std::string::ToString;

use color_eyre::{Result, eyre::ContextCompat};

use crate::twitch::{
    models::ReceivedTwitchMessagePayload,
    tests::notifications::{BADGES, INVALID_BADGES, NO_BADGES},
};

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
        .context("Could not find badges deserialized event")?
        .badges()
        .iter()
        .map(|badge| badge.set_id().to_string())
        .collect();

    assert_eq!(raw_badges, badges);

    Ok(())
}

#[test]
fn test_deserialize_no_badges() -> Result<()> {
    let raw_message: serde_json::Value = serde_json::from_str(&NO_BADGES)?;
    let message = serde_json::from_str::<ReceivedTwitchMessagePayload>(&NO_BADGES)?;

    let raw_badges_len = raw_message
        .pointer("/event/badges")
        .context("Could not find badges")?
        .as_array()
        .context("Could not convert badges to array")?
        .len();

    let badges_len = message
        .event()
        .context("Could not find badges deserialized event")?
        .badges()
        .len();

    assert_eq!(raw_badges_len, badges_len);

    Ok(())
}

#[test]
#[should_panic(expected = "Invalid badges field")]
fn test_deserialize_invalid_badges() {
    serde_json::from_str::<ReceivedTwitchMessagePayload>(&INVALID_BADGES)
        .expect("Invalid badges field");
}
