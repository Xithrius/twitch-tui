use serde::{Deserialize, Serialize};

use crate::events::Key;

type Keybind = Box<[Key]>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct KeybindsConfig {
    pub toggle_debug_focus: Keybind,
    pub dashboard: DashboardKeybindsConfig,
    pub normal: NormalKeybindsConfig,
    pub insert: InsertKeybindsConfig,
    pub selection: SelectionKeybindsConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct DashboardKeybindsConfig {
    pub join: Keybind,
    pub recent_channels_search: Keybind,
    pub followed_channels_search: Keybind,
    pub help: Keybind,
    pub quit: Keybind,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct NormalKeybindsConfig {
    pub enter_insert: Keybind,
    pub enter_insert_with_mention: Keybind,
    pub enter_insert_with_command: Keybind,
    pub enter_dashboard: Keybind,
    pub search_messages: Keybind,
    pub toggle_filters: Keybind,
    pub reverse_filters: Keybind,
    pub back_to_previous_window: Keybind,
    pub scroll_down: Keybind,
    pub scroll_up: Keybind,
    pub scroll_to_end: Keybind,
    pub scroll_to_start: Keybind,
    pub open_in_player: Keybind,
    pub recent_channels_search: Keybind,
    pub followed_channels_search: Keybind,
    pub help: Keybind,
    pub quit: Keybind,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct InsertKeybindsConfig {
    pub fill_suggestion: Keybind,
    pub confirm_text_input: Keybind,
    pub back_to_previous_window: Keybind,
    pub move_cursor_right: Keybind,
    pub move_cursor_left: Keybind,
    pub move_cursor_start: Keybind,
    pub move_cursor_end: Keybind,
    pub swap_previous_item_with_current: Keybind,
    pub remove_after_cursor: Keybind,
    pub remove_before_cursor: Keybind,
    pub remove_previous_word: Keybind,
    pub remove_item_to_right: Keybind,
    pub toggle_filters: Keybind,
    pub reverse_filters: Keybind,
    pub end_of_next_word: Keybind,
    pub start_of_previous_word: Keybind,
    pub swap_previous_word_with_current: Keybind,
    pub toggle_emote_picker: Keybind,
    pub quit: Keybind,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct SelectionKeybindsConfig {
    pub next_item: Keybind,
    pub prev_item: Keybind,
    pub delete_item: Keybind,
    pub select: Keybind,
    pub back_to_previous_window: Keybind,
    pub quit: Keybind,
}

impl Default for KeybindsConfig {
    fn default() -> Self {
        Self {
            toggle_debug_focus: Box::new([Key::Ctrl('d')]),
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
            join: Box::new([Key::Enter]),
            recent_channels_search: Box::new([Key::Char('s')]),
            followed_channels_search: Box::new([Key::Char('f')]),
            help: Box::new([Key::Char('?'), Key::Char('h')]),
            quit: Box::new([Key::Char('q')]),
        }
    }
}
impl Default for NormalKeybindsConfig {
    fn default() -> Self {
        Self {
            enter_insert: Box::new([Key::Char('i'), Key::Char('c')]),
            enter_insert_with_mention: Box::new([Key::Char('@')]),
            enter_insert_with_command: Box::new([Key::Char('/')]),
            enter_dashboard: Box::new([Key::Char('S')]),
            search_messages: Box::new([Key::Ctrl('f')]),
            toggle_filters: Box::new([Key::Ctrl('t')]),
            reverse_filters: Box::new([Key::Ctrl('r')]),
            back_to_previous_window: Box::new([Key::Esc]),
            scroll_up: Box::new([Key::ScrollUp, Key::Up, Key::Char('k')]),
            scroll_down: Box::new([Key::ScrollDown, Key::Down, Key::Char('j')]),
            scroll_to_end: Box::new([Key::Char('G')]),
            scroll_to_start: Box::new([Key::Char('g')]),
            open_in_player: Box::new([Key::Char('o')]),
            recent_channels_search: Box::new([Key::Char('s')]),
            followed_channels_search: Box::new([Key::Char('f')]),
            help: Box::new([Key::Char('?'), Key::Char('h')]),
            quit: Box::new([Key::Char('q')]),
        }
    }
}

impl Default for InsertKeybindsConfig {
    fn default() -> Self {
        Self {
            fill_suggestion: Box::new([Key::Tab]),
            confirm_text_input: Box::new([Key::Enter]),
            back_to_previous_window: Box::new([Key::Esc]),
            move_cursor_right: Box::new([Key::Right, Key::Ctrl('f')]),
            move_cursor_left: Box::new([Key::Left, Key::Ctrl('b')]),
            move_cursor_start: Box::new([Key::Home, Key::Ctrl('a')]),
            move_cursor_end: Box::new([Key::End, Key::Ctrl('e')]),
            swap_previous_item_with_current: Box::new([Key::Ctrl('t')]),
            remove_after_cursor: Box::new([Key::Ctrl('k')]),
            remove_before_cursor: Box::new([Key::Ctrl('u')]),
            remove_previous_word: Box::new([Key::Ctrl('w')]),
            remove_item_to_right: Box::new([Key::Delete, Key::Ctrl('d')]),
            toggle_filters: Box::new([Key::Ctrl('t')]),
            reverse_filters: Box::new([Key::Ctrl('r')]),
            end_of_next_word: Box::new([Key::Alt('f')]),
            start_of_previous_word: Box::new([Key::Alt('b')]),
            swap_previous_word_with_current: Box::new([Key::Alt('t')]),
            toggle_emote_picker: Box::new([Key::Alt('e')]),
            quit: Box::new([Key::Ctrl('q')]),
        }
    }
}

impl Default for SelectionKeybindsConfig {
    fn default() -> Self {
        Self {
            prev_item: Box::new([Key::ScrollUp, Key::Up]),
            next_item: Box::new([Key::ScrollDown, Key::Down]),
            select: Box::new([Key::Enter]),
            delete_item: Box::new([Key::Ctrl('d')]),
            back_to_previous_window: Box::new([Key::Esc]),
            quit: Box::new([Key::Char('q')]),
        }
    }
}
