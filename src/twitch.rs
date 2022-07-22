use std::{collections::HashMap, time::Duration};

use futures::StreamExt;
use irc::{
    client::{
        prelude::{Capability, Config},
        Client, ClientStream,
    },
    error::Error::PingTimeout,
    proto::Command,
};
use log::debug;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::sleep,
};

use crate::handlers::{
    config::CompleteConfig,
    data::{Data, DataBuilder},
};

const VIP_BADGE: char = '\u{1F48E}';
const MODERATOR_BADGE: char = '\u{1F528}';
const SUBSCRIBER_BADGE: char = '\u{2B50}';
const PRIME_GAMING_BADGE: char = '\u{1F451}';

#[derive(Debug)]
pub enum Action {
    Privmsg(String),
    Join(String),
}

async fn create_client_stream(config: CompleteConfig) -> (Client, ClientStream) {
    let irc_config = Config {
        nickname: Some(config.twitch.username.to_owned()),
        server: Some(config.twitch.server.to_owned()),
        channels: vec![format!("#{}", config.twitch.channel)],
        password: Some(config.twitch.token.to_owned()),
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

pub async fn twitch_irc(mut config: CompleteConfig, tx: Sender<Data>, mut rx: Receiver<Action>) {
    debug!("Spawned Twitch IRC thread.");

    let data_builder = DataBuilder::new(&config.frontend.date_format);
    let mut room_state_startup = false;

    let (mut client, mut stream) = create_client_stream(config.clone()).await;

    let sender = client.sender();

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

            Some(action) = rx.recv() => {
                let current_channel = format!("#{}", config.twitch.channel);

                match action {
                    Action::Privmsg(message) => {
                        client
                            .send_privmsg(current_channel, message)
                            .unwrap();
                    }
                    Action::Join(channel) => {
                        let channel_list = format!("#{}", channel);

                        // Leave previous channel
                        if let Err(err) = sender.send_part(current_channel) {
                            tx.send(data_builder.twitch(err.to_string())).await.unwrap()
                        } else {
                            tx.send(data_builder.twitch(format!("Joined {}", channel_list))).await.unwrap();
                        }

                        // Join specified channel
                        if let Err(err) = sender.send_join(&channel_list) {
                            tx.send(data_builder.twitch(err.to_string())).await.unwrap()
                        }

                        // Set old channel to new channel
                        config.twitch.channel = channel;
                    }
                }
            }
            Some(_message) = stream.next() => {
                match _message {
                    Ok(message) => {
                        let mut tags: HashMap<&str, &str> = HashMap::new();
                        if let Some(ref _tags) = message.tags {
                            for tag in _tags {
                                if let Some(ref tag_value) = tag.1 {
                                    tags.insert(&tag.0, tag_value);
                                }
                            }
                        }

                        match message.command {
                            Command::PRIVMSG(ref _target, ref msg) => {
                                // lowercase username from message
                                let mut name = match message.source_nickname() {
                                    Some(username) => username.to_string(),
                                    None => "Undefined username".to_string(),
                                };
                                if config.frontend.badges {
                                    let mut badges = String::new();
                                    if let Some(ref tags) = message.tags {
                                        let mut vip_badge = None;
                                        let mut moderator_badge = None;
                                        let mut subscriber_badge = None;
                                        let mut prime_badge = None;
                                        let mut display_name = None;
                                        for tag in tags {
                                            if tag.0 == *"display-name" {
                                                if let Some(ref value) = tag.1 {
                                                    display_name = Some(value.to_string());
                                                }
                                            }
                                            if tag.0 == *"badges" {
                                                if let Some(ref value) = tag.1 {
                                                    if !value.is_empty() && value.contains("vip") {
                                                        vip_badge = Some(VIP_BADGE);
                                                    }
                                                    if !value.is_empty() && value.contains("moderator") {
                                                        moderator_badge = Some(MODERATOR_BADGE);
                                                    }
                                                    if !value.is_empty() && value.contains("subscriber") {
                                                        subscriber_badge = Some(SUBSCRIBER_BADGE);
                                                    }
                                                    if !value.is_empty() && value.contains("premium") {
                                                        prime_badge = Some(PRIME_GAMING_BADGE);
                                                    }
                                                }
                                            }
                                        }
                                        if let Some(display_name) = display_name {
                                            name = display_name;
                                        }
                                        if let Some(badge) = vip_badge {
                                            badges.push(badge);
                                        }
                                        if let Some(badge) = moderator_badge {
                                            badges.push(badge);
                                        }
                                        if let Some(badge) = subscriber_badge {
                                            badges.push(badge);
                                        }
                                        if let Some(badge) = prime_badge {
                                            badges.push(badge);
                                        }
                                        if !badges.is_empty() {
                                            name = badges.clone() + &name;
                                        }
                                    }
                                }
                                tx.send(data_builder.user(name, msg.to_string()))
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
                                        // Only display roomstate on startup, since twitch
                                        // sends a NOTICE whenever roomstate changes.
                                        if !room_state_startup {
                                            handle_roomstate(&tx, data_builder, &tags).await;
                                        }
                                        room_state_startup = true;
                                    }
                                    "USERNOTICE" => {
                                        if let Some(value) = tags.get("system-msg") {
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
                    Err(err) => {
                        match err {
                            PingTimeout => {
                                tx.send(data_builder.system("Attempting to reconnect due to Twitch ping timeout.".to_string())).await.unwrap();
                            }
                            _ => {
                                tx.send(data_builder.system(format!("Attempting to reconnect due to fatal error: {:?}", err).to_string())).await.unwrap();
                            }
                        }

                        (client, stream) = create_client_stream(config.clone()).await;

                        sleep(Duration::from_millis(1000)).await;
                    }
                }
            }
            else => {}
        };
    }
}

pub async fn handle_roomstate(
    tx: &Sender<Data>,
    builder: DataBuilder<'_>,
    tags: &HashMap<&str, &str>,
) {
    let mut room_state = String::new();
    for (name, value) in tags.iter() {
        match *name {
            "emote-only" if *value == "1" => {
                room_state.push_str("The channel is emote-only.\n");
            }
            "followers-only" if *value != "-1" => {
                room_state.push_str("The channel is followers-only.\n");
            }
            "subs-only" if *value == "1" => {
                room_state.push_str("The channel is subscribers-only.\n");
            }
            "slow" if *value != "0" => {
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
