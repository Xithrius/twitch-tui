use std::{str::FromStr, vec};

use color_eyre::eyre::{Error, Result, bail};
use serde::{Deserialize, Serialize};
use serde_with::DeserializeFromStr;
use tui::widgets::BorderType;

use crate::handlers::config::ToVec;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct FrontendConfig {
    /// The command and arguments that should be used to view the stream
    pub view_command: Vec<String>,
    /// Whether `view_command` should automatically be started when opening a stream
    pub autostart_view_command: bool,
    /// If the time and date is to be shown.
    pub show_datetimes: bool,
    /// The format of string that will show up in the terminal.
    pub datetime_format: String,
    /// If the username should be shown.
    pub username_shown: bool,
    /// The color palette.
    pub palette: Palette,
    /// Show Title with time and channel.
    pub title_shown: bool,
    /// The amount of space between the chat window and the terminal border.
    pub margin: u16,
    /// Show twitch badges next to usernames.
    pub badges: bool,
    /// Theme, being either light or dark.
    pub theme: Theme,
    /// If the username should be highlighted when it appears in chat.
    pub username_highlight: bool,
    /// If there should be state tabs shown on the bottom of the terminal.
    pub state_tabs: bool,
    /// The shape of the cursor in insert boxes.
    pub cursor_shape: CursorType,
    /// If the cursor should be blinking.
    pub blinking_cursor: bool,
    /// If the scrolling should be inverted.
    pub inverted_scrolling: bool,
    /// If scroll offset integer should be shown.
    pub show_scroll_offset: bool,
    /// If twitch emotes should be displayed (requires kitty terminal).
    pub twitch_emotes: bool,
    /// If betterttv emotes should be displayed (requires kitty terminal).
    pub betterttv_emotes: bool,
    /// If 7tv emotes should be displayed (requires kitty terminal).
    pub seventv_emotes: bool,
    /// If frankerfacez emotes should be displayed (requires kitty terminal).
    pub frankerfacez_emotes: bool,
    /// Channels to always be displayed in the start screen.
    pub favorite_channels: Vec<String>,
    /// The amount of recent channels that should be shown on the start screen.
    pub recent_channel_count: u16,
    /// A border wrapper around [`BorderType`].
    pub border_type: Border,
    /// If chat border should be hidden
    pub hide_chat_border: bool,
    /// If the usernames should be aligned to the right.
    pub right_align_usernames: bool,
    /// Do not display the window size warning.
    pub show_unsupported_screen_size: bool,
    /// Only show followed channels that are currently live.
    pub only_get_live_followed_channels: bool,
}

impl FrontendConfig {
    pub const fn is_emotes_enabled(&self) -> bool {
        self.twitch_emotes
            || self.betterttv_emotes
            || self.seventv_emotes
            || self.frankerfacez_emotes
    }
}

impl Default for FrontendConfig {
    fn default() -> Self {
        Self {
            view_command: vec![],
            autostart_view_command: false,
            show_datetimes: true,
            datetime_format: "%a %b %e %T %Y".to_string(),
            username_shown: true,
            palette: Palette::default(),
            title_shown: true,
            margin: 0,
            badges: false,
            theme: Theme::default(),
            username_highlight: true,
            state_tabs: false,
            cursor_shape: CursorType::default(),
            blinking_cursor: false,
            inverted_scrolling: false,
            show_scroll_offset: false,
            twitch_emotes: false,
            betterttv_emotes: false,
            seventv_emotes: false,
            frankerfacez_emotes: false,
            favorite_channels: vec![],
            recent_channel_count: 5,
            border_type: Border::default(),
            hide_chat_border: false,
            right_align_usernames: false,
            show_unsupported_screen_size: true,
            only_get_live_followed_channels: false,
        }
    }
}

