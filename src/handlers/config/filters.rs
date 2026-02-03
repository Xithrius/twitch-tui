use std::vec;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct FiltersConfig {
    /// If filters should be enabled at all.
    pub enabled: bool,
    /// If the regex filters should be reversed.
    pub reversed: bool,
}

impl From<FiltersConfig> for Vec<(String, String)> {
    fn from(config: FiltersConfig) -> Self {
        vec![
            ("Enabled".to_string(), config.enabled.to_string()),
            ("Reversed".to_string(), config.reversed.to_string()),
        ]
    }
}
