use std::{
    fmt,
    io::{stdout, Stdout, Write},
};

use crossterm::{
    cursor::{DisableBlinking, EnableBlinking, SetCursorStyle},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    Command,
};
use tui::{backend::CrosstermBackend, Terminal};

use crate::handlers::config::{CursorType, FrontendConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResetCursorShape;

/// Fs escape sequence RIS for full reset
/// <https://en.wikipedia.org/wiki/ANSI_escape_code#Fs_Escape_sequences/>
impl Command for ResetCursorShape {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        f.write_str("\x1Bc")
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

pub fn reset_terminal() {
    disable_raw_mode().unwrap();

    execute!(stdout(), LeaveAlternateScreen, ResetCursorShape).unwrap();
}

pub fn init_terminal(frontend_config: &FrontendConfig) -> Terminal<CrosstermBackend<Stdout>> {
    enable_raw_mode().unwrap();

    let cursor_style = match frontend_config.cursor_shape {
        CursorType::Line => SetCursorStyle::BlinkingBar,
        CursorType::Block => SetCursorStyle::BlinkingBlock,
        CursorType::UnderScore => SetCursorStyle::BlinkingUnderScore,
        CursorType::User => SetCursorStyle::DefaultUserShape,
    };

    let mut stdout = stdout();

    queue!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        cursor_style,
    )
    .unwrap();

    if frontend_config.blinking_cursor {
        queue!(stdout, EnableBlinking).unwrap();
    } else {
        queue!(stdout, DisableBlinking).unwrap();
    }

    stdout.flush().unwrap();

    let backend = CrosstermBackend::new(stdout);

    Terminal::new(backend).unwrap()
}

pub fn quit_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) {
    disable_raw_mode().unwrap();

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )
    .unwrap();

    terminal.show_cursor().unwrap();
}