impl ToVec<(String, String)> for FrontendConfig {
    fn to_vec(&self) -> Vec<(String, String)> {
        vec![
            ("View command".to_string(), self.view_command.join(" ")),
            (
                "Autostart view command".to_string(),
                self.autostart_view_command.to_string(),
            ),
            (
                "Show datetimes".to_string(),
                self.show_datetimes.to_string(),
            ),
            ("Datetime format".to_string(), self.datetime_format.clone()),
            (
                "Username shown".to_string(),
                self.username_shown.to_string(),
            ),
            // ("".to_string(), self.palette.to_string()),
            ("Title shown".to_string(), self.title_shown.to_string()),
            ("Margin".to_string(), self.margin.to_string()),
            ("Badges".to_string(), self.badges.to_string()),
            // ("".to_string(), self.theme.to_string()),
            (
                "Username highlight".to_string(),
                self.username_highlight.to_string(),
            ),
            ("State tabs".to_string(), self.state_tabs.to_string()),
            // ("".to_string(), self.cursor_shape.to_string()),
            (
                "Blinking cursor".to_string(),
                self.blinking_cursor.to_string(),
            ),
            (
                "Inverted scrolling".to_string(),
                self.inverted_scrolling.to_string(),
            ),
            (
                "Scroll offset shown".to_string(),
                self.show_scroll_offset.to_string(),
            ),
            ("Twitch emotes".to_string(), self.twitch_emotes.to_string()),
            (
                "BetterTTV emotes".to_string(),
                self.betterttv_emotes.to_string(),
            ),
            (
                "SevenTV emotes".to_string(),
                self.seventv_emotes.to_string(),
            ),
            (
                "FrankerFacez emotes".to_string(),
                self.frankerfacez_emotes.to_string(),
            ),
            // ("".to_string(), self.favorite_channels.to_string()),
            (
                "Recent channel count".to_string(),
                self.recent_channel_count.to_string(),
            ),
            // ("".to_string(), self.border_type.to_string()),
            (
                "Right aligned usernames".to_string(),
                self.right_align_usernames.to_string(),
            ),
        ]
    }
}

#[derive(Serialize, DeserializeFromStr, Debug, Clone)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Palette {
    #[default]
    Pastel,
    Vibrant,
    Warm,
    Cool,
}

impl FromStr for Palette {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pastel" => Ok(Self::Pastel),
            "vibrant" => Ok(Self::Vibrant),
            "warm" => Ok(Self::Warm),
            "cool" => Ok(Self::Cool),
            _ => bail!("Palette '{}' cannot be deserialized", s),
        }
    }
}

#[derive(Serialize, DeserializeFromStr, Debug, Clone)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,

    #[allow(dead_code)]
    Custom,
}

impl FromStr for Theme {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dark" => Ok(Self::Dark),
            "light" => Ok(Self::Light),
            _ => bail!("Theme '{}' cannot be deserialized", s),
        }
    }
}

#[derive(Serialize, DeserializeFromStr, Debug, Clone)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum CursorType {
    #[default]
    User,
    Line,
    Block,
    UnderScore,
}

impl FromStr for CursorType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(Self::User),
            "line" => Ok(Self::Line),
            "underscore" => Ok(Self::UnderScore),
            "block" => Ok(Self::Block),
            _ => bail!("Cursor type of '{}' cannot be deserialized", s),
        }
    }
}

#[derive(Serialize, DeserializeFromStr, Debug, Clone)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Border {
    #[default]
    Plain,
    Rounded,
    Double,
    Thick,
}

impl FromStr for Border {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "plain" => Ok(Self::Plain),
            "rounded" => Ok(Self::Rounded),
            "double" => Ok(Self::Double),
            "thick" => Ok(Self::Thick),
            _ => bail!("Border '{}' cannot be deserialized", s),
        }
    }
}

impl From<Border> for BorderType {
    fn from(val: Border) -> Self {
        match val {
            Border::Plain => Self::Plain,
            Border::Rounded => Self::Rounded,
            Border::Double => Self::Double,
            Border::Thick => Self::Thick,
        }
    }
}
