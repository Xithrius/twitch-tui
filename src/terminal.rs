use std::{
    fmt,
    io::{stdout, Stdout},
    time::Duration,
};

use crossterm::{
    cursor::{CursorShape, SetCursorShape},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    Command,
};
use log::{debug, info};
use tokio::sync::mpsc::{Receiver, Sender};
use tui::{backend::CrosstermBackend, Terminal};

use crate::{
    handlers::{
        app::App,
        config::{CompleteConfig, CursorType},
        data::Data,
        user_input::{
            events::{Config, Events, Key},
            input::{handle_stateful_user_input, TerminalAction},
        },
    },
    twitch::TwitchAction,
    ui::draw_ui,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResetCursorShape;

impl Command for ResetCursorShape {
    /// Fs escape sequence RIS for full reset
    /// <https://en.wikipedia.org/wiki/ANSI_escape_code#Fs_Escape_sequences/>
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        f.write_str("\x1Bc")
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

fn reset_terminal() {
    disable_raw_mode().unwrap();

    execute!(stdout(), LeaveAlternateScreen, ResetCursorShape).unwrap();
}

fn init_terminal(cursor_shape: CursorType) -> Terminal<CrosstermBackend<Stdout>> {
    enable_raw_mode().unwrap();

    let cursor_type = match cursor_shape {
        CursorType::Line => CursorShape::Line,
        CursorType::UnderScore => CursorShape::UnderScore,
        CursorType::Block => CursorShape::Block,
    };

    let mut stdout = stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        SetCursorShape(cursor_type),
    )
    .unwrap();

    let backend = CrosstermBackend::new(stdout);

    Terminal::new(backend).unwrap()
}

fn quit_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) {
    disable_raw_mode().unwrap();

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )
    .unwrap();

    terminal.show_cursor().unwrap();
}

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

    let mut terminal = init_terminal(config.frontend.cursor_shape.clone());

    terminal.clear().unwrap();

    loop {
        if let Ok(info) = rx.try_recv() {
            app.messages.push_front(info);

            // If scrolling is enabled, pad for more messages.
            if app.scroll_offset > 0 {
                app.scroll_offset += 1;
            }
        }

        terminal
            .draw(|frame| draw_ui(frame, &mut app, &config))
            .unwrap();

        if let Some(TerminalAction::Quitting) =
            handle_stateful_user_input(&mut events, &mut app, &mut config, tx.clone()).await
        {
            quit_terminal(terminal);

            break;
        }
    }

    app.cleanup();

    reset_terminal();
}
