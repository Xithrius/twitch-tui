use color_eyre::{Result, eyre::bail};
use futures::StreamExt;
use tokio::sync::{broadcast::Receiver, mpsc::Sender};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, info};

use crate::{
    config::SharedCoreConfig,
    handlers::{
        data::{DataBuilder, TwitchToTerminalAction},
        state::State,
    },
    twitch::{
        actions::TwitchAction,
        context::TwitchWebsocketContext,
        handlers::{
            incoming_message::handle_incoming_message,
            message_commands::handle_command_message,
            send_message::handle_send_message,
            welcome_message::{handle_channel_join, handle_welcome_message},
        },
        models::ReceivedTwitchMessage,
    },
};

pub struct TwitchWebsocket {
    // rx: Receiver<TwitchAction>,
}

impl TwitchWebsocket {
    pub fn new(
        config: SharedCoreConfig,
        tx: Sender<TwitchToTerminalAction>,
        rx: Receiver<TwitchAction>,
    ) -> Self {
        let mut actor = TwitchWebsocketThread::new(config, tx, rx);
        tokio::task::spawn(async move { actor.run().await });

        Self {}
    }

    // pub fn next(&self) -> Result<Event> {
    //     Ok(self.rx.recv()?)
    // }
}

pub type WebsocketStream = futures::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
>;

pub struct TwitchWebsocketThread {
    config: SharedCoreConfig,
    context: TwitchWebsocketContext,
    tx: Sender<TwitchToTerminalAction>,
    rx: Receiver<TwitchAction>,
}

impl TwitchWebsocketThread {
    fn new(
        config: SharedCoreConfig,
        tx: Sender<TwitchToTerminalAction>,
        rx: Receiver<TwitchAction>,
    ) -> Self {
        Self {
            config,
            context: TwitchWebsocketContext::default(),
            tx,
            rx,
        }
    }

    async fn connect(&mut self) -> Result<WebsocketStream> {
        let url = self.config.twitch.config_twitch_websocket_url();
        let (ws_stream, _) = match connect_async(url).await {
            Ok(websocket_connection) => websocket_connection,
            Err(err) => {
                bail!(
                    "Failed to connect to websocket server at {}: {}",
                    self.config.twitch.server,
                    err
                )
            }
        };

        info!("Twitch websocket handshake successful");

        let mut stream = ws_stream.split().1;

        // If the dashboard is the start state, wait until the user has selected
        // a channel before connecting to Twitch's websocket server.
        if self.config.terminal.first_state == State::Dashboard {
            debug!("Waiting for user to select channel from debug screen");

            loop {
                if let Ok(TwitchAction::JoinChannel(channel_name)) = self.rx.recv().await {
                    self.context.set_channel_name(Some(channel_name));

                    debug!("User has selected channel from start screen");
                    break;
                }
            }
        }

        let emotes_enabled = self.config.frontend.is_emotes_enabled();

        self.context.set_emotes_state(emotes_enabled);
        self.context.set_token(self.config.twitch.token.clone());

        if stream.next().await.is_some() {
            debug!("Websocket server has pinged you to make sure you're here");
        }

        // Handle the welcome message, it should arrive after the initial ping
        let Some(Ok(Message::Text(message))) = stream.next().await else {
            let error_message = "Welcome message from websocket server was not found, something has gone terribly wrong";
            self.tx
                .send(DataBuilder::system(error_message.to_string()))
                .await?;
            bail!(error_message);
        };
        if let Err(err) = handle_welcome_message(&mut self.context, &self.tx, message).await {
            let error_message = format!("Failed to handle welcome message: {err}");
            self.tx
                .send(DataBuilder::system(error_message.clone()))
                .await?;
            bail!(error_message);
        }

        Ok(stream)
    }

    async fn run(&mut self) -> Result<()> {
        let mut stream = self.connect().await?;

        loop {
            tokio::select! {
                biased;

                Ok(action) = self.rx.recv() => {
                    match action {
                        TwitchAction::Message(message) => {
                            if let Some(command) = message.strip_prefix('/') {
                                if let Err(err) = handle_command_message(&self.context, &self.tx, command).await {
                                    error!("Failed to handle Twitch message command from terminal: {err}");
                                    self.tx.send(DataBuilder::twitch(format!("Failed to handle Twitch message command from terminal: {err}"))).await?;
                                }
                            }
                            else if let Err(err) = handle_send_message(&self.context, message).await {
                                error!("Failed to send Twitch message from terminal: {err}");
                            }
                        },
                        TwitchAction::JoinChannel(channel_name) => {
                            if let Err(err) = handle_channel_join(&mut self.context, &self.tx, channel_name, false).await {
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
                                self.config.clone(),
                                &self.context,
                                &self.tx,
                                received_message,
                                self.context.is_emotes_enabled(),
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
}
