use color_eyre::{Result, eyre::ContextCompat};

use crate::twitch::{
    models::ReceivedTwitchMessagePayload,
    tests::{notifications::REPLY, utils::load_data},
};

#[test]
fn test_deserialize_reply() -> Result<()> {
    let (raw, message) = load_data::<ReceivedTwitchMessagePayload>(&REPLY)?;

    Ok(())
}
