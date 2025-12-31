use std::vec;

use serde::{Deserialize, Serialize};

use crate::handlers::config::ToVec;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct FiltersConfig {
    /// If filters should be enabled at all.
    pub enabled: bool,
    /// If the regex filters should be reversed.
    pub reversed: bool,
}

impl ToVec<(String, String)> for FiltersConfig {
    fn to_vec(&self) -> Vec<(String, String)> {
        vec![
            ("Enabled".to_string(), self.enabled.to_string()),
            ("Reversed".to_string(), self.reversed.to_string()),
        ]
    }
}
