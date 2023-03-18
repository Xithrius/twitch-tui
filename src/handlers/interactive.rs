use dialoguer::{console::Style, theme::ColorfulTheme, Confirm, Input};

use crate::handlers::config::{CompleteConfig, TwitchConfig};

pub(super) fn interactive_config() -> Option<CompleteConfig> {
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
        .unwrap();

    let token: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Token: ")
        .interact_text()
        .unwrap();

    let channel: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Channel: ")
        .interact_text()
        .unwrap();

    let server: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("IRC server: ")
        .default("irc.chat.twitch.tv".to_string())
        .interact_text()
        .unwrap();

    let api: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("API Token: ")
        .interact_text()
        .unwrap();

    let id: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("ID: ")
        .interact_text()
        .unwrap();

    Some(CompleteConfig {
        twitch: TwitchConfig {
            username,
            channel,
            server,
            token: Some(token),
            api: Some(api),
            id: Some(id),
        },
        ..Default::default()
    })
}
