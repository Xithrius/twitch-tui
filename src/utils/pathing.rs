use std::panic::panic_any;

const BINARY_NAME: &str = env!("CARGO_BIN_NAME");

pub fn config_path() -> String {
    match std::env::consts::OS {
        "linux" | "macos" => match std::env::var("HOME") {
            Ok(env_home_path) => format!("{}/.config/{}/config.toml", env_home_path, BINARY_NAME),
            Err(err) => panic_any(err),
        },
        "windows" => match std::env::var("APPDATA") {
            Ok(appdata_path) => format!("{}\\{}\\config.toml", appdata_path, BINARY_NAME),
            Err(err) => std::panic::panic_any(err),
        },
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_config_path() {
        match std::env::var("APPDATA") {
            Ok(appdata_path) => assert_eq!(
                config_path(),
                format!("{}\\{}\\config.toml", appdata_path, BINARY_NAME)
            ),
            Err(err) => std::panic::panic_any(err),
        }
    }

    #[test]
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn test_unix_config_path() {
        match std::env::var("HOME") {
            Ok(env_home_path) => assert_eq!(
                config_path(),
                format!("{}/.config/{}/config.toml", env_home_path, BINARY_NAME)
            ),
            Err(err) => std::panic::panic_any(err),
        }
    }
}
