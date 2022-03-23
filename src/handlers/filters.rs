use std::fs::read_to_string;

use regex::Regex;

use crate::handlers::config::FiltersConfig;

#[derive(Debug, Clone)]
pub struct Filters {
    captures: Vec<Regex>,
    enabled: bool,
    reversed: bool,
}

impl Filters {
    pub fn new(filter_path: String, config: FiltersConfig) -> Self {
        Self {
            captures: if let Ok(f) = read_to_string(filter_path) {
                f.split('\n')
                    .filter(|s| !s.is_empty())
                    .flat_map(Regex::new)
                    .collect::<Vec<Regex>>()
            } else {
                vec![]
            },
            enabled: config.enabled,
            reversed: config.reversed,
        }
    }

    pub fn contaminated(&self, data: String) -> bool {
        if self.enabled {
            for re in &self.captures {
                if re.is_match(&data) {
                    return !self.reversed;
                }
            }
        }

        self.reversed
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn reversed(&self) -> bool {
        self.reversed
    }

    pub fn reverse(&mut self) {
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

        assert!(filters.contaminated("bad word".to_string()));
    }

    #[test]
    fn test_non_contaminated() {
        let filters = setup();

        assert!(!filters.contaminated("not a bad word".to_string()));
    }

    #[test]
    fn test_reversed_contaminated() {
        let mut filters = setup();

        filters.reverse();

        assert!(!filters.contaminated("bad word".to_string()));
    }

    #[test]
    fn test_reversed_non_contaminated() {
        let mut filters = setup();

        filters.reverse();

        assert!(filters.contaminated("not a bad word".to_string()));
    }
}
