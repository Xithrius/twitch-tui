use std::cmp::min;
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
async fn create_client_stream(config: CompleteConfig) -> Result<(Client, ClientStream), Error> {
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

    let mut client = Client::from_config(irc_config.clone()).await?;

    client.identify()?;

    let stream = client.stream()?;

    Ok((client, stream))
}

pub async fn wait_client_stream(
    tx: Sender<MessageData>,
    data_builder: DataBuilder<'_>,
    config: CompleteConfig,
) -> (Client, ClientStream) {
    let mut timeout = 1;

    loop {
        match create_client_stream(config.clone()).await {
            Ok(v) => return v,
            Err(err) => match err {
                Error::Io(io) => tx
                    .send(data_builder.system(format!("Unable to connect: {io}")))
                    .await
                    .unwrap(),
                _ => {
                    tx.send(data_builder.system(format!("Fatal error: {err:?}").to_string()))
                        .await
                        .unwrap();
                }
            },
        };

        sleep(Duration::from_secs(timeout)).await;

        timeout = min(timeout * 2, 30);
    }
}

/// If an error of any kind occurs, attempt to reconnect to the IRC channel.
pub async fn client_stream_reconnect(
    err: Error,
    tx: Sender<MessageData>,
    data_builder: DataBuilder<'_>,
    config: &CompleteConfig,
) -> (Client, ClientStream) {
    match err {
        PingTimeout => {
            tx.send(data_builder.system("Ping to Twitch has timed out.".to_string()))
                .await
                .unwrap();
        }
        _ => {
            tx.send(data_builder.system(format!("Fatal error: {err:?}").to_string()))
                .await
                .unwrap();
        }
    }

    sleep(Duration::from_millis(1000)).await;

    tx.send(data_builder.system("Attempting reconnect...".to_string()))
        .await
        .unwrap();

    let (client, stream) = wait_client_stream(tx, data_builder, config.clone()).await;

    (client, stream)
}
