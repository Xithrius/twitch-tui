use std::time::Duration;

use crossterm::event::{self, Event as CEvent, KeyCode, KeyModifiers, MouseButton, MouseEventKind};
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
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Enter,
    Char(char),
    Ctrl(char),
    Alt(char),
    F(u8),
    Null,

    // Mouse controls
    ScrollUp,
    ScrollDown,
    PressedButton(MouseButton),
    ReleasedButton(MouseButton),
    Drag(MouseButton),
    Moved,
}

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Events {
    pub async fn with_config(config: Config) -> Events {
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
                        Ok(CEvent::Key(key)) => {
                            let key = match key.code {
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
                                KeyCode::PageUp => Key::PageUp,
                                KeyCode::PageDown => Key::PageDown,
                                KeyCode::Tab => Key::Tab,
                                KeyCode::BackTab => Key::BackTab,
                                KeyCode::Enter => Key::Enter,
                                KeyCode::Null => Key::Null,
                                KeyCode::F(k) => Key::F(k),
                                KeyCode::Char(c) => match key.modifiers {
                                    KeyModifiers::NONE | KeyModifiers::SHIFT => Key::Char(c),
                                    KeyModifiers::CONTROL => Key::Ctrl(c),
                                    KeyModifiers::ALT => Key::Alt(c),
                                    _ => Key::Null,
                                },
                            };
                            if let Err(err) = tx.send(Event::Input(key)).await {
                                eprintln!("Keyboard input error: {}", err);
                                return;
                            }
                        }
                        Ok(CEvent::Mouse(key)) => {
                            let key = match key.kind {
                                MouseEventKind::ScrollDown => Key::ScrollDown,
                                MouseEventKind::ScrollUp => Key::ScrollUp,
                                MouseEventKind::Down(button) => Key::PressedButton(button),
                                MouseEventKind::Up(button) => Key::ReleasedButton(button),
                                MouseEventKind::Drag(button) => Key::Drag(button),
                                MouseEventKind::Moved => Key::Moved,
                            };

                            if let Err(err) = tx.send(Event::Input(key)).await {
                                eprintln!("Mouse input error: {}", err);
                                return;
                            }
                        }
                        _ => (),
                    }
                }

                if last_tick.elapsed() >= config.tick_rate {
                    if let Err(err) = tx.send(Event::Tick).await {
                        eprintln!("{}", err);
                        return;
                    }
                    last_tick = Instant::now();
                }
            }
        });
        Events { rx }
    }

    pub async fn next(&mut self) -> Option<Event<Key>> {
        self.rx.recv().await
    }
}
