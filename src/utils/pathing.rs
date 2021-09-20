use std::panic::panic_any;

pub fn config_path() -> String {
    match std::env::var("HOME") {
        Ok(env_home_path) => {
            return match std::env::consts::OS {
                "linux" | "macos" => {
                    format!("{}/.config/ttc/config.toml", env_home_path)
                }
                "windows" => "%appdata%\\ttc\\config.toml".to_string(),
                _ => unimplemented!(),
            }
        }
        Err(err) => panic_any(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_config_path() {
        assert_eq!(config_path(), "%appdata%\\ttc\\config.toml");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_config_path() {
        match std::env::var("HOME") {
            Ok(env_home_path) => assert_eq!(
                config_path(),
                format!("{}/{}", env_home_path, ".config/ttc/config.toml")
            ),
            Err(err) => std::panic::panic_any(err),
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_config_path() {
        match std::env::var("HOME") {
            Ok(env_home_path) => assert_eq!(
                config_path(),
                format!("{}/{}", env_home_path, ".config/ttc/config.toml")
            ),
            Err(err) => std::panic::panic_any(err),
        }
    }
}
