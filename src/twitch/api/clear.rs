use color_eyre::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::TWITCH_API_BASE_URL;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeleteMessageQuery {
    broadcaster_id: String,
    moderator_id: String,
    message_id: Option<String>,
}

impl DeleteMessageQuery {
    pub const fn new(
        broadcaster_id: String,
        moderator_id: String,
        message_id: Option<String>,
    ) -> Self {
        Self {
            broadcaster_id,
            moderator_id,
            message_id,
        }
    }
}

/// Removes a single chat message or all chat messages from the broadcaster’s chat room.
/// If no message ID is specified in the query, the entire chat room will be cleared.
///
/// <https://dev.twitch.tv/docs/api/reference/#delete-chat-messages>
pub async fn delete_twitch_messages(client: &Client, query: DeleteMessageQuery) -> Result<()> {
    let url = format!("{TWITCH_API_BASE_URL}/moderation/chat");

    let mut delete_message_query = vec![
        ("broadcaster_id", query.broadcaster_id),
        ("moderator_id", query.moderator_id),
    ];
    if let Some(message_id) = query.message_id {
        delete_message_query.push(("message_id", message_id));
    }

    client
        .delete(url)
        .query(&delete_message_query)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
