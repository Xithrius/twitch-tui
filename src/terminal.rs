use log::{debug, info};
use std::time::Duration;
use tokio::sync::{broadcast::Sender, mpsc::Receiver};
use tui::layout::Rect;

use crate::{
    commands::{init_terminal, quit_terminal, reset_terminal},
    emotes::Emotes,
    handlers::{
        app::{App, State},
        config::CompleteConfig,
        data::MessageData,
        user_input::{
            events::{Config, Events, Key},
            input::{handle_stateful_user_input, TerminalAction},
        },
    },
    twitch::TwitchAction,
    ui::{
        components::dashboard::start::render_dashboard_ui, error::render_error_ui, render_chat_ui,
    },
};

pub async fn ui_driver(
    mut config: CompleteConfig,
    mut app: App,
    tx: Sender<TwitchAction>,
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

    if !app.storage.contains("channels", &config.twitch.channel) {
        app.storage.add("channels", config.twitch.channel.clone());
    }

    let mut terminal = init_terminal(&config.frontend);

    terminal.clear().unwrap();

    let mut emotes: Emotes = Emotes::default();
    let mut terminal_size = Rect::default();

    loop {
        if let Ok(e) = erx.try_recv() {
            emotes = e;
            for message in &mut app.messages {
                message.parse_emotes(&mut emotes);
            }
        };

        if let Ok(mut info) = rx.try_recv() {
            info.parse_emotes(&mut emotes);
            app.messages.push_front(info);

            // If scrolling is enabled, pad for more messages.
            if app.scrolling.get_offset() > 0 {
                app.scrolling.up();
            }
        }

        terminal
            .draw(|frame| {
                let size = frame.size();

                if size != terminal_size {
                    terminal_size = size;
                    emotes.displayed.clear();
                    emotes.loaded.clear();
                }

                if size.height < 10 || size.width < 60 {
                    render_error_ui(
                        frame,
                        &[
                            "Window to small!",
                            "Must allow for at least 60x10.",
                            "Restart and resize.",
                        ],
                    );
                } else if app.get_state() == State::Start
                    || (Some(State::Start) == app.get_previous_state()
                        && State::ChannelSwitch == app.get_state())
                {
                    render_dashboard_ui(frame, &mut app, &config);
                } else {
                    render_chat_ui(frame, &mut app, &config, &mut emotes);
                }
            })
            .unwrap();

        if matches!(
            handle_stateful_user_input(&mut events, &mut app, &mut config, tx.clone(), &mut emotes)
                .await,
            Some(TerminalAction::Quitting)
        ) {
            quit_terminal(terminal);

            break;
        }
    }

    app.cleanup();

    reset_terminal();
}
