use std::{cell::RefCell, fs::read_to_string, rc::Rc};

use regex::Regex;

use crate::{handlers::config::FiltersConfig, utils::pathing::config_path};

pub type SharedFilters = Rc<RefCell<Filters>>;

#[derive(Debug, Clone)]
pub struct Filters {
    captures: Vec<Regex>,
    enabled: bool,
    reversed: bool,
}

impl Filters {
    pub fn new(file: &str, config: &FiltersConfig) -> Self {
        let file_path = config_path(file);

        Self {
            captures: read_to_string(file_path).map_or_else(
                |_| vec![],
                |f| {
                    f.split('\n')
                        .filter(|s| !s.is_empty())
                        .flat_map(Regex::new)
                        .collect::<Vec<Regex>>()
                },
            ),
            enabled: config.enabled,
            reversed: config.reversed,
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
