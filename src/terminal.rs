use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{info, warn};

use crate::{
    app::App,
    commands::{init_terminal, quit_terminal, reset_terminal},
    config::SharedCoreConfig,
    emotes::{ApplyCommand, DecodedEmote, display_emote, query_emotes},
    events::{Event, Events, InternalEvent, TwitchAction, TwitchEvent, TwitchNotification},
    handlers::{
        data::{KNOWN_CHATTERS, MessageData},
        state::State,
    },
    twitch::oauth::TwitchOauth,
    ui::components::Component,
    utils::sanitization::clean_channel_name,
};

pub async fn ui_driver(
    config: SharedCoreConfig,
    mut app: App,
    mut events: Events,
    twitch_oauth: TwitchOauth,
    twitch_tx: Sender<TwitchAction>,
    mut drx: Option<Receiver<Result<DecodedEmote, String>>>,
) {
    info!("Started UI driver.");

    let mut erx = query_emotes(&config, twitch_oauth.clone(), config.twitch.channel.clone());

    let mut terminal = init_terminal(&config.frontend);

    let is_emotes_enabled = app.emotes.enabled;

    loop {
        if is_emotes_enabled {
            // Check if we have received any emotes
            if let Ok((user_emotes, global_emotes)) = erx.try_recv() {
                *app.emotes.user_emotes.borrow_mut() = user_emotes;
                *app.emotes.global_emotes.borrow_mut() = global_emotes;

                for message in &mut *app.messages.borrow_mut() {
                    message.reparse_emotes(&app.emotes);
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
                            app.emotes.user_emotes.borrow_mut().remove(&name);
                            app.emotes.global_emotes.borrow_mut().remove(&name);
                            app.emotes.info.borrow_mut().remove(&name);
                        }
                    }
                }
            }
        }

        if let Some(event) = events.next().await {
            match event.clone() {
                Event::Internal(internal_event) => {
                    match internal_event {
                        InternalEvent::Quit => {
                            // Emotes need to be unloaded before we exit the alternate screen
                            app.emotes.unload();
                            quit_terminal(terminal);

                            break;
                        }
                        InternalEvent::BackOneLayer => {
                            if let Some(previous_state) = app.get_previous_state() {
                                app.set_state(previous_state);
                            } else {
                                app.set_state(config.terminal.first_state.clone());
                            }
                        }
                        InternalEvent::SwitchState(state) => {
                            if state == State::Normal {
                                app.clear_messages();
                            }

                            app.set_state(state);
                        }
                        InternalEvent::OpenStream(channel) => {
                            app.open_stream(&channel);
                        }
                        InternalEvent::SelectEmote(_) => {}
                    }
                }
                Event::Twitch(twitch_event) => match twitch_event {
                    TwitchEvent::Action(twitch_action) => match twitch_action {
                        TwitchAction::JoinChannel(channel) => {
                            let channel = clean_channel_name(&channel);
                            app.clear_messages();
                            app.emotes.unload();

                            // TODO: Handle error
                            let _ = twitch_tx
                                .send(TwitchAction::JoinChannel(channel.clone()))
                                .await;

                            if config.frontend.autostart_view_command {
                                app.open_stream(&channel);
                            }
                            erx = query_emotes(&config, twitch_oauth.clone(), channel);
                            app.set_state(State::Normal);
                        }
                        TwitchAction::Message(message) => {
                            // TODO: Handle error
                            let _ = twitch_tx.send(TwitchAction::Message(message)).await;
                        }
                    },
                    TwitchEvent::Notification(twitch_notification) => {
                        match twitch_notification {
                            TwitchNotification::Message(m) => {
                                let message_data = MessageData::from_twitch_message(m, &app.emotes);
                                if !KNOWN_CHATTERS.contains(&message_data.author.as_str())
                                    && config.twitch.username != message_data.author
                                {
                                    app.storage
                                        .borrow_mut()
                                        .add("chatters", message_data.author.clone());
                                }
                                app.messages.borrow_mut().push_front(message_data);

                                // If scrolling is enabled, pad for more messages.
                                if app.components.chat.scroll_offset.get_offset() > 0 {
                                    app.components.chat.scroll_offset.up();
                                }
                            }
                            TwitchNotification::ClearChat(user_id) => {
                                if let Some(user) = user_id {
                                    app.purge_user_messages(user.as_str());
                                } else {
                                    app.clear_messages();
                                }
                            }
                            TwitchNotification::DeleteMessage(message_id) => {
                                app.remove_message_with(message_id.as_str());
                            }
                        }
                    }
                },
                _ => {}
            }

            // TODO: Handle possible errors
            let _ = app.event(&event).await;
        }

        terminal.draw(|f| app.draw(f, Some(f.area()))).unwrap();
    }

    app.cleanup();

    reset_terminal();
}
