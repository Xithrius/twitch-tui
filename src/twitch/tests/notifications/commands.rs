use color_eyre::{Result, eyre::ContextCompat};

use crate::{
    twitch::{
        models::ReceivedTwitchMessagePayload,
        tests::{
            notifications::{CLEAR_COMMAND, ME_COMMAND},
            utils::load_data,
        },
    },
    utils::text::parse_message_action,
};

#[test]
fn test_deserialize_clear_command() -> Result<()> {
    let (raw, message) = load_data::<ReceivedTwitchMessagePayload>(CLEAR_COMMAND)?;

    let raw_subscription_type = raw
        .pointer("/subscription/type")
        .context("Could not get clear command subscription type")?
        .as_str()
        .context("Could not convert clear command subscription type to string")?
        .to_string();

    let subscription_type = message
        .subscription()
        .context("Could not get message event")?
        .subscription_type()
        .context("Could not get subscription type from event subscription")?
        .to_string();

    assert_eq!(raw_subscription_type, subscription_type);

    Ok(())
}

#[test]
fn test_deserialize_and_parse_me_command() -> Result<()> {
    let (raw, message) = load_data::<ReceivedTwitchMessagePayload>(ME_COMMAND)?;

    let raw_message_text = raw
        .pointer("/event/message/text")
        .context("Could not get event message text")?
        .as_str()
        .context("Could not convert event message text to string")?
        .to_string();

    let message_text = message
        .event()
        .context("Could not find message deserialized event")?
        .message_text()
        .context("Could not find message text in event message")?;

    assert_eq!(raw_message_text, message_text);

    let (msg, highlight) = parse_message_action(&message_text);

    assert_eq!(msg, "asdf");
    assert!(highlight);

    Ok(())
}
