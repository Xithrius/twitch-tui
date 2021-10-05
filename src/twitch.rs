use futures::StreamExt;
use irc::{
    client::{data, Client},
    proto::Command,
};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::handlers::{
    config::CompleteConfig,
    data::{Data, DataBuilder},
};

pub async fn twitch_irc(config: &CompleteConfig, tx: Sender<Data>, mut rx: Receiver<String>) {
    let irc_config = data::Config {
        nickname: Some(config.twitch.username.to_owned()),
        server: Some(config.twitch.server.to_owned()),
        channels: vec![format!("#{}", config.twitch.channel)],
        password: Some(config.twitch.token.to_owned()),
        port: Some(6667),
        use_tls: Some(false),
        ..Default::default()
    };

    let mut client = Client::from_config(irc_config.clone()).await.unwrap();
    client.identify().unwrap();
    let mut stream = client.stream().unwrap();
    let data_builder = DataBuilder::new(&config.frontend.date_format);

    loop {
        tokio::select! {
            biased;

            Some(message) = rx.recv() => {
                client
                .send_privmsg(format!("#{}", config.twitch.channel), message)
                .unwrap();
            }
            Some(_message) = stream.next() => {
                let message = _message.unwrap();
                match message.command {
                    Command::PRIVMSG(ref _target, ref msg) => {
                        let user = match message.source_nickname() {
                            Some(username) => username.to_string(),
                            None => "Undefined username".to_string(),
                        };
                        tx.send(data_builder.user(user, msg.to_string()))
                        .await
                        .unwrap();
                    }
                    Command::NOTICE(ref _target, ref msg) => {
                        tx.send(data_builder.twitch(format!("NOTICE: {}", msg)))
                        .await
                        .unwrap();
                    }
                    _ => ()
                }
            }
        };
    }
}
