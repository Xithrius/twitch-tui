const BINARY_NAME: &str = env!("CARGO_BIN_NAME");

pub fn config_path(file: &str) -> String {
    match std::env::consts::OS {
        "linux" | "macos" => format!(
            "{}/.config/{}/{}",
            std::env::var("HOME").unwrap(),
            BINARY_NAME,
            file
        ),
        "windows" => format!(
            "{}\\{}\\{}",
            std::env::var("APPDATA").unwrap(),
            BINARY_NAME,
            file
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
        assert_eq!(
            config_path("config.toml"),
            format!(
                "{}\\{}\\config.toml",
                std::env::var("APPDATA").unwrap(),
                BINARY_NAME
            )
        )
    }

    #[test]
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn test_unix_config_path() {
        assert_eq!(
            config_path("config.toml"),
            format!(
                "{}/.config/{}/config.toml",
                std::env::var("HOME").unwrap(),
                BINARY_NAME,
            )
        )
    }

    #[test]
    #[should_panic]
    #[cfg(any(
        target_os = "ios",
        target_os = "android",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd"
    ))]
    fn test_ios_config_path() {
        config_path("config.toml");
    }
}
