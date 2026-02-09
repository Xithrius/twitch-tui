use std::vec;

use serde::{Deserialize, Serialize};

use crate::events::Key;

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
    pub recent_channels_search: Vec<Key>,
    pub followed_channels_search: Vec<Key>,
    pub help: Vec<Key>,
    pub quit: Vec<Key>,
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
    pub open_in_player: Vec<Key>,
    pub recent_channels_search: Vec<Key>,
    pub followed_channels_search: Vec<Key>,
    pub help: Vec<Key>,
    pub quit: Vec<Key>,
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct SelectionKeybindsConfig {
    pub next_item: Vec<Key>,
    pub prev_item: Vec<Key>,
    pub delete_item: Vec<Key>,
    pub select: Vec<Key>,
    pub back_to_previous_window: Vec<Key>,
    pub quit: Vec<Key>,
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
            recent_channels_search: vec![Key::Char('s')],
            followed_channels_search: vec![Key::Char('f')],
            help: vec![Key::Char('?'), Key::Char('h')],
            quit: vec![Key::Char('q')],
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
            open_in_player: vec![Key::Char('o')],
            recent_channels_search: vec![Key::Char('s')],
            followed_channels_search: vec![Key::Char('f')],
            help: vec![Key::Char('?'), Key::Char('h')],
            quit: vec![Key::Char('q')],
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
            toggle_emote_picker: vec![Key::Alt('e')],
            quit: vec![Key::Ctrl('q')],
        }
    }
}

impl Default for SelectionKeybindsConfig {
    fn default() -> Self {
        Self {
            prev_item: vec![Key::ScrollUp, Key::Up],
            next_item: vec![Key::ScrollDown, Key::Down],
            select: vec![Key::Enter],
            delete_item: vec![Key::Ctrl('d')],
            back_to_previous_window: vec![Key::Esc],
            quit: vec![Key::Char('q')],
        }
    }
}
