use color_eyre::Result;
use futures::StreamExt;
use tokio::sync::{broadcast::Receiver, mpsc::Sender};
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::error;

use crate::{
    config::SharedCoreConfig,
    handlers::data::{DataBuilder, TwitchToTerminalAction},
    twitch::{
        actions::TwitchAction,
        context::TwitchWebsocketContext,
        handlers::{
            incoming_message::handle_incoming_message, message_commands::handle_command_message,
            send_message::handle_send_message, welcome_message::handle_channel_join,
        },
        models::ReceivedTwitchMessage,
    },
};

type WebsocketStream = futures::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
>;

pub async fn websocket_event_loop(
    config: SharedCoreConfig,
    tx: Sender<TwitchToTerminalAction>,
    mut rx: Receiver<TwitchAction>,
    mut stream: WebsocketStream,
    emotes_enabled: bool,
    mut context: TwitchWebsocketContext,
) -> Result<()> {
    loop {
        tokio::select! {
            biased;

            Ok(action) = rx.recv() => {
                match action {
                    TwitchAction::Message(message) => {
                        if let Some(command) = message.strip_prefix('/') {
                            if let Err(err) = handle_command_message(&context, &tx, command).await {
                                error!("Failed to handle Twitch message command from terminal: {err}");
                                tx.send(DataBuilder::twitch(format!("Failed to handle Twitch message command from terminal: {err}"))).await?;
                            }
                        }
                        else if let Err(err) = handle_send_message(&context, message).await {
                            error!("Failed to send Twitch message from terminal: {err}");
                        }
                    },
                    TwitchAction::JoinChannel(channel_name) => {
                        if let Err(err) = handle_channel_join(&mut context, &tx, channel_name, false).await {
                            error!("Joining channel failed: {err}");
                        }
                    },
                }
            }
            Some(message) = stream.next() => {
                match message {
                    Ok(message) => {
                        let Message::Text(message_text) = message else {
                            continue;
                        };

                        let received_message = match serde_json::from_str::<ReceivedTwitchMessage>(&message_text) {
                            Ok(received_message) => received_message,
                            Err(err) => {
                                error!("Error when deserializing received message: {err}");
                                continue;
                            }
                        };

                        if let Err(err) = handle_incoming_message(
                            config.clone(),
                            &context,
                            &tx,
                            received_message,
                            emotes_enabled,
                        ).await {
                            error!("Error when handling incoming message: {err}");
                        }
                    }
                    Err(err) => {
                        error!("Twitch connection error encountered: {err}, attempting to reconnect.");
                    }
                }
            }
            else => {}
        };
    }
}
