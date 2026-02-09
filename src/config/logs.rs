use std::fmt;

use serde::{Deserialize, Serialize};

#[allow(clippy::upper_case_acronyms)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::DEBUG => "debug",
                Self::INFO => "info",
                Self::WARN => "warn",
                Self::ERROR => "error",
            }
        )
    }
}
