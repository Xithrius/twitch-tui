use futures::StreamExt;
use irc::{
    client::{data, prelude::*, Client},
    proto::{message::Tag, Command},
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

    // Request commands capabilities
    if client
        .send_cap_req(&[
            Capability::Custom("twitch.tv/commands"),
            Capability::Custom("twitch.tv/tags"),
        ])
        .is_err()
    {
        tx.send(
            data_builder.system(
                "Unable to request commands/tags capability, certain features may be affected."
                    .to_string(),
            ),
        )
        .await
        .unwrap();
    }

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
                        tx.send(data_builder.twitch(msg.to_string()))
                        .await
                        .unwrap();
                    }
                    Command::Raw(ref cmd, ref _items) => {
                        match cmd.as_ref() {
                            "ROOMSTATE" => {
                                if let Some(tags) = message.tags {
                                    handle_roomstate(&tx, data_builder, tags).await;
                                }
                            }
                            "USERNOTICE" => {
                                if let Some(Some(value)) = message.tags.iter().flatten().find(|t| t.0 == "system-msg").map(|t| t.1.as_ref()) {
                                    tx.send(data_builder.twitch(value.to_string()))
                                    .await
                                    .unwrap();
                                }
                            }
                            _ => ()
                        }
                    }
                    _ => ()
                }
            }
        };
    }
}

pub async fn handle_roomstate(tx: &Sender<Data>, builder: DataBuilder<'_>, tags: Vec<Tag>) {
    let mut room_state = String::new();
    for tag in tags {
        let value = tag.1.as_deref().unwrap_or("0");
        match tag.0.as_ref() {
            "emote-only" if value == "1" => {
                room_state.push_str("The channel is emote-only.\n");
            }
            "followers-only" if value != "-1" => {
                room_state.push_str("The channel is followers-only.\n");
            }
            "subs-only" if value == "1" => {
                room_state.push_str("The channel is subscribers-only.\n");
            }
            "slow" if value != "0" => {
                room_state.push_str("The channel has a ");
                room_state.push_str(value);
                room_state.push_str("s slowmode.\n");
            }
            _ => (),
        }
    }
    // Trim last newline
    room_state.pop();
    if !room_state.is_empty() {
        tx.send(builder.user(String::from("Info"), room_state))
            .await
            .unwrap();
    }
}
