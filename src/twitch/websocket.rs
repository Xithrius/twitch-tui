use color_eyre::{Result, eyre::bail};
use futures::StreamExt;
use tokio::sync::{broadcast::Receiver, mpsc::Sender};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, info};

use crate::{
    handlers::{
        config::CoreConfig,
        data::{DataBuilder, TwitchToTerminalAction},
        state::State,
    },
    twitch::{
        actions::TwitchAction,
        context::TwitchWebsocketContext,
        handlers::{event_loop::websocket_event_loop, welcome_message::handle_welcome_message},
    },
};

#[allow(clippy::cognitive_complexity)]
pub async fn twitch_websocket(
    mut config: CoreConfig,
    tx: Sender<TwitchToTerminalAction>,
    mut rx: Receiver<TwitchAction>,
) -> Result<()> {
    let url = config.twitch.config_twitch_websocket_url();
    let (ws_stream, _) = match connect_async(url).await {
        Ok(websocket_connection) => websocket_connection,
        Err(err) => {
            bail!(
                "Failed to connect to websocket server at {}: {}",
                config.twitch.server,
                err
            )
        }
    };

    info!("Twitch websocket handshake successful");

    let (_, mut stream) = ws_stream.split();

    // If the dashboard is the start state, wait until the user has selected
    // a channel before connecting to Twitch's websocket server.
    if config.terminal.first_state == State::Dashboard {
        debug!("Waiting for user to select channel from debug screen");

        loop {
            if let Ok(TwitchAction::JoinChannel(channel)) = rx.recv().await {
                config.twitch.channel = channel;

                debug!("User has selected channel from start screen");
                break;
            }
        }
    }

    let emotes_enabled = config.frontend.is_emotes_enabled();

    let mut context = TwitchWebsocketContext::default();
    context.set_emotes(emotes_enabled);
    context.set_token(config.twitch.token.clone());

    if stream.next().await.is_some() {
        debug!("Websocket server has pinged you to make sure you're here");
    }

    // Handle the welcome message, it should arrive after the initial ping
    let Some(Ok(Message::Text(message))) = stream.next().await else {
        let error_message = "Welcome message from websocket server was not found, something has gone terribly wrong";
        tx.send(DataBuilder::system(error_message.to_string()))
            .await?;
        bail!(error_message);
    };
    if let Err(err) = handle_welcome_message(&mut config.twitch, &mut context, &tx, message).await {
        let error_message = format!("Failed to handle welcome message: {err}");
        tx.send(DataBuilder::system(error_message.clone())).await?;
        bail!(error_message);
    }

    websocket_event_loop(config, tx, rx, stream, emotes_enabled, context).await
}
