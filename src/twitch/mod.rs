mod badges;
mod connection;
pub mod oauth;

use std::{borrow::Borrow, collections::HashMap};

use futures::StreamExt;
use irc::{
    client::prelude::Capability,
    proto::{Command, Message},
};
use log::{debug, info};
use tokio::sync::{broadcast::Receiver, mpsc::Sender};

use crate::{
    handlers::{
        config::CompleteConfig,
        data::{DataBuilder, TwitchToTerminalAction},
        state::State,
    },
    twitch::{
        badges::retrieve_user_badges,
        connection::{client_stream_reconnect, wait_client_stream},
    },
};

#[derive(Debug, Clone)]
pub enum TwitchAction {
    Privmsg(String),
    Join(String),
    ClearMessages,
}

pub async fn twitch_irc(
    mut config: CompleteConfig,
    tx: Sender<TwitchToTerminalAction>,
    mut rx: Receiver<TwitchAction>,
) {
    info!("Spawned Twitch IRC thread.");

    // If the dashboard is the start state, wait until the user has selected
    // a channel before connecting to Twitch's IRC.
    if config.borrow().terminal.first_state == State::Dashboard {
        debug!("Waiting for user to select channel from debug screen");

        loop {
            if let Ok(TwitchAction::Join(channel)) = rx.recv().await {
                config.twitch.channel = channel;

                debug!("User has selected channel from start screen");
                break;
            }
        }
    }

    let data_builder = DataBuilder::new(&config.frontend.datetime_format);
    let mut room_state_startup = false;

    let (mut client, mut stream) =
        wait_client_stream(tx.clone(), data_builder, config.clone()).await;

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

    let mut connected = true;

    loop {
        tokio::select! {
            biased;

            Ok(action) = rx.recv() => {
                let current_channel = format!("#{}", config.twitch.channel);

                match action {
                    TwitchAction::Privmsg(message) => {
                        debug!("Sending message to Twitch: {}", message);

                        client
                            .send_privmsg(current_channel, message)
                            .unwrap();
                    }
                    TwitchAction::Join(channel) => {
                        debug!("Switching to channel {channel}");

                        let channel_list = format!("#{channel}");

                        // Leave previous channel
                        if let Err(err) = sender.send_part(current_channel) {
                            tx.send(data_builder.twitch(err.to_string())).await.unwrap();
                        }

                        // Join specified channel
                        if let Err(err) = sender.send_join(&channel_list) {
                            tx.send(data_builder.twitch(err.to_string())).await.unwrap();
                        }

                        // Set old channel to new channel
                        config.twitch.channel = channel;
                    }
                    TwitchAction::ClearMessages => {
                        client.send(Command::Raw("CLEARCHAT".to_string(), vec![])).unwrap();
                    }
                }
            }
            Some(message) = stream.next() => {
                match message {
                    Ok(message) => {
                        if !connected {
                            tx.send(data_builder.system("Reconnect succcessful.".to_string())).await.unwrap();
                            connected = true;
                        }

                        if let Some(b) = handle_message_command(message, tx.clone(), data_builder, config.frontend.badges, room_state_startup).await {
                            room_state_startup = b;
                        }
                    }
                    Err(err) => {
                        connected = false;

                        debug!("Twitch connection error encountered: {}, attempting to reconnect.", err);

                        (client, stream) = client_stream_reconnect(err, tx.clone(), data_builder, &config).await;

                    }
                }
            }
            else => {}
        };
    }
}

async fn handle_message_command(
    message: Message,
    tx: Sender<TwitchToTerminalAction>,
    data_builder: DataBuilder<'_>,
    badges: bool,
    room_state_startup: bool,
) -> Option<bool> {
    let mut tags: HashMap<&str, &str> = HashMap::new();

    if let Some(ref ref_tags) = message.tags {
        for tag in ref_tags {
            if let Some(ref tag_value) = tag.1 {
                tags.insert(&tag.0, tag_value);
            }
        }
    }

    match message.command {
        Command::PRIVMSG(ref _target, ref msg) => {
            // lowercase username from message
            let mut name = message.source_nickname().unwrap().to_string();

            if badges {
                retrieve_user_badges(&mut name, &message);
            }

            // An attempt to remove null bytes from the message.
            let cleaned_message = msg.trim_matches(char::from(0));

            let id = tags.get("id").map(|&s| s.to_string());

            tx.send(DataBuilder::user(
                name.to_string(),
                cleaned_message.to_string(),
                id,
            ))
            .await
            .unwrap();

            debug!("Message received from twitch: {} - {}", name, msg);
        }
        Command::NOTICE(ref _target, ref msg) => {
            tx.send(data_builder.twitch(msg.to_string())).await.unwrap();
        }
        Command::JOIN(ref channel, _, _) => {
            tx.send(data_builder.twitch(format!("Joined {}", *channel)))
                .await
                .unwrap();
        }
        Command::Raw(ref cmd, ref _items) => {
            match cmd.as_ref() {
                // https://dev.twitch.tv/docs/irc/tags/#roomstate-tags
                "ROOMSTATE" => {
                    // Only display roomstate on startup, since twitch
                    // sends a NOTICE whenever roomstate changes.
                    if !room_state_startup {
                        handle_roomstate(&tx, &tags).await;
                    }

                    return Some(true);
                }
                // https://dev.twitch.tv/docs/irc/tags/#usernotice-tags
                "USERNOTICE" => {
                    if let Some(value) = tags.get("system-msg") {
                        tx.send(data_builder.twitch((*value).to_string()))
                            .await
                            .unwrap();
                    }
                }
                // https://dev.twitch.tv/docs/irc/tags/#clearchat-tags
                "CLEARCHAT" => {
                    tx.send(TwitchToTerminalAction::ClearChat).await.unwrap();
                    tx.send(data_builder.twitch("Chat cleared by a moderator.".to_string()))
                        .await
                        .unwrap();
                }
                // https://dev.twitch.tv/docs/irc/tags/#clearmsg-tags
                "CLEARMSG" => {
                    if let Some(id) = tags.get("target-msg-id") {
                        tx.send(TwitchToTerminalAction::DeleteMessage((*id).to_string()))
                            .await
                            .unwrap();
                    }
                }
                _ => (),
            }
        }
        _ => (),
    }

    None
}

pub async fn handle_roomstate(tx: &Sender<TwitchToTerminalAction>, tags: &HashMap<&str, &str>) {
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

    if room_state.is_empty() {
        return;
    }

    let id = tags.get("target-msg-id").map(|&s| s.to_string());

    tx.send(DataBuilder::user(String::from("Info"), room_state, id))
        .await
        .unwrap();
}
