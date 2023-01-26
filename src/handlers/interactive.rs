use crate::handlers::config::{CompleteConfig, TwitchConfig};
use dialoguer::console::Style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input};

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

    Some(CompleteConfig {
        twitch: TwitchConfig {
            username,
            channel,
            server,
            token: Some(token),
        },
        ..Default::default()
    })
}
