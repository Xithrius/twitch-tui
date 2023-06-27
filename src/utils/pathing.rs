use anyhow::{anyhow, Result};
use std::{
    env,
    fs::remove_file,
    fs::File,
    io::Write,
    mem::drop,
    path::{Path, PathBuf},
};

const BINARY_NAME: &str = env!("CARGO_BIN_NAME");

pub fn config_path(file: &str) -> String {
    match env::consts::OS {
        "linux" | "macos" => format!(
            "{}/.config/{}/{}",
            env::var("HOME").unwrap(),
            BINARY_NAME,
            file
        ),
        "windows" => format!(
            "{}\\{}\\{}",
            env::var("APPDATA").unwrap(),
            BINARY_NAME,
            file
        ),
        _ => unimplemented!(),
    }
}

pub fn cache_path(file: &str) -> String {
    match env::consts::OS {
        "linux" | "macos" => format!(
            "{}/.cache/{}/{}",
            env::var("HOME").unwrap(),
            BINARY_NAME,
            file
        ),
        "windows" => format!(
            "{}\\{}\\{}\\{}",
            env::var("APPDATA").unwrap(),
            BINARY_NAME,
            "cache",
            file
        ),
        _ => unimplemented!(),
    }
}

pub fn create_temp_file(prefix: &str) -> Result<(File, PathBuf)> {
    let (tempfile, pathbuf) = tempfile::Builder::new()
        .prefix(prefix)
        .rand_bytes(5)
        .tempfile()?
        .keep()?;

    Ok((tempfile, pathbuf))
}

pub fn save_in_temp_file(buffer: &[u8], file: &mut File) -> Result<()> {
    file.write_all(buffer)?;
    file.flush()?;
    Ok(())
}

pub fn remove_temp_file(path: &Path) {
    drop(remove_file(path));
}

pub fn pathbuf_try_to_string(pathbuf: &Path) -> Result<String> {
    pathbuf.to_str().map_or_else(
        || {
            remove_temp_file(pathbuf);
            Err(anyhow!("Could not convert pathbuf to string."))
        },
        |str| Ok(str.to_string()),
    )
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
                env::var("APPDATA").unwrap(),
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
                env::var("HOME").unwrap(),
                BINARY_NAME,
            )
        );
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
