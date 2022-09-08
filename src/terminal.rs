use std::{
    io::{stdout, Stdout},
    time::Duration,
};

use chrono::offset::Local;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{debug, info};
use tokio::sync::mpsc::{Receiver, Sender};
use tui::{backend::CrosstermBackend, layout::Constraint, Terminal};

use crate::{
    handlers::{
        app::{App, BufferName, State},
        config::CompleteConfig,
        data::{Data, DataBuilder, PayLoad},
        event::{Config, Events, Key},
    },
    input::{handle_user_input, TerminalAction},
    twitch::TwitchAction,
    ui::draw_ui,
    utils::text::align_text,
};

fn reset_terminal() {
    disable_raw_mode().unwrap();

    execute!(stdout(), LeaveAlternateScreen).unwrap();
}

fn init_terminal() -> Terminal<CrosstermBackend<Stdout>> {
    enable_raw_mode().unwrap();

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();

    let backend = CrosstermBackend::new(stdout);

    Terminal::new(backend).unwrap()
}

fn quit_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) {
    disable_raw_mode().unwrap();

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .unwrap();

    terminal.show_cursor().unwrap();
}

pub async fn ui_driver(
    config: CompleteConfig,
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

    let mut terminal = init_terminal();

    let username_column_title = align_text(
        "Username",
        &config.frontend.username_alignment,
        config.frontend.maximum_username_length,
    );

    let mut column_titles = vec![username_column_title.clone(), "Message content".to_string()];

    let mut table_constraints = vec![
        Constraint::Length(config.frontend.maximum_username_length),
        Constraint::Percentage(100),
    ];

    if config.frontend.date_shown {
        column_titles.insert(0, "Time".to_string());

        table_constraints.insert(
            0,
            Constraint::Length(
                Local::now()
                    .format(config.frontend.date_format.as_str())
                    .to_string()
                    .len() as u16,
            ),
        );
    }

    app.column_titles = Some(column_titles);
    app.table_constraints = Some(table_constraints);

    terminal.clear().unwrap();

    let data_builder = DataBuilder::new(&config.frontend.date_format);

    loop {
        if let Ok(info) = rx.try_recv() {
            match info.payload {
                PayLoad::Message(_) => app.messages.push_front(info),

                // If something such as a keypress failed, fallback to the normal state of the application.
                PayLoad::Err(err) => {
                    app.state = State::Normal;
                    app.selected_buffer = BufferName::Chat;

                    app.messages.push_front(data_builder.system(err));
                }
            }

            // If scrolling is enabled, pad for more messages.
            if app.scroll_offset > 0 {
                app.scroll_offset += 1;
            }
        }

        terminal
            .draw(|frame| draw_ui(frame, &mut app, &config))
            .unwrap();

        if let Some(TerminalAction::Quitting) =
            handle_user_input(&mut events, &mut app, &mut config.clone(), tx.clone()).await
        {
            quit_terminal(terminal);

            break;
        }
    }

    app.cleanup();

    reset_terminal();
}
