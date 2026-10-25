use std::{
    fmt,
    io::{Write, stdout},
};

use color_eyre::Result;
use tui::{
    DefaultTerminal, Terminal,
    backend::CrosstermBackend,
    crossterm::{
        Command,
        cursor::{DisableBlinking, EnableBlinking, SetCursorStyle},
        event::{DisableMouseCapture, EnableMouseCapture},
        execute, queue,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};

use crate::config::{CursorType, FrontendConfig};

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

pub fn reset_terminal() -> Result<()> {
    disable_raw_mode()?;

    execute!(
        stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        ResetCursorShape
    )?;

    Ok(())
}

pub fn init_terminal(frontend_config: &FrontendConfig) -> Result<DefaultTerminal> {
    enable_raw_mode()?;

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
    )?;

    if frontend_config.blinking_cursor {
        queue!(stdout, EnableBlinking)?;
    } else {
        queue!(stdout, DisableBlinking)?;
    }

    stdout.flush()?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    Ok(terminal)
}
