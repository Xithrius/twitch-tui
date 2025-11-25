use tui::{
    Frame,
    layout::{Constraint, Rect},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::{
    handlers::{
        config::SharedCoreConfig,
        user_input::events::{Event, get_keybind_text},
    },
    terminal::TerminalAction,
    ui::{components::Component, statics::HELP_COLUMN_TITLES},
    utils::styles::{BOLD_STYLE, COLUMN_TITLE_STYLE},
};

// Once a solution is found to calculate constraints, this will be removed.
const TABLE_CONSTRAINTS: [Constraint; 3] =
    [Constraint::Min(11), Constraint::Min(8), Constraint::Min(38)];

#[derive(Debug, Clone)]
pub struct HelpWidget {
    config: SharedCoreConfig,
}

impl HelpWidget {
    pub const fn new(config: SharedCoreConfig) -> Self {
        Self { config }
    }
    fn get_help_keybinds(&self) -> Vec<(&'static str, Vec<(String, &'static str)>)> {
        let keybinds = self.config.borrow().keybinds.clone();
        let dashboard_keybinds = vec![
            (
                get_keybind_text(&keybinds.dashboard.join),
                "Join the selected channel",
            ),
            (
                get_keybind_text(&keybinds.dashboard.recent_channels_search),
                "Open the recent channel search popup",
            ),
            (
                get_keybind_text(&keybinds.dashboard.followed_channels_search),
                "Open the followed channel search popup",
            ),
            (
                get_keybind_text(&keybinds.dashboard.help),
                "Have the keybinds popup window appear",
            ),
            (
                get_keybind_text(&keybinds.dashboard.quit),
                "Quit the application",
            ),
            (
                get_keybind_text(&keybinds.dashboard.crash_application),
                "Manually crash the application",
            ),
        ];
        let normal_keybinds = vec![
            (
                get_keybind_text(&keybinds.normal.enter_insert),
                "Enter message (chat) mode for sending messages",
            ),
            (
                get_keybind_text(&keybinds.normal.enter_insert_with_mention),
                "Messaging mode with mention symbol",
            ),
            (
                get_keybind_text(&keybinds.normal.enter_insert_with_command),
                "Messaging mode with command symbol",
            ),
            (
                get_keybind_text(&keybinds.normal.enter_dashboard),
                "Go to the dashboard screen (start screen)",
            ),
            (
                get_keybind_text(&keybinds.normal.search_messages),
                "Search messages",
            ),
            (
                get_keybind_text(&keybinds.normal.toggle_message_filter),
                "Toggle the message filter",
            ),
            (
                get_keybind_text(&keybinds.normal.reverse_message_filter),
                "Reverse the message filter",
            ),
            (
                get_keybind_text(&keybinds.normal.back_to_previous_window),
                "Go back to the previous window",
            ),
            (
                get_keybind_text(&keybinds.normal.scroll_down),
                "Scroll chat down",
            ),
            (
                get_keybind_text(&keybinds.normal.scroll_up),
                "Scroll chat up",
            ),
            (
                get_keybind_text(&keybinds.normal.scroll_to_start),
                "Scroll chat to top",
            ),
            (
                get_keybind_text(&keybinds.normal.scroll_to_end),
                "Scroll chat to bottom",
            ),
            (
                get_keybind_text(&keybinds.normal.open_in_player),
                "Open current stream in player",
            ),
            (
                get_keybind_text(&keybinds.normal.recent_channels_search),
                "Open the recent channel search widget",
            ),
            (
                get_keybind_text(&keybinds.normal.followed_channels_search),
                "Open the followed channel search widget",
            ),
            (get_keybind_text(&keybinds.normal.help), "* You are here!"),
            (
                get_keybind_text(&keybinds.normal.quit),
                "Quit the application",
            ),
            (
                get_keybind_text(&keybinds.normal.crash_application),
                "Manually crash the application",
            ),
        ];
        let insert_keybinds = vec![
            (
                get_keybind_text(&keybinds.insert.fill_suggestion),
                "Fill in suggestion, if available",
            ),
            (
                get_keybind_text(&keybinds.insert.confirm_text_input),
                "Confirm the input text to go through",
            ),
            (
                get_keybind_text(&keybinds.insert.back_to_previous_window),
                "Go back to the previous window",
            ),
            (
                get_keybind_text(&keybinds.insert.move_cursor_right),
                "Move cursor to the right",
            ),
            (
                get_keybind_text(&keybinds.insert.move_cursor_left),
                "Move cursor to the left",
            ),
            (
                get_keybind_text(&keybinds.insert.move_cursor_start),
                "Move cursor to the start",
            ),
            (
                get_keybind_text(&keybinds.insert.move_cursor_end),
                "Move cursor to the end",
            ),
            (
                get_keybind_text(&keybinds.insert.swap_previous_item_with_current),
                "Swap previous item with current item",
            ),
            (
                get_keybind_text(&keybinds.insert.remove_after_cursor),
                "Remove everything after the cursor",
            ),
            (
                get_keybind_text(&keybinds.insert.remove_before_cursor),
                "Remove everything before the cursor",
            ),
            (
                get_keybind_text(&keybinds.insert.remove_previous_word),
                "Remove the previous word",
            ),
            (
                get_keybind_text(&keybinds.insert.remove_item_to_right),
                "Remove item to the right",
            ),
            (
                get_keybind_text(&keybinds.insert.toggle_message_filter),
                "Toggle the filter",
            ),
            (
                get_keybind_text(&keybinds.insert.reverse_message_filter),
                "Reverse the filter",
            ),
            (
                get_keybind_text(&keybinds.insert.end_of_next_word),
                "Move to the end of the next word",
            ),
            (
                get_keybind_text(&keybinds.insert.start_of_previous_word),
                "Move to the start of the previous word",
            ),
            (
                get_keybind_text(&keybinds.insert.swap_previous_word_with_current),
                "Swap previous word with current word",
            ),
            (
                get_keybind_text(&keybinds.insert.toggle_emote_picker),
                "Toggle emote picker",
            ),
            (
                get_keybind_text(&keybinds.insert.quit),
                "Quit the application",
            ),
            (
                get_keybind_text(&keybinds.insert.crash_application),
                "Manually crash the application",
            ),
        ];
        let selection_keybinds = vec![
            (
                get_keybind_text(&keybinds.selection.next_item),
                "Move selection to next item",
            ),
            (
                get_keybind_text(&keybinds.selection.prev_item),
                "Move selection to previous item",
            ),
            (
                get_keybind_text(&keybinds.selection.delete_item),
                "Delete the currently selected item",
            ),
            (
                get_keybind_text(&keybinds.selection.select),
                "Confirm the currently selected item",
            ),
            (
                get_keybind_text(&keybinds.selection.back_to_previous_window),
                "Go back to the previous window",
            ),
            (
                get_keybind_text(&keybinds.selection.crash_application),
                "Manually crash the application",
            ),
        ];
        vec![
            ("Dashboard", dashboard_keybinds),
            ("Normal mode", normal_keybinds),
            ("Insert modes", insert_keybinds),
            ("Selections", selection_keybinds),
        ]
    }
}

