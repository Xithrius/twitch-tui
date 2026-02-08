use tokio::sync::{broadcast::Sender, mpsc::Receiver};
use tracing::{info, warn};

use crate::{
    commands::{init_terminal, quit_terminal, reset_terminal},
    config::SharedCoreConfig,
    emotes::{ApplyCommand, DecodedEmote, display_emote, query_emotes},
    events::Events,
    handlers::{
        context::Context,
        data::{KNOWN_CHATTERS, MessageData, TwitchToTerminalAction},
        state::State,
    },
    twitch::TwitchAction,
    utils::sanitization::clean_channel_name,
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
    config: SharedCoreConfig,
    mut context: Context,
    tx: Sender<TwitchAction>,
    mut rx: Receiver<TwitchToTerminalAction>,
    mut drx: Option<Receiver<Result<DecodedEmote, String>>>,
) {
    info!("Started UI driver.");

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        reset_terminal();
        original_hook(panic);
    }));

    let mut events = Events::new(config.terminal.delay);

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
                    let message_data = MessageData::from_twitch_message(m, &context.emotes);
                    if !KNOWN_CHATTERS.contains(&message_data.author.as_str())
                        && config.twitch.username != message_data.author
                    {
                        context
                            .storage
                            .borrow_mut()
                            .add("chatters", message_data.author.clone());
                    }
                    context.messages.borrow_mut().push_front(message_data);

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
                            let channel = clean_channel_name(&channel);
                            context.clear_messages();
                            context.emotes.unload();
                            tx.send(TwitchAction::JoinChannel(channel.clone())).unwrap();

                            if config.frontend.autostart_view_command {
                                context.open_stream(&channel);
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
