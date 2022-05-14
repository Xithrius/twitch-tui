const BINARY_NAME: &str = env!("CARGO_BIN_NAME");

pub fn config_path() -> String {
    let partial_path = dirs::config_dir().unwrap();

    match std::env::consts::OS {
        "linux" | "macos" => format!(
            "{}/{}/config.toml",
            partial_path.to_string_lossy(),
            BINARY_NAME
        ),
        "windows" => format!(
            "{}\\{}\\config.toml",
            partial_path.to_string_lossy(),
            BINARY_NAME
        ),
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
