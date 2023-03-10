use std::time::Duration;

use irc::{
    client::{prelude::Config, Client, ClientStream},
    error::Error::{self, PingTimeout},
};
use tokio::{sync::mpsc::Sender, time::sleep};

use crate::handlers::{
    config::CompleteConfig,
    data::{DataBuilder, MessageData},
};

/// Initialize the config and send it to the client to connect to an IRC channel.
pub async fn create_client_stream(config: CompleteConfig) -> (Client, ClientStream) {
    let irc_config = Config {
        nickname: Some(config.twitch.username.clone()),
        server: Some(config.twitch.server.clone()),
        channels: vec![format!("#{}", config.twitch.channel)],
        password: config.twitch.token.clone(),
        port: Some(6667),
        use_tls: Some(false),
        ping_timeout: Some(10),
        ping_time: Some(10),
        ..Default::default()
    };

    let mut client = Client::from_config(irc_config.clone()).await.unwrap();

    client.identify().unwrap();

    let stream = client.stream().unwrap();

    (client, stream)
}

/// If an error of any kind occurs, attempt to reconnect to the IRC channel.
pub async fn client_stream_reconnect(
    err: Error,
    tx: Sender<MessageData>,
    data_builder: DataBuilder<'_>,
    client: &mut Client,
    stream: &mut ClientStream,
    config: &CompleteConfig,
) {
    match err {
        PingTimeout => {
            tx.send(
                data_builder
                    .system("Attempting to reconnect due to Twitch ping timeout.".to_string()),
            )
            .await
            .unwrap();
        }
        _ => {
            tx.send(data_builder.system(
                format!("Attempting to reconnect due to fatal error: {err:?}").to_string(),
            ))
            .await
            .unwrap();
        }
    }

    (*client, *stream) = create_client_stream(config.clone()).await;

    sleep(Duration::from_millis(1000)).await;
}
