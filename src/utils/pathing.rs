use std::panic::panic_any;

pub fn config_path() -> String {
    match std::env::consts::OS {
        "linux" | "macos" => match std::env::var("HOME") {
            Ok(env_home_path) => format!("{}/.config/ttc/config.toml", env_home_path),
            Err(err) => panic_any(err),
        },
        "windows" => match std::env::var("APPDATA") {
            Ok(appdata_path) => format!("{}\\{}", appdata_path, "ttc\\config.toml"),
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
                format!("{}\\{}", appdata_path, "ttc\\config.toml")
            ),
            Err(err) => std::panic::panic_any(err),
        }
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
