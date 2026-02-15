use color_eyre::{Result, eyre::bail};
use futures::StreamExt;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, info};

use crate::{
    config::SharedCoreConfig,
    events::{Event, TwitchAction},
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
        oauth::TwitchOauth,
    },
};

pub struct TwitchWebsocket {
    // rx: Receiver<TwitchAction>,
}

impl TwitchWebsocket {
    pub fn new(
        config: SharedCoreConfig,
        twitch_oauth: TwitchOauth,
        tx: Sender<Event>,
        rx: Receiver<TwitchAction>,
    ) -> Self {
        let mut context = TwitchWebsocketContext::default();
        context.set_oauth(Some(twitch_oauth));

        let mut actor = TwitchWebsocketThread::new(config, context, tx, rx);
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
    event_tx: Sender<Event>,
    action_rx: Receiver<TwitchAction>,
}

impl TwitchWebsocketThread {
    const fn new(
        config: SharedCoreConfig,
        context: TwitchWebsocketContext,
        event_tx: Sender<Event>,
        action_rx: Receiver<TwitchAction>,
    ) -> Self {
        Self {
            config,
            context,
            event_tx,
            action_rx,
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
            debug!("Waiting for user to select channel from dashboard screen");

            loop {
                if let Some(TwitchAction::JoinChannel(channel_name)) = self.action_rx.recv().await {
                    self.context.set_channel_name(Some(channel_name));

                    debug!("User has selected channel from start screen");
                    break;
                }
            }
        } else {
            self.context
                .set_channel_name(Some(self.config.twitch.channel.clone()));
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
            self.event_tx
                .send(DataBuilder::system(error_message.to_string()).into())
                .await?;
            bail!(error_message);
        };
        if let Err(err) = handle_welcome_message(&mut self.context, &self.event_tx, message).await {
            let error_message = format!("Failed to handle welcome message: {err}");
            self.event_tx
                .send(DataBuilder::system(error_message.clone()).into())
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

                Some(action) = self.action_rx.recv() => {
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
                    if let Err(err) =
                        handle_command_message(&self.context, &self.event_tx, command).await
                    {
                        self.event_tx
                            .send(
                                DataBuilder::twitch(format!(
                                    "Failed to handle Twitch message command from terminal: {err}"
                                ))
                                .into(),
                            )
                            .await?;
                        return Err(err);
                    }
                }

                handle_send_message(&self.context, message).await?;
            }
            TwitchAction::JoinChannel(channel_name) => {
                handle_channel_join(&mut self.context, &self.event_tx, channel_name, false).await?;
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
            &self.event_tx,
            received_message,
        )
        .await
    }
}
