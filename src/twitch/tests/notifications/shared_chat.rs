use color_eyre::Result;

use crate::twitch::{
    models::ReceivedTwitchMessagePayload,
    tests::{notifications::SHARED_CHAT, utils::load_data},
};

#[test]
fn test_deserialize_shared_chat_source_badges() -> Result<()> {
    let (_, message) = load_data::<ReceivedTwitchMessagePayload>(SHARED_CHAT)?;

    assert!(message.event().is_some());

    Ok(())
}
