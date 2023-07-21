use clap::{builder::PossibleValue, Parser, ValueEnum};

use crate::handlers::{
    config::{CompleteConfig, Palette, Theme},
    state::State,
};

impl ValueEnum for Palette {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Pastel, Self::Vibrant, Self::Warm, Self::Cool]
    }

    fn to_possible_value<'a>(&self) -> Option<PossibleValue> {
        Some(PossibleValue::new(match self {
            Self::Pastel => "pastel",
            Self::Vibrant => "vibrant",
            Self::Warm => "warm",
            Self::Cool => "cool",
        }))
    }
}

impl ValueEnum for Theme {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Dark, Self::Light]
    }

    fn to_possible_value<'a>(&self) -> Option<PossibleValue> {
        Some(PossibleValue::new(match self {
            Self::Light => "light",
            _ => "dark",
        }))
    }
}

impl ValueEnum for State {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Dashboard, Self::Normal, Self::Help]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(PossibleValue::new(match self {
            Self::Dashboard => "start",
            Self::Normal => "normal",
            Self::Help => "help",
        }))
    }
}

#[derive(Parser, Debug)]
#[clap(rename_all = "kebab-case")]
#[clap(author, version, about)]
/// Twitch chat in the terminal
pub struct Cli {
    /// The streamer's name
    #[arg(short, long)]
    pub channel: Option<String>,
    /// File to log to
    #[arg(short, long)]
    pub log_file: Option<String>,
    /// If debug logs should be shown
    #[arg(short, long)]
    pub verbose: bool,
    /// The delay in milliseconds between terminal updates
    #[arg(short, long)]
    pub delay: Option<u64>,
    /// The maximum amount of messages to be stored
    #[arg(short, long)]
    pub max_messages: Option<usize>,
    /// Show the date/time
    #[arg(short, long)]
    pub show_datetimes: bool,
    /// Username color palette
    #[arg(short, long)]
    pub palette: Option<Palette>,
    /// Twitch badges support
    #[arg(short, long)]
    pub badges: bool,
    /// The theme of the terminal
    #[arg(short, long)]
    pub theme: Option<Theme>,
    /// The starting state of the terminal
    #[arg(short, long)]
    pub first_state: Option<State>,
}

pub fn merge_args_into_config(config: &mut CompleteConfig, args: Cli) {
    // Terminal arguments
    if let Some(log_file) = args.log_file {
        config.terminal.log_file = Some(log_file);
    }

    config.terminal.verbose = config.terminal.verbose || args.verbose;

    if let Some(delay) = args.delay {
        config.terminal.delay = delay;
    }
    if let Some(max_messages) = args.max_messages {
        config.terminal.maximum_messages = max_messages;
    }
    if let Some(first_state) = args.first_state {
        config.terminal.first_state = first_state;
    }

    // Twitch arguments
    if let Some(channel) = args.channel {
        config.twitch.channel = channel;
    }

    // Frontend arguments
    config.frontend.show_datetimes = config.frontend.show_datetimes || args.show_datetimes;

    if let Some(palette) = args.palette {
        config.frontend.palette = palette;
    }

    config.frontend.badges = config.frontend.badges || args.badges;

    if let Some(theme) = args.theme {
        config.frontend.theme = theme;
    }
}
