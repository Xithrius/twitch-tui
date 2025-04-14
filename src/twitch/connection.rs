use color_eyre::{Result, eyre::ContextCompat};
use reqwest::Client;

use super::{
    messages::{ReceivedTwitchSubscription, TwitchSubscriptionResponse},
    oauth::ClientId,
};

// https://dev.twitch.tv/docs/api/reference/#create-eventsub-subscription
// No need to delete a subscription if session ends, since they're disabled automatically
// https://dev.twitch.tv/docs/eventsub/handling-websocket-events/#which-events-is-my-websocket-subscribed-to
pub async fn subscribe_to_channel(
    client: &Client,
    client_id: &ClientId,
    session_id: Option<String>,
    channel_id: String,
) -> Result<TwitchSubscriptionResponse> {
    let session_id = session_id.context("Session ID is empty")?;

    let url = "https://api.twitch.tv/helix/eventsub/subscriptions";

    // TODO: Handle different subscription types
    // https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/#subscription-types
    let subscription = ReceivedTwitchSubscription::new(
        "channel.chat.message".to_string(),
        channel_id,
        client_id.user_id.clone(),
        session_id,
    );

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&subscription)
        .send()
        .await?;

    // Example of a bad response:
    // Object {
    //     "error": String("Bad Request"),
    //     "message": String("missing or unparseable subscription condition"),
    //     "status": Number(400),
    // }

    let response_data: TwitchSubscriptionResponse = response.json().await?;

    Ok(response_data)
}
