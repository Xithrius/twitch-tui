use std::str::FromStr;

use clap::Parser;

use crate::handlers::config::{CompleteConfig, Palette, Theme};

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
#[clap(author, version, about)]
/// Twitch chat in the terminal
pub struct Cli {
    /// The streamer's name
    #[clap(short, long)]
    pub channel: Option<String>,
    /// File to log to
    #[clap(short, long)]
    pub log_file: Option<String>,
    /// If debug logs should be shown
    #[clap(short, long)]
    pub verbose: bool,
    /// The delay in milliseconds between terminal updates
    #[clap(short, long)]
    pub tick_delay: Option<u64>,
    /// The maximum amount of messages to be stored
    #[clap(short, long)]
    pub max_messages: Option<usize>,
    /// Show the date/time
    #[clap(short, long, possible_values = &["true", "false"])]
    pub date_shown: Option<String>,
    /// Maximum length for Twitch usernames
    #[clap(short = 'u', long)]
    pub max_username_length: Option<u16>,
    /// Username column alignment
    #[clap(short = 'a', long, possible_values = &["left", "center", "right"])]
    pub username_alignment: Option<String>,
    /// Username color palette
    #[clap(short, long, possible_values = &["pastel", "vibrant", "warm", "cool"])]
    pub palette: Option<Palette>,
    /// Twitch badges support
    #[clap(short, long)]
    pub badges: bool,
    /// The theme of the terminal
    #[clap(long, possible_values = &["dark", "light"])]
    pub theme: Option<String>,
}

pub fn merge_args_into_config(config: &mut CompleteConfig, args: Cli) {
    // Terminal arguments
    if let Some(log_file) = args.log_file {
        config.terminal.log_file = Some(log_file);
    }
    config.terminal.verbose = args.verbose;

    if let Some(tick_delay) = args.tick_delay {
        config.terminal.tick_delay = tick_delay;
    }
    if let Some(max_messages) = args.max_messages {
        config.terminal.maximum_messages = max_messages;
    }

    // Twitch arguments
    if let Some(channel) = args.channel {
        config.twitch.channel = channel;
    }

    // Frontend arguments
    if let Some(date_shown) = args.date_shown {
        config.frontend.date_shown = matches!(date_shown.as_str(), "true");
    }
    if let Some(maximum_username_length) = args.max_username_length {
        config.frontend.maximum_username_length = maximum_username_length;
    }
    if let Some(username_alignment) = args.username_alignment {
        config.frontend.username_alignment = username_alignment;
    }
    if let Some(palette) = args.palette {
        config.frontend.palette = palette;
    }
    config.frontend.badges = args.badges;
    if let Some(theme) = args.theme {
        config.frontend.theme = Theme::from_str(theme.as_str()).unwrap();
    }
}
