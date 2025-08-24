use std::{fmt::Display, str::FromStr, time::Duration};

use color_eyre::eyre::{Error, bail};

use serde_with::{DeserializeFromStr, SerializeDisplay};

use tokio::{sync::mpsc, time::Instant};
use tui::crossterm::event::{
    self, Event as CEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind,
};

#[derive(Debug, Clone, Copy, SerializeDisplay, DeserializeFromStr, PartialEq, Eq)]
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
    Char(char),
    Ctrl(char),
    Alt(char),
    Null,

    // Mouse controls
    ScrollUp,
    ScrollDown,
}

impl FromStr for Key {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn get_single_char(s: &str) -> Result<char, Error> {
            if s.trim().chars().count() == 1 {
                return Ok(s.trim().chars().next().expect("Must be a char here"));
            }
            bail!("Key char '{}' cannot be deserialized", s);
        }
        match s.to_lowercase().as_str() {
            "esc" => Ok(Self::Esc),
            "enter" => Ok(Self::Enter),
            "tab" => Ok(Self::Tab),
            "insert" => Ok(Self::Insert),
            "down" => Ok(Self::Down),
            "up" => Ok(Self::Up),
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "home" => Ok(Self::Home),
            "end" => Ok(Self::End),
            "delete" => Ok(Self::Delete),
            "backspace" => Ok(Self::Backspace),
            "scrolldown" => Ok(Self::ScrollDown),
            "scrollup" => Ok(Self::ScrollUp),
            "plus" => Ok(Self::Char('+')),
            _ => {
                if let Some((modifier, key)) = s.split_once('+') {
                    match modifier.to_lowercase().trim() {
                        "alt" => Ok(Self::Alt(get_single_char(key)?)),
                        "ctrl" => Ok(Self::Ctrl(get_single_char(key)?)),
                        _ => bail!("Key '{}' cannot be deserialized", s),
                    }
                } else {
                    Ok(Self::Char(get_single_char(s)?))
                }
            }
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Char('+') => write!(f, "+"),
            Self::Char(c) => write!(f, "{c}"),
            Self::Ctrl(c) => write!(f, "Ctrl+{c}"),
            Self::Alt(c) => write!(f, "Alt+{c}"),
            Self::Esc => write!(f, "Esc"),
            Self::Enter => write!(f, "Enter"),
            Self::Tab => write!(f, "Tab"),
            Self::Insert => write!(f, "Insert"),
            Self::Down => write!(f, "Down"),
            Self::Up => write!(f, "Up"),
            Self::Left => write!(f, "Left"),
            Self::Right => write!(f, "Right"),
            Self::Home => write!(f, "Home"),
            Self::End => write!(f, "End"),
            Self::Delete => write!(f, "Delete"),
            Self::Backspace => write!(f, "Backspace"),
            Self::ScrollDown => write!(f, "ScrollDown"),
            Self::ScrollUp => write!(f, "ScrollUp"),
            Self::Null => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_key_parsing() {
        assert_eq!(Key::Char('a'), Key::from_str("a").unwrap());
        assert_eq!(Key::Char('A'), Key::from_str("A").unwrap());
        assert_eq!(Key::Char('!'), Key::from_str("!").unwrap());
    }
    #[test]
    fn special_key_parsing() {
        assert_eq!(Key::Backspace, Key::from_str("backspace").unwrap());
        assert_eq!(Key::Backspace, Key::from_str("Backspace").unwrap());
        //TODO fill this in
    }

    #[test]
    fn parsing_modifiers() {
        assert_eq!(Key::Ctrl('a'), Key::from_str("ctrl + a").unwrap());
        assert_eq!(Key::Ctrl('a'), Key::from_str("ctrl+a").unwrap());
        assert_eq!(Key::Alt('B'), Key::from_str("Alt + B").unwrap());
        assert_eq!(Key::Alt('B'), Key::from_str("Alt+B").unwrap());
    }

    #[test]
    fn parse_seperator_key() {
        assert_eq!(Key::Char('+'), Key::from_str("plus").unwrap());
        assert_eq!(Key::Char('+'), Key::from_str("Plus").unwrap());
    }

    #[test]
    fn symmetry() {
        for i in '0'..='z' {
            let key = Key::Char(i);
            assert_eq!(key, Key::from_str(&key.to_string()).unwrap());

            let key = Key::Ctrl(i);
            assert_eq!(key, Key::from_str(&key.to_string()).unwrap());

            let key = Key::Alt(i);
            assert_eq!(key, Key::from_str(&key.to_string()).unwrap());
        }
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
pub struct EventConfig {
    tick_rate: Duration,
}

impl EventConfig {
    pub const fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }
}

impl Events {
    pub fn with_config(config: EventConfig) -> Self {
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
                                KeyCode::Enter => Key::Enter,
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
