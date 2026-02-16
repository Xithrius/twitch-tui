use std::{cell::RefCell, fs::read_to_string, path::PathBuf, rc::Rc, string::ToString};

use color_eyre::Result;
use regex::Regex;

use crate::config::{SharedCoreConfig, get_config_dir};

const DEFAULT_MESSAGE_FILTERS_FILE_NAME: &str = "message_filters.txt";
const DEFAULT_USERNAME_FILTERS_FILE_NAME: &str = "username_filters.txt";

pub type SharedFilters = Rc<RefCell<Filters>>;

#[derive(Debug, Clone)]
pub struct Filters {
    pub message: MessageFilters,
    pub username: UsernameFilters,
}

#[derive(Debug, Clone)]
pub struct MessageFilters {
    captures: Vec<Regex>,
    enabled: bool,
    reversed: bool,
}

impl MessageFilters {
    pub fn contaminated(&self, data: &str) -> bool {
        if self.enabled {
            for re in &self.captures {
                if re.is_match(data) {
                    return !self.reversed;
                }
            }
        }

        self.reversed
    }

    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub const fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub const fn is_reversed(&self) -> bool {
        self.reversed
    }

    pub const fn reverse(&mut self) {
        self.reversed = !self.reversed;
    }
}

#[derive(Debug, Clone)]
pub struct UsernameFilters {
    captures: Vec<Regex>,
    enabled: bool,
    reversed: bool,
}

impl UsernameFilters {
    pub fn contaminated(&self, data: &str) -> bool {
        if self.enabled {
            for re in &self.captures {
                if re.is_match(data) {
                    return !self.reversed;
                }
            }
        }

        self.reversed
    }

    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub const fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub const fn is_reversed(&self) -> bool {
        self.reversed
    }

    pub const fn reverse(&mut self) {
        self.reversed = !self.reversed;
    }
}

fn parse_filters_regex(entries: Vec<String>) -> Vec<Regex> {
    entries
        .into_iter()
        .filter(|s| !s.is_empty())
        .flat_map(|s| Regex::new(&s))
        .collect()
}

fn parse_filters_file(path: PathBuf) -> Result<Vec<Regex>> {
    let captures = parse_filters_regex(
        read_to_string(path)?
            .split('\n')
            .map(ToString::to_string)
            .collect(),
    );

    Ok(captures)
}

/// Get the filter array from the config.
/// If that array doesn't exist, then we'll try seeing if there's anything
/// at the filter config path. If we can't parse it, we populate the filters
/// with an empty vector. Otherwise if the config path hasn't been provided,
/// we go to the default filters path.
fn parse_filters(
    config_filters: Option<Vec<String>>,
    filters_path: Option<PathBuf>,
    default_filters_file: PathBuf,
) -> Vec<Regex> {
    config_filters.map_or_else(
        || {
            filters_path.map_or_else(
                || {
                    if default_filters_file.exists() {
                        parse_filters_file(default_filters_file).unwrap_or_default()
                    } else {
                        vec![]
                    }
                },
                |filters_path| parse_filters_file(filters_path).unwrap_or_default(),
            )
        },
        parse_filters_regex,
    )
}

impl Filters {
    pub fn new(config: &SharedCoreConfig) -> Self {
        let config_dir = get_config_dir();

        let message_filters_config = config.filters.message.clone();
        let message_filters = parse_filters(
            message_filters_config.filters,
            message_filters_config.path,
            config_dir.join(DEFAULT_MESSAGE_FILTERS_FILE_NAME),
        );

        let username_filters_config = config.filters.username.clone();
        let username_filters = parse_filters(
            username_filters_config.filters,
            username_filters_config.path,
            config_dir.join(DEFAULT_USERNAME_FILTERS_FILE_NAME),
        );

        Self {
            message: MessageFilters {
                captures: message_filters,
                enabled: message_filters_config.enabled,
                reversed: message_filters_config.reversed,
            },
            username: UsernameFilters {
                captures: username_filters,
                enabled: username_filters_config.enabled,
                reversed: username_filters_config.reversed,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_message_filters() -> MessageFilters {
        MessageFilters {
            captures: vec![Regex::new("^bad.*$").unwrap()],
            enabled: true,
            reversed: false,
        }
    }

    fn setup_username_filters() -> UsernameFilters {
        UsernameFilters {
            captures: vec![Regex::new("^not-good.*$").unwrap()],
            enabled: true,
            reversed: false,
        }
    }

    #[test]
    fn test_contaminated_message() {
        let filters = setup_message_filters();

        assert!(filters.contaminated("bad word"));
    }

    #[test]
    fn test_non_contaminated_message() {
        let filters = setup_message_filters();

        assert!(!filters.contaminated("not a bad word"));
    }

    #[test]
    fn test_reversed_contaminated_message() {
        let mut filters = setup_message_filters();

        filters.reverse();

        assert!(!filters.contaminated("bad word"));
    }

    #[test]
    fn test_reversed_non_contaminated_message() {
        let mut filters = setup_message_filters();

        filters.reverse();

        assert!(filters.contaminated("not a bad word"));
    }

    #[test]
    fn test_contaminated_username() {
        let filters = setup_username_filters();

        assert!(filters.contaminated("not-good-username"));
    }

    #[test]
    fn test_non_contaminated_username() {
        let filters = setup_username_filters();

        assert!(!filters.contaminated("good-username"));
    }

    #[test]
    fn test_reversed_contaminated_username() {
        let mut filters = setup_username_filters();

        filters.reverse();

        assert!(!filters.contaminated("not-good-username"));
    }

    #[test]
    fn test_reversed_non_contaminated_username() {
        let mut filters = setup_username_filters();

        filters.reverse();

        assert!(filters.contaminated("good-username"));
    }
}
