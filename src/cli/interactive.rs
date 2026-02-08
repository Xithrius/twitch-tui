use dialoguer::{Confirm, Input, Password, console::Style, theme::ColorfulTheme};

use crate::config::{core::CoreConfig, twitch::TwitchConfig};

pub fn interactive_config() -> Option<CoreConfig> {
    let theme = ColorfulTheme {
        values_style: Style::new().yellow().dim(),
        ..ColorfulTheme::default()
    };
    println!("It looks like Twitch TUI is not configured yet.");

    if !Confirm::with_theme(&theme)
        .with_prompt("Do you want to use interactive wizard?")
        .interact()
        .ok()?
    {
        return None;
    }

    let username: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Username: ")
        .interact_text()
        .ok()?;

    let token: String = Password::with_theme(&theme)
        .with_prompt("Token (paste/type password and press enter): ")
        .interact()
        .ok()
        .map(|t| {
            if t.starts_with("oauth:") {
                t
            } else {
                format!("oauth:{t}")
            }
        })?;

    let channel: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Channel: ")
        .interact_text()
        .ok()?;

    let server: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Websocket server: ")
        .default("wss://eventsub.wss.twitch.tv/ws".to_string())
        .interact_text()
        .ok()?;

    let keepalive_timeout_seconds: usize = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Keep alive timeout: ")
        .default(30)
        .interact_text()
        .ok()?;

    Some(CoreConfig {
        twitch: TwitchConfig {
            username,
            channel,
            server,
            token: Some(token),
            keepalive_timeout_seconds,
        },
        ..Default::default()
    })
}
