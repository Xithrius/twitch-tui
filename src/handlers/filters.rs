use std::{cell::RefCell, fs::read_to_string, rc::Rc};

use regex::Regex;

use crate::handlers::config::{SharedCoreConfig, persistence::get_config_dir};

const DEFAULT_FILTERS_FILE_NAME: &str = "filters.txt";

pub type SharedFilters = Rc<RefCell<Filters>>;

#[derive(Debug, Clone)]
pub struct Filters {
    captures: Vec<Regex>,
    enabled: bool,
    reversed: bool,
}

impl Filters {
    pub fn new(config: &SharedCoreConfig) -> Self {
        let filters_config = &config.filters;

        let filters_path = get_config_dir().join(DEFAULT_FILTERS_FILE_NAME);
        let captures = read_to_string(filters_path).map_or_else(
            |_| vec![],
            |f| {
                f.split('\n')
                    .filter(|s| !s.is_empty())
                    .flat_map(Regex::new)
                    .collect::<Vec<Regex>>()
            },
        );

        Self {
            captures,
            enabled: filters_config.enabled,
            reversed: filters_config.reversed,
        }
    }

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

    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    pub const fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub const fn reversed(&self) -> bool {
        self.reversed
    }

    pub const fn reverse(&mut self) {
        self.reversed = !self.reversed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Filters {
        Filters {
            captures: vec![Regex::new("^bad.*$").unwrap()],
            enabled: true,
            reversed: false,
        }
    }

    #[test]
    fn test_contaminated() {
        let filters = setup();

        assert!(filters.contaminated("bad word"));
    }

    #[test]
    fn test_non_contaminated() {
        let filters = setup();

        assert!(!filters.contaminated("not a bad word"));
    }

    #[test]
    fn test_reversed_contaminated() {
        let mut filters = setup();

        filters.reverse();

        assert!(!filters.contaminated("bad word"));
    }

    #[test]
    fn test_reversed_non_contaminated() {
        let mut filters = setup();

        filters.reverse();

        assert!(filters.contaminated("not a bad word"));
    }
}
