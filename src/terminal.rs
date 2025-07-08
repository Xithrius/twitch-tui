use std::time::Duration;

use tokio::sync::{broadcast::Sender, mpsc::Receiver};
use tracing::{debug, info, warn};

use crate::{
    commands::{init_terminal, quit_terminal, reset_terminal},
    emotes::{ApplyCommand, DecodedEmote, display_emote, query_emotes},
    handlers::{
        config::CoreConfig,
        context::Context,
        data::{MessageData, TwitchToTerminalAction},
        state::State,
        user_input::events::{EventConfig, Events},
    },
    twitch::TwitchAction,
};

pub enum TerminalAction {
    Quit,
    BackOneLayer,
    SwitchState(State),
    Enter(TwitchAction),
    OpenStream(String),
}

#[allow(
    clippy::match_wildcard_for_single_variants,
    clippy::cognitive_complexity
)]
pub async fn ui_driver(
    config: CoreConfig,
    mut context: Context,
    tx: Sender<TwitchAction>,
    mut rx: Receiver<TwitchToTerminalAction>,
    mut drx: Option<Receiver<Result<DecodedEmote, String>>>,
) {
    info!("Started UI driver.");

    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic| {
        debug!("Panic hook hit.");

        reset_terminal();
        original_hook(panic);
    }));

    let event_config = EventConfig::new(Duration::from_millis(config.terminal.delay));
    let mut events = Events::with_config(event_config);

    let mut erx = query_emotes(&config, config.twitch.channel.clone());

    let mut terminal = init_terminal(&config.frontend);

    terminal.clear().unwrap();

    let is_emotes_enabled = context.emotes.enabled;

    loop {
        if is_emotes_enabled {
            // Check if we have received any emotes
            if let Ok((user_emotes, global_emotes)) = erx.try_recv() {
                *context.emotes.user_emotes.borrow_mut() = user_emotes;
                *context.emotes.global_emotes.borrow_mut() = global_emotes;

                for message in &mut *context.messages.borrow_mut() {
                    message.reparse_emotes(&context.emotes);
                }
            }

            // Check if we need to load a decoded emote
            if let Some(rx) = &mut drx {
                if let Ok(r) = rx.try_recv() {
                    match r {
                        Ok(d) => {
                            if let Err(e) = d.apply() {
                                warn!("Unable to send command to load emote. {e}");
                            } else if let Err(e) = display_emote(d.id(), 1, d.cols()) {
                                warn!("Unable to send command to display emote. {e}");
                            }
                        }
                        Err(name) => {
                            warn!("Unable to load emote: {name}.");
                            context.emotes.user_emotes.borrow_mut().remove(&name);
                            context.emotes.global_emotes.borrow_mut().remove(&name);
                            context.emotes.info.borrow_mut().remove(&name);
                        }
                    }
                }
            }
        }

        if let Ok(msg) = rx.try_recv() {
            match msg {
                TwitchToTerminalAction::Message(m) => {
                    context
                        .messages
                        .borrow_mut()
                        .push_front(MessageData::from_twitch_message(m, &context.emotes));

                    // If scrolling is enabled, pad for more messages.
                    if context.components.chat.scroll_offset.get_offset() > 0 {
                        context.components.chat.scroll_offset.up();
                    }
                }
                TwitchToTerminalAction::ClearChat(user_id) => {
                    if let Some(user) = user_id {
                        context.purge_user_messages(user.as_str());
                    } else {
                        context.clear_messages();
                    }
                }
                TwitchToTerminalAction::DeleteMessage(message_id) => {
                    context.remove_message_with(message_id.as_str());
                }
            }
        }

        if let Some(event) = events.next().await {
            if let Some(action) = context.event(&event).await {
                match action {
                    TerminalAction::Quit => {
                        // Emotes need to be unloaded before we exit the alternate screen
                        context.emotes.unload();
                        quit_terminal(terminal);

                        break;
                    }
                    TerminalAction::BackOneLayer => {
                        if let Some(previous_state) = context.get_previous_state() {
                            context.set_state(previous_state);
                        } else {
                            context.set_state(config.terminal.first_state.clone());
                        }
                    }
                    TerminalAction::SwitchState(state) => {
                        if state == State::Normal {
                            context.clear_messages();
                        }

                        context.set_state(state);
                    }
                    TerminalAction::Enter(action) => {
                        if let TwitchAction::JoinChannel(channel) = action {
                            context.clear_messages();
                            context.emotes.unload();
                            tx.send(TwitchAction::JoinChannel(channel.clone())).unwrap();

                            if config.frontend.autostart_view_command {
                                //TODO dedupe (or should this be part of open_stream?)
                                let channel_name =
                                    if config.frontend.only_get_live_followed_channels {
                                        channel.split(':').next().map_or_else(
                                            || channel.as_str(),
                                            |name| name.trim_end(),
                                        )
                                    } else {
                                        channel.as_str()
                                    };
                                context.open_stream(channel_name);
                            }
                            erx = query_emotes(&config, channel);
                            context.set_state(State::Normal);
                        } else {
                            tx.send(action).unwrap();
                        }
                    }
                    TerminalAction::OpenStream(channel) => {
                        context.open_stream(&channel);
                    }
                }
            }
        }

        terminal.draw(|f| context.draw(f)).unwrap();
    }

    context.cleanup();

    reset_terminal();
}
