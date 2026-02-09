use std::{fmt::Display, str::FromStr};

use color_eyre::eyre::{Error, bail};
use serde_with::{DeserializeFromStr, SerializeDisplay};

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
        }
    }
}

pub fn get_keybind_text(keybind: &[Key]) -> String {
    keybind
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(" or ")
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
        assert_eq!(Key::Esc, Key::from_str("Esc").unwrap());
        assert_eq!(Key::Esc, Key::from_str("esc").unwrap());
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
    fn key_parsing_display_symmetry() {
        let mut vec: Vec<Key> = Vec::new();
        for i in '0'..='z' {
            vec.extend_from_slice(&[Key::Char(i), Key::Ctrl(i), Key::Alt(i)]);
        }
        vec.extend_from_slice(&[
            Key::Backspace,
            Key::Esc,
            Key::Up,
            Key::Down,
            Key::Left,
            Key::Right,
            Key::Home,
            Key::End,
            Key::Delete,
            Key::Insert,
            Key::Tab,
            Key::Enter,
            Key::ScrollDown,
            Key::ScrollUp,
        ]);
        for key in &vec {
            assert_eq!(*key, Key::from_str(&key.to_string()).unwrap());
        }
    }
}
