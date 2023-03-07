use std::time::Duration;

use log::{debug, info};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    commands::{init_terminal, quit_terminal, reset_terminal},
    handlers::{
        app::App,
        config::CompleteConfig,
        data::Data,
        user_input::{
            events::{Config, Events, Key},
            input::{handle_stateful_user_input, TerminalAction},
        },
    },
    twitch::TwitchAction,
    ui::{draw_ui, error::draw_error_ui},
};

pub async fn ui_driver(
    mut config: CompleteConfig,
    mut app: App,
    tx: Sender<TwitchAction>,
    mut rx: Receiver<Data>,
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

    let mut terminal = init_terminal(&config.frontend);

    terminal.clear().unwrap();

    loop {
        if let Ok(info) = rx.try_recv() {
            app.messages.push_front(info);

            // If scrolling is enabled, pad for more messages.
            if app.scrolling.get_offset() > 0 {
                app.scrolling.up();
            }
        }

        terminal
            .draw(|frame| {
                let size = frame.size();

                if size.height < 10 || size.width < 60 {
                    draw_error_ui(
                        frame,
                        &[
                            "Window to small!",
                            "Must allow for at least 60x10.",
                            "Restart and resize.",
                        ],
                    );
                } else {
                    draw_ui(frame, &mut app, &config);
                }
            })
            .unwrap();

        if matches!(
            handle_stateful_user_input(&mut events, &mut app, &mut config, tx.clone()).await,
            Some(TerminalAction::Quitting)
        ) {
            quit_terminal(terminal);

            break;
        }
    }

    app.cleanup();

    reset_terminal();
}
