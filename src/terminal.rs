use log::{debug, info};
use std::time::Duration;
use tokio::sync::{broadcast::Sender, mpsc::Receiver};
use tui::layout::Rect;

use crate::{
    commands::{init_terminal, quit_terminal, reset_terminal},
    emotes::DownloadedEmotes,
    handlers::{
        app::App,
        config::CompleteConfig,
        data::{DataBuilder, TwitchToTerminalAction},
        state::State,
        user_input::events::{Config, Events, Key},
    },
    twitch::TwitchAction,
};

pub enum TerminalAction {
    Quit,
    BackOneLayer,
    SwitchState(State),
    ClearMessages,
    Enter(TwitchAction),
}

pub async fn ui_driver(
    config: CompleteConfig,
    mut app: App,
    tx: Sender<TwitchAction>,
    mut rx: Receiver<TwitchToTerminalAction>,
    mut erx: Receiver<DownloadedEmotes>,
) {
    info!("Started UI driver.");

    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic| {
        debug!("Panic hook hit.");

        reset_terminal();
        original_hook(panic);
    }));

    let mut events = Events::with_config(Config {
        exit_key: Key::Null,
        tick_rate: Duration::from_millis(config.terminal.delay),
    })
    .await;

    let mut terminal = init_terminal(&config.frontend);

    terminal.clear().unwrap();

    let mut terminal_size = Rect::default();

    loop {
        if let Ok(e) = erx.try_recv() {
            // If the user switched channels too quickly,
            // emotes will be from the wrong channel for a short time.
            // Clear the emotes to use the ones from the right channel.
            app.emotes.unload();
            app.emotes.emotes = e;
            for message in app.messages.borrow_mut().iter_mut() {
                message.parse_emotes(&mut app.emotes);
            }
        };

        if let Ok(msg) = rx.try_recv() {
            match msg {
                TwitchToTerminalAction::Message(mut m) => {
                    m.parse_emotes(&mut app.emotes);
                    app.messages.borrow_mut().push_front(m);

                    // If scrolling is enabled, pad for more messages.
                    if app.components.chat.scroll_offset.get_offset() > 0 {
                        app.components.chat.scroll_offset.up();
                    }
                }
                TwitchToTerminalAction::ClearChat => {
                    app.clear_messages();
                }
                TwitchToTerminalAction::DeleteMessage(message_id) => {
                    app.remove_message_with(message_id.as_str());
                }
            }
        }

        if let Some(event) = events.next().await {
            if let Some(action) = app.event(&event) {
                match action {
                    TerminalAction::Quit => {
                        quit_terminal(terminal);

                        break;
                    }
                    TerminalAction::BackOneLayer => {
                        if let Some(previous_state) = app.get_previous_state() {
                            app.set_state(previous_state);
                        } else {
                            app.set_state(config.terminal.first_state.clone());
                        }
                    }
                    TerminalAction::SwitchState(state) => {
                        if state == State::Normal {
                            app.clear_messages();
                        }

                        app.set_state(state);
                    }
                    TerminalAction::ClearMessages => {
                        app.clear_messages();

                        tx.send(TwitchAction::ClearMessages).unwrap();
                    }
                    TerminalAction::Enter(action) => match action {
                        TwitchAction::Privmsg(message) => {
                            let message_data = DataBuilder::user(
                                config.twitch.username.to_string(),
                                message.to_string(),
                                None,
                            );

                            if let TwitchToTerminalAction::Message(mut msg) = message_data {
                                msg.parse_emotes(&mut app.emotes);

                                app.messages.borrow_mut().push_front(msg);

                                tx.send(TwitchAction::Privmsg(message)).unwrap();
                            }
                        }
                        TwitchAction::Join(channel) => {
                            app.clear_messages();
                            app.emotes.unload();

                            tx.send(TwitchAction::Join(channel)).unwrap();

                            app.set_state(State::Normal);
                        }
                        TwitchAction::ClearMessages => {}
                    },
                }
            }
        }

        terminal
            .draw(|f| {
                let size = f.size();

                if size != terminal_size {
                    terminal_size = size;
                    app.emotes.clear();
                    app.emotes.loaded.clear();
                }

                app.draw(f);
            })
            .unwrap();
    }

    app.cleanup();

    reset_terminal();
}
