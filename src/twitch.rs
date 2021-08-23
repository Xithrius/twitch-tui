use std::sync::mpsc::Sender;

use dotenv;
use futures::prelude::*;
use irc::client::prelude::*;
use tokio;

#[tokio::main]
pub async fn twitch_irc(tx: &Sender<Vec<String>>) {
    let config = Config {
        nickname: Some(dotenv::var("NICKNAME").unwrap().to_owned()),
        server: Some(dotenv::var("SERVER").unwrap().to_owned()),
        channels: vec![format!("#{}", dotenv::var("CHANNEL").unwrap().to_owned())],
        password: Some(dotenv::var("TOKEN").unwrap().to_owned()),
        port: Some(6667),
        use_tls: Some(false),
        ..Default::default()
    };

    let mut client = Client::from_config(config).await.unwrap();
    client.identify().unwrap();
    let mut stream = client.stream().unwrap();

    while let Ok(Some(message)) = stream.next().await.transpose() {
        if let Command::PRIVMSG(ref _target, ref msg) = message.command {
            let user = match message.source_nickname() {
                Some(username) => username.to_string(),
                None => "Undefined username".to_string(),
            };
            tx.send(vec![user, msg.to_string()]).unwrap();
        }
    }
}
