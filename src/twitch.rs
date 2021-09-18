use chrono::offset::Local;
use futures::{FutureExt, StreamExt};
use irc::{
    client::{data::Config, Client},
    proto::Command,
};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::unconstrained,
};

use crate::handlers::{config::CompleteConfig, data::Data};

pub async fn twitch_irc(config: &CompleteConfig, tx: Sender<Data>, mut rx: Receiver<String>) {
    let irc_config = Config {
        nickname: Some(config.twitch.username.to_owned()),
        server: Some(config.twitch.server.to_owned()),
        channels: vec![format!("#{}", config.twitch.channel)],
        password: Some(dotenv::var("TOKEN").unwrap()),
        port: Some(6667),
        use_tls: Some(false),
        ..Default::default()
    };

    let mut client = Client::from_config(irc_config.clone()).await.unwrap();
    client.identify().unwrap();
    let mut stream = client.stream().unwrap();

    loop {
        if let Some(Some(Ok(message))) = unconstrained(stream.next()).now_or_never() {
            if let Command::PRIVMSG(ref _target, ref msg) = message.command {
                let user = match message.source_nickname() {
                    Some(username) => username.to_string(),
                    None => "Undefined username".to_string(),
                };
                tx.send(Data::new(
                    Local::now()
                        .format(config.frontend.date_format.as_str())
                        .to_string(),
                    user,
                    msg.to_string(),
                    false,
                ))
                .await
                .unwrap();
            }
        }

        if let Some(Some(message)) = unconstrained(rx.recv()).now_or_never() {
            client
                .send_privmsg(format!("#{}", config.twitch.channel), message)
                .unwrap();
        }
    }
}
