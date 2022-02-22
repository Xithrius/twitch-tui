use clap::Parser;

use crate::handlers::config::{CompleteConfig, Palette};

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
/// Twitch chat in the terminal
pub struct Cli {
    /// The streamer's name
    #[clap(short, long)]
    pub channel: Option<String>,

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
}

pub fn merge_args_into_config(config: &mut CompleteConfig, args: Cli) {
    // Twitch section of the config
    if let Some(ch) = args.channel {
        config.twitch.channel = ch;
    }

    // Terminal section of the config
    if let Some(tick_delay) = args.tick_delay {
        config.terminal.tick_delay = tick_delay;
    }
    if let Some(max_messages) = args.max_messages {
        config.terminal.maximum_messages = max_messages;
    }

    // Frontend section of the config
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
}