impl Component for HelpWidget {
    fn draw(&mut self, f: &mut Frame, area: Option<Rect>) {
        let r = area.map_or_else(|| f.area(), |a| a);

        let mut rows = vec![];

        for (s, v) in &self.get_help_keybinds() {
            for (i, (key, desc)) in v.iter().enumerate() {
                rows.push(Row::new(vec![
                    if i == 0 {
                        Cell::from((*s).to_string())
                    } else {
                        Cell::from("")
                    }
                    .style(*BOLD_STYLE),
                    Cell::from((*key).clone()),
                    Cell::from((*desc).to_string()),
                ]));
            }

            rows.push(Row::new(vec![Cell::from("")]));
        }

        let help_table = Table::new(rows, TABLE_CONSTRAINTS)
            .header(Row::new(HELP_COLUMN_TITLES.iter().copied()).style(*COLUMN_TITLE_STYLE))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("[ Keybinds ]")
                    .border_type(self.config.borrow().frontend.border_type.clone().into()),
            )
            .column_spacing(2);

        f.render_widget(help_table, r);
    }
    //TODO should be default impl if not for the config requirement
    async fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            let keybinds = self.config.borrow().keybinds.selection.clone();
            match key {
                key if keybinds.quit.contains(key) => return Some(TerminalAction::Quit),
                key if keybinds.back_to_previous_window.contains(key) => {
                    return Some(TerminalAction::BackOneLayer);
                }
                key if keybinds.crash_application.contains(key) => {
                    panic!("Manual panic triggered by user.")
                }
                _ => {}
            }
        }

        None
    }
}
