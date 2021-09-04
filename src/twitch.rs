use std::sync::mpsc::Sender;

use chrono::offset::Local;
use futures::prelude::*;
use irc::client::prelude::*;

use crate::handlers::{config::CompleteConfig, data::Data};

#[tokio::main]
pub async fn twitch_irc(config: &CompleteConfig, tx: &Sender<Data>) {
    let irc_config = Config {
        nickname: Some(config.twitch.username.to_owned()),
        server: Some(config.twitch.server.to_owned()),
        channels: vec![format!("#{}", config.twitch.channel)],
        password: Some(dotenv::var("TOKEN").unwrap().to_owned()),
        port: Some(6667),
        use_tls: Some(false),
        ..Default::default()
    };

    let mut client = Client::from_config(irc_config).await.unwrap();
    client.identify().unwrap();
    let mut stream = client.stream().unwrap();

    while let Ok(Some(message)) = stream.next().await.transpose() {
        if let Command::PRIVMSG(ref _target, ref msg) = message.command {
            let user = match message.source_nickname() {
                Some(username) => username.to_string(),
                None => "Undefined username".to_string(),
            };
            if let Err(e) = tx.send(Data::new(
                Local::now()
                    .format(config.frontend.date_format.as_str())
                    .to_string(),
                user,
                msg.to_string(),
                false,
            )) {
                println!("{}", e);
            }
        }
    }
}
