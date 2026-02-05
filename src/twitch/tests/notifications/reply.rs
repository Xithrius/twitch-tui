use color_eyre::{Result, eyre::ContextCompat};

use crate::twitch::{
    models::ReceivedTwitchMessagePayload,
    tests::{notifications::REPLY, utils::load_data},
};

#[test]
fn test_deserialize_reply() -> Result<()> {
    let (raw, message) = load_data::<ReceivedTwitchMessagePayload>(REPLY)?;

    let raw_parent_message_body = raw
        .pointer("/event/reply/parent_message_body")
        .context("Could not find raw parent message body")?
        .as_str()
        .context("Could not convert raw parent message body to string")?
        .to_string();

    let parent_message_body = message
        .event()
        .context("Could not find reply deserialized event")?
        .reply()
        .context("Could not get reply out of event")?
        .parent_message_body()
        .clone();

    assert_eq!(raw_parent_message_body, parent_message_body);

    Ok(())
}
