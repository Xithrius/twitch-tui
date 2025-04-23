use color_eyre::{Result, eyre::ContextCompat};

use crate::twitch::{
    models::ReceivedTwitchMessagePayload,
    tests::{
        notifications::{CHEER, INVALID_CHEER},
        utils::load_data,
    },
};

#[test]
fn test_deserialize_cheer() -> Result<()> {
    let (raw, message) = load_data::<ReceivedTwitchMessagePayload>(&CHEER)?;

    let raw_bits = raw
        .pointer("/event/cheer/bits")
        .context("Could not find cheer bits")?
        .as_u64()
        .context("Cheer could not be converted to u64")?;

    let bits = message
        .event()
        .context("Could not find cheer deserialized event")?
        .cheer()
        .context("Could not find cheer in event")?
        .bits();

    assert_eq!(raw_bits, bits);

    Ok(())
}

#[test]
#[should_panic(expected = "Invalid cheer field")]
fn test_deserialize_invalid_cheer() {
    load_data::<ReceivedTwitchMessagePayload>(&INVALID_CHEER)
        .expect("Invalid cheer field");
}
