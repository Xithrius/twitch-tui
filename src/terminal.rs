use log::{debug, info};
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tui::layout::Rect;

use crate::{
    commands::{init_terminal, quit_terminal, reset_terminal},
    emotes::{unload_all_emotes, Emotes},
    handlers::{
        app::App,
        config::CompleteConfig,
        data::MessageData,
        state::State,
        user_input::{
            events::{Config, Events, Key},
            input::TerminalAction,
        },
    },
};

pub async fn ui_driver(
    config: CompleteConfig,
    mut app: App,
    mut rx: Receiver<MessageData>,
    mut erx: Receiver<Emotes>,
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
        tick_rate: Duration::from_millis(config.terminal.tick_delay),
    })
    .await;

    if !app
        .storage
        .borrow()
        .contains("channels", &config.twitch.channel)
    {
        app.storage
            .borrow_mut()
            .add("channels", config.twitch.channel.clone());
    }

    let mut terminal = init_terminal(&config.frontend);

    terminal.clear().unwrap();

    let mut emotes: Emotes = Emotes::default();

    let mut terminal_size = Rect::default();

    loop {
        if let Ok(e) = erx.try_recv() {
            emotes = e;
            for message in app.messages.borrow().iter() {
                message.clone().parse_emotes(&mut emotes);
            }
        };

        if let Ok(mut info) = rx.try_recv() {
            info.parse_emotes(&mut emotes);
            app.messages.borrow_mut().push_front(info);

            // If scrolling is enabled, pad for more messages.
            if app.components.chat.scroll_offset.get_offset() > 0 {
                app.components.chat.scroll_offset.up();
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
                            app.set_state(config.terminal.start_state.clone());
                        }
                    }
                    TerminalAction::SwitchState(state) => {
                        if state == State::Normal {
                            unload_all_emotes(&mut emotes);
                            app.clear_messages();
                        }

                        app.set_state(state);
                    }
                }
            }
        }

        terminal
            .draw(|f| {
                let size = f.size();

                if size != terminal_size {
                    terminal_size = size;
                    emotes.displayed.clear();
                    emotes.loaded.clear();
                }

                app.draw(f, emotes.clone());
            })
            .unwrap();
    }

    app.cleanup();

    reset_terminal();
}
