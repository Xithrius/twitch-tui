use std::{
    cell::RefCell,
    env, fmt,
    fs::{File, create_dir_all, read_to_string},
    io::Write,
    path::Path,
    rc::Rc,
    str::FromStr,
    vec,
};

use color_eyre::eyre::{Error, Result, bail};
use serde::{Deserialize, Serialize};
use serde_with::DeserializeFromStr;
use tokio::{runtime::Handle, task};
use tui::widgets::BorderType;

use crate::{
    emotes::support_graphics_protocol,
    handlers::{
        args::{Cli, merge_args_into_config},
        interactive::interactive_config,
        state::State,
        user_input::events::Key,
    },
    utils::pathing::{cache_path, config_path},
};

pub type SharedCoreConfig = Rc<RefCell<CoreConfig>>;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct CoreConfig {
    /// Connecting to Twitch.
    pub twitch: TwitchConfig,
    /// Internal functionality.
    pub terminal: TerminalConfig,
    /// If anything should be recorded for future use.
    pub storage: StorageConfig,
    /// Filtering out messages.
    pub filters: FiltersConfig,
    /// How everything looks to the user.
    pub frontend: FrontendConfig,

    pub keybinds: KeybindsConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct TwitchConfig {
    /// The current user's name.
    pub username: String,
    /// The streamer's channel name.
    pub channel: String,
    /// The websocket server to connect to.
    pub server: String,
    /// The authentication token for the websocket server.
    pub token: Option<String>,
    /// Keepalive timeout
    pub keepalive_timeout_seconds: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct TerminalConfig {
    /// The delay in milliseconds between terminal updates.
    pub delay: u64,
    /// The maximum amount of messages before truncation.
    pub maximum_messages: usize,
    /// The file path to log to.
    pub log_file: Option<String>,
    /// Which log level the tracing library should be set to.
    pub log_level: LogLevel,
    /// What state the application should start in.
    pub first_state: State,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct StorageConfig {
    /// If previous channels switched to should be tracked.
    pub channels: bool,
    /// If previous username mentions should be tracked.
    pub mentions: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct FiltersConfig {
    /// If filters should be enabled at all.
    pub enabled: bool,
    /// If the regex filters should be reversed.
    pub reversed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct FrontendConfig {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct KeybindsConfig {
    pub toggle_debug_focus: Vec<Key>,
    pub dashboard: DashboardKeybindsConfig,
    pub normal: NormalKeybindsConfig,
    pub insert: InsertKeybindsConfig,
    pub selection: SelectionKeybindsConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct DashboardKeybindsConfig {
    pub join: Vec<Key>,
    pub help: Vec<Key>,
    pub quit: Vec<Key>,
    pub recent_channels_search: Vec<Key>,
    pub followed_channels_search: Vec<Key>,
    pub crash_application: Vec<Key>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct NormalKeybindsConfig {
    pub enter_insert: Vec<Key>,
    pub enter_insert_with_mention: Vec<Key>,
    pub enter_insert_with_command: Vec<Key>,
    pub enter_dashboard: Vec<Key>,
    pub search_messages: Vec<Key>,
    pub toggle_message_filter: Vec<Key>,
    pub reverse_message_filter: Vec<Key>,
    pub back_to_previous_window: Vec<Key>,
    pub scroll_down: Vec<Key>,
    pub scroll_up: Vec<Key>,
    pub scroll_to_end: Vec<Key>,
    pub scroll_to_start: Vec<Key>,
    pub open_in_browser: Vec<Key>,
    pub help: Vec<Key>,
    pub quit: Vec<Key>,
    pub recent_channels_search: Vec<Key>,
    pub followed_channels_search: Vec<Key>,
    pub crash_application: Vec<Key>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct InsertKeybindsConfig {
    pub fill_suggestion: Vec<Key>,
    pub confirm_text_input: Vec<Key>,
    pub back_to_previous_window: Vec<Key>,
    pub move_cursor_right: Vec<Key>,
    pub move_cursor_left: Vec<Key>,
    pub move_cursor_start: Vec<Key>,
    pub move_cursor_end: Vec<Key>,
    pub swap_previous_item_with_current: Vec<Key>,
    pub remove_after_cursor: Vec<Key>,
    pub remove_before_cursor: Vec<Key>,
    pub remove_previous_word: Vec<Key>,
    pub remove_item_to_right: Vec<Key>,
    pub toggle_message_filter: Vec<Key>,
    pub reverse_message_filter: Vec<Key>,
    pub end_of_next_word: Vec<Key>,
    pub start_of_previous_word: Vec<Key>,
    pub swap_previous_word_with_current: Vec<Key>,
    pub toggle_emote_picker: Vec<Key>,
    pub quit: Vec<Key>,
    pub crash_application: Vec<Key>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct SelectionKeybindsConfig {
    pub scroll_down: Vec<Key>,
    pub scroll_up: Vec<Key>,
    pub delete_entry: Vec<Key>,
    pub select: Vec<Key>,
    pub back_to_previous_window: Vec<Key>,
    pub crash_application: Vec<Key>,
}

impl Default for TwitchConfig {
    fn default() -> Self {
        Self {
            username: String::new(),
            channel: String::new(),
            server: "wss://eventsub.wss.twitch.tv/ws".to_string(),
            token: None,
            keepalive_timeout_seconds: 30,
        }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            delay: 30,
            maximum_messages: 500,
            log_file: None,
            log_level: LogLevel::INFO,
            first_state: State::default(),
        }
    }
}

impl Default for FrontendConfig {
    fn default() -> Self {
        Self {
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

impl Default for KeybindsConfig {
    fn default() -> Self {
        Self {
            toggle_debug_focus: vec![Key::Ctrl('d')],
            dashboard: DashboardKeybindsConfig::default(),
            normal: NormalKeybindsConfig::default(),
            insert: InsertKeybindsConfig::default(),
            selection: SelectionKeybindsConfig::default(),
        }
    }
}

impl Default for DashboardKeybindsConfig {
    fn default() -> Self {
        Self {
            join: vec![Key::Enter],
            help: vec![Key::Char('?'), Key::Char('h')],
            quit: vec![Key::Char('q')],
            recent_channels_search: vec![Key::Char('s')],
            followed_channels_search: vec![Key::Char('f')],
            crash_application: vec![Key::Ctrl('p')],
        }
    }
}
impl Default for NormalKeybindsConfig {
    fn default() -> Self {
        Self {
            enter_insert: vec![Key::Char('i'), Key::Char('c')],
            enter_insert_with_mention: vec![Key::Char('@')],
            enter_insert_with_command: vec![Key::Char('/')],
            enter_dashboard: vec![Key::Char('S')],
            search_messages: vec![Key::Ctrl('f')],
            toggle_message_filter: vec![Key::Ctrl('t')],
            reverse_message_filter: vec![Key::Ctrl('r')],
            back_to_previous_window: vec![Key::Esc],
            scroll_up: vec![Key::ScrollUp, Key::Up, Key::Char('k')],
            scroll_down: vec![Key::ScrollDown, Key::Down, Key::Char('j')],
            scroll_to_end: vec![Key::Char('G')],
            scroll_to_start: vec![Key::Char('g')],
            open_in_browser: vec![Key::Char('o')],
            help: vec![Key::Char('?'), Key::Char('h')],
            quit: vec![Key::Char('q')],
            recent_channels_search: vec![Key::Char('s')],
            followed_channels_search: vec![Key::Char('f')],
            crash_application: vec![Key::Ctrl('p')],
        }
    }
}

impl Default for InsertKeybindsConfig {
    fn default() -> Self {
        Self {
            fill_suggestion: vec![Key::Tab],
            confirm_text_input: vec![Key::Enter],
            back_to_previous_window: vec![Key::Esc],
            move_cursor_right: vec![Key::Right, Key::Ctrl('f')],
            move_cursor_left: vec![Key::Left, Key::Ctrl('b')],
            move_cursor_start: vec![Key::Home, Key::Ctrl('a')],
            move_cursor_end: vec![Key::End, Key::Ctrl('e')],
            swap_previous_item_with_current: vec![Key::Ctrl('t')],
            remove_after_cursor: vec![Key::Ctrl('k')],
            remove_before_cursor: vec![Key::Ctrl('u')],
            remove_previous_word: vec![Key::Ctrl('w')],
            remove_item_to_right: vec![Key::Delete, Key::Ctrl('d')],
            toggle_message_filter: vec![Key::Ctrl('t')],
            reverse_message_filter: vec![Key::Ctrl('r')],
            end_of_next_word: vec![Key::Alt('f')],
            start_of_previous_word: vec![Key::Alt('b')],
            swap_previous_word_with_current: vec![Key::Alt('t')],
            toggle_emote_picker: vec![Key::Ctrl('e')],
            quit: vec![Key::Ctrl('q')],
            crash_application: vec![Key::Ctrl('p')],
        }
    }
}

impl Default for SelectionKeybindsConfig {
    fn default() -> Self {
        Self {
            scroll_up: vec![Key::ScrollUp, Key::Up],
            scroll_down: vec![Key::ScrollDown, Key::Down],
            select: vec![Key::Enter],
            delete_entry: vec![Key::Ctrl('d')],
            back_to_previous_window: vec![Key::Esc],
            crash_application: vec![Key::Enter],
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::DEBUG => "debug",
                Self::INFO => "info",
                Self::WARN => "warn",
                Self::ERROR => "error",
            }
        )
    }
}

#[derive(Serialize, DeserializeFromStr, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Palette {
    Pastel,
    Vibrant,
    Warm,
    Cool,
}

impl Default for Palette {
    fn default() -> Self {
        Self::Pastel
    }
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
pub enum Theme {
    Dark,
    Light,

    #[allow(dead_code)]
    Custom,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Dark
    }
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
pub enum CursorType {
    User,
    Line,
    Block,
    UnderScore,
}

impl Default for CursorType {
    fn default() -> Self {
        Self::User
    }
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
pub enum Border {
    Plain,
    Rounded,
    Double,
    Thick,
}

impl Default for Border {
    fn default() -> Self {
        Self::Plain
    }
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

pub trait ToVec<T> {
    fn to_vec(&self) -> Vec<T>;
}

impl ToVec<(String, String)> for TwitchConfig {
    fn to_vec(&self) -> Vec<(String, String)> {
        vec![
            ("Username".to_string(), self.username.to_string()),
            ("Channel".to_string(), self.channel.to_string()),
            ("Server".to_string(), self.server.to_string()),
        ]
    }
}

impl ToVec<(String, String)> for TerminalConfig {
    fn to_vec(&self) -> Vec<(String, String)> {
        vec![
            ("Current channel".to_string(), self.delay.to_string()),
            (
                "Max messages".to_string(),
                self.maximum_messages.to_string(),
            ),
            (
                "Log file".to_string(),
                self.log_file
                    .clone()
                    .map_or_else(|| "None".to_string(), |f| f),
            ),
            ("First state".to_string(), self.first_state.to_string()),
        ]
    }
}

impl ToVec<(String, String)> for StorageConfig {
    fn to_vec(&self) -> Vec<(String, String)> {
        vec![
            ("Channels enabled".to_string(), self.channels.to_string()),
            ("Mentions enabled".to_string(), self.mentions.to_string()),
        ]
    }
}

impl ToVec<(String, String)> for FiltersConfig {
    fn to_vec(&self) -> Vec<(String, String)> {
        vec![
            ("Enabled".to_string(), self.enabled.to_string()),
            ("Reversed".to_string(), self.reversed.to_string()),
        ]
    }
}

impl ToVec<(String, String)> for FrontendConfig {
    fn to_vec(&self) -> Vec<(String, String)> {
        vec![
            (
                "Show datetimes".to_string(),
                self.show_datetimes.to_string(),
            ),
            (
                "Datetime format".to_string(),
                self.datetime_format.to_string(),
            ),
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

impl TwitchConfig {
    #[must_use]
    pub fn config_twitch_websocket_url(&self) -> String {
        format!(
            "{}?keepalive_timeout_seconds={}",
            self.server, self.keepalive_timeout_seconds
        )
    }
}

fn persist_config(path: &Path, config: &CoreConfig) -> Result<()> {
    let toml_string = toml::to_string(&config)?;
    let mut file = File::create(path)?;

    file.write_all(toml_string.as_bytes())?;
    drop(file);

    Ok(())
}

const RAW_DEFAULT_CONFIG_URL: &str =
    "https://raw.githubusercontent.com/Xithrius/twitch-tui/main/default-config.toml";

fn persist_default_config(path: &Path) {
    let default_config = task::block_in_place(move || {
        Handle::current().block_on(async move {
            reqwest::get(RAW_DEFAULT_CONFIG_URL)
                .await
                .unwrap()
                .text()
                .await
                .unwrap()
        })
    });

    let mut file = File::create(path).unwrap();

    file.write_all(default_config.as_bytes()).unwrap();
    drop(file);
}

impl FrontendConfig {
    pub const fn is_emotes_enabled(&self) -> bool {
        self.twitch_emotes
            || self.betterttv_emotes
            || self.seventv_emotes
            || self.frankerfacez_emotes
    }
}

impl CoreConfig {
    pub fn new(cli: Cli) -> Result<Self, Error> {
        let path_str = cache_path("");

        let p = Path::new(&path_str);
        if !p.exists() {
            create_dir_all(p).unwrap();
        }

        let path_str = config_path("config.toml");

        let p = Path::new(&path_str);

        if !p.exists() {
            create_dir_all(p.parent().unwrap()).unwrap();

            if let Some(config) = interactive_config() {
                persist_config(p, &config)?;
                Ok(config)
            } else {
                persist_default_config(p);
                bail!(
                    "Default configuration was generated at {path_str}, please fill it out with necessary information."
                )
            }
        } else if let Ok(file_content) = read_to_string(p) {
            let mut config: Self = match toml::from_str(&file_content) {
                Ok(c) => c,
                Err(err) => bail!("Config could not be processed. Error: {:?}", err.message()),
            };

            merge_args_into_config(&mut config, cli);

            let token = env::var("TWT_TOKEN").ok();
            if let Some(env_token) = token {
                if !env_token.is_empty() {
                    config.twitch.token = Some(env_token);
                }
            }

            {
                let t = &config.twitch;

                let check_token = t.token.as_ref().map_or("", |t| t);

                if t.username.is_empty() || t.channel.is_empty() || check_token.is_empty() {
                    bail!(
                        "Twitch config section is missing one or more of the following: username, channel, token."
                    );
                }

                if config.frontend.is_emotes_enabled()
                    && !support_graphics_protocol().unwrap_or(false)
                {
                    eprintln!(
                        "This terminal does not support the graphics protocol.\nUse a terminal such as kitty, or disable emotes."
                    );
                    std::process::exit(1);
                }
            }

            // Channel names for the websocket connection can only be in lowercase.
            config.twitch.channel = config.twitch.channel.to_lowercase();

            Ok(config)
        } else {
            bail!(
                "Configuration could not be read correctly. See the following link for the example config: {}",
                format!(
                    "{}/blob/main/default-config.toml",
                    env!("CARGO_PKG_REPOSITORY")
                )
            )
        }
    }
}
