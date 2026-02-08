use color_eyre::{Result, eyre::bail};
use futures::StreamExt;
use tokio::sync::{broadcast::Receiver, mpsc::Sender};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, info};

use crate::{
    config::SharedCoreConfig,
    events::{TwitchAction, TwitchNotification},
    handlers::{data::DataBuilder, state::State},
    twitch::{
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
        tx: Sender<TwitchNotification>,
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
    tx: Sender<TwitchNotification>,
    rx: Receiver<TwitchAction>,
}

impl TwitchWebsocketThread {
    fn new(
        config: SharedCoreConfig,
        tx: Sender<TwitchNotification>,
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

        let (_, mut stream) = ws_stream.split();

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
                    if let Err(err) = self.handle_twitch_action(action).await {
                        error!("Failed to handle twitch action: {err}");
                    }
                }
                Some(message) = stream.next() => {
                    match message {
                        Ok(msg) => if let Err(err) = self.handle_websocket_stream_message(msg).await {
                            error!("Failed to handle websocket message: {err}");
                        },
                        Err(err) => {
                            error!("Twitch connection error encountered: {err}, attempting to reconnect.");
                        }
                    }
                }
                else => {}
            };
        }
    }

    async fn handle_twitch_action(&mut self, action: TwitchAction) -> Result<()> {
        match action {
            TwitchAction::Message(message) => {
                if let Some(command) = message.strip_prefix('/') {
                    if let Err(err) = handle_command_message(&self.context, &self.tx, command).await
                    {
                        self.tx
                            .send(DataBuilder::twitch(format!(
                                "Failed to handle Twitch message command from terminal: {err}"
                            )))
                            .await?;
                        return Err(err);
                    }
                }

                handle_send_message(&self.context, message).await?;
            }
            TwitchAction::JoinChannel(channel_name) => {
                handle_channel_join(&mut self.context, &self.tx, channel_name, false).await?;
            }
        }

        Ok(())
    }

    async fn handle_websocket_stream_message(&self, message: Message) -> Result<()> {
        let Message::Text(message_text) = message else {
            return Ok(());
        };

        let received_message = serde_json::from_str::<ReceivedTwitchMessage>(&message_text)?;

        handle_incoming_message(
            self.config.clone(),
            &self.context,
            &self.tx,
            received_message,
        )
        .await
    }
}
