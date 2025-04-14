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

// async fn create_client_stream(config: CoreConfig) -> Result<(Client, ClientStream), Error> {
//     let irc_config = Config {
//         nickname: Some(config.twitch.username.clone()),
//         server: Some(config.twitch.server.clone()),
//         channels: vec![format!("#{}", config.twitch.channel)],
//         password: config.twitch.token.clone(),
//         port: Some(6697),
//         use_tls: Some(true),
//         ping_timeout: Some(10),
//         ping_time: Some(10),
//         ..Default::default()
//     };

//     let mut client = Client::from_config(irc_config.clone()).await?;

//     client.identify()?;

//     let stream = client.stream()?;

//     Ok((client, stream))
// }

// pub async fn wait_client_stream(
//     tx: Sender<TwitchToTerminalAction>,
//     data_builder: DataBuilder<'_>,
//     config: CoreConfig,
// ) -> (Client, ClientStream) {
//     let mut timeout = 1;

//     loop {
//         match create_client_stream(config.clone()).await {
//             Ok(v) => return v,
//             Err(err) => match err {
//                 Error::Io(io) => tx
//                     .send(data_builder.system(format!("Unable to connect: {io}")))
//                     .await
//                     .unwrap(),
//                 _ => {
//                     tx.send(data_builder.system(format!("Fatal error: {err:?}").to_string()))
//                         .await
//                         .unwrap();
//                 }
//             },
//         }

//         sleep(Duration::from_secs(timeout)).await;

//         timeout = min(timeout * 2, 30);
//     }
// }

// pub async fn client_stream_reconnect(
//     err: Error,
//     tx: Sender<TwitchToTerminalAction>,
//     data_builder: DataBuilder<'_>,
//     config: &CoreConfig,
// ) -> (Client, ClientStream) {
//     match err {
//         PingTimeout => {
//             tx.send(data_builder.system("Ping to Twitch has timed out.".to_string()))
//                 .await
//                 .unwrap();
//         }
//         _ => {
//             tx.send(data_builder.system(format!("Fatal error: {err:?}").to_string()))
//                 .await
//                 .unwrap();
//         }
//     }

//     sleep(Duration::from_millis(1000)).await;

//     tx.send(data_builder.system("Attempting reconnect...".to_string()))
//         .await
//         .unwrap();

//     let (client, stream) = wait_client_stream(tx, data_builder, config.clone()).await;

//     (client, stream)
// }
