use color_eyre::{Result, eyre::ContextCompat};

use crate::twitch::{
    models::ReceivedTwitchMessagePayload,
    tests::{
        notifications::{USER_BAN, USER_TIMEOUT},
        utils::load_data,
    },
};

#[test]
fn test_deserialize_user_ban() -> Result<()> {
    let (_, message) = load_data::<ReceivedTwitchMessagePayload>(&USER_BAN)?;

    let event = message.event().context("Could not find message event")?;
    let timeout_duration = event.timeout_duration();

    assert_eq!(timeout_duration, None);

    Ok(())
}

#[test]
fn test_deserialize_user_timeout() -> Result<()> {
    let (_, message) = load_data::<ReceivedTwitchMessagePayload>(&USER_TIMEOUT)?;

    let event = message.event().context("Could not find message event")?;
    let timeout_duration = event
        .timeout_duration()
        .context("Timeout duration should be some")?;

    assert_eq!(timeout_duration, 60);

    Ok(())
}
