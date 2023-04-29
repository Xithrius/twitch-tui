use std::time::Duration;

use crossterm::event::{
    self, Event as CEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind,
};
use tokio::{sync::mpsc, time::Instant};

#[derive(Debug, Clone, Copy)]
pub enum Key {
    // Keyboard controls
    Backspace,
    Esc,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    Delete,
    Insert,
    Tab,
    Enter,
    ShiftEnter,
    Char(char),
    Ctrl(char),
    Alt(char),
    Null,

    // Mouse controls
    ScrollUp,
    ScrollDown,
}

impl ToString for Key {
    fn to_string(&self) -> String {
        match self {
            Self::Char(c) | Self::Ctrl(c) | Self::Alt(c) => c,
            _ => unimplemented!(),
        }
        .to_string()
    }
}

pub enum Event {
    Input(Key),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Events {
    pub async fn with_config(config: Config) -> Self {
        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let mut last_tick = Instant::now();

            loop {
                let timeout = config
                    .tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).unwrap() {
                    match event::read() {
                        Ok(CEvent::Key(KeyEvent {
                            code,
                            kind: KeyEventKind::Press,
                            modifiers,
                            state: _,
                        })) => {
                            let key = match code {
                                KeyCode::Backspace => Key::Backspace,
                                KeyCode::Esc => Key::Esc,
                                KeyCode::Up => Key::Up,
                                KeyCode::Down => Key::Down,
                                KeyCode::Left => Key::Left,
                                KeyCode::Right => Key::Right,
                                KeyCode::Home => Key::Home,
                                KeyCode::End => Key::End,
                                KeyCode::Delete => Key::Delete,
                                KeyCode::Insert => Key::Insert,
                                KeyCode::Tab => Key::Tab,
                                KeyCode::Enter => match modifiers {
                                    KeyModifiers::SHIFT => Key::ShiftEnter,
                                    _ => Key::Enter,
                                },
                                KeyCode::Char(c) => match modifiers {
                                    KeyModifiers::NONE | KeyModifiers::SHIFT => Key::Char(c),
                                    KeyModifiers::CONTROL => Key::Ctrl(c),
                                    KeyModifiers::ALT => Key::Alt(c),
                                    _ => Key::Null,
                                },
                                _ => Key::Null,
                            };
                            if let Err(err) = tx.send(Event::Input(key)).await {
                                eprintln!("Keyboard input error: {err}");
                                return;
                            }
                        }
                        Ok(CEvent::Mouse(key)) => {
                            let key = match key.kind {
                                MouseEventKind::ScrollDown => Key::ScrollDown,
                                MouseEventKind::ScrollUp => Key::ScrollUp,
                                _ => Key::Null,
                            };

                            if let Err(err) = tx.send(Event::Input(key)).await {
                                eprintln!("Mouse input error: {err}");
                                return;
                            }
                        }
                        _ => (),
                    }
                }

                if last_tick.elapsed() >= config.tick_rate {
                    if let Err(err) = tx.send(Event::Tick).await {
                        eprintln!("{err}");
                        return;
                    }
                    last_tick = Instant::now();
                }
            }
        });

        Self { rx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}
