use color_eyre::{Result, eyre::ContextCompat};

use crate::twitch::{
    models::ReceivedTwitchMessagePayload,
    tests::messages::{CHEER, INVALID_CHEER},
};

#[test]
fn test_deserialize_cheer() -> Result<()> {
    let raw_message: serde_json::Value = serde_json::from_str(&CHEER)?;
    let message = serde_json::from_str::<ReceivedTwitchMessagePayload>(&CHEER)?;

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
#[should_panic(expected = "Invalid cheer field")]
fn test_deserialize_invalid_cheer() {
    serde_json::from_str::<ReceivedTwitchMessagePayload>(&INVALID_CHEER)
        .expect("Invalid cheer field");
}
