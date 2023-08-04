use rustyline::{line_buffer::LineBuffer, At, Word};
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{block::Position, Block, Borders, Clear, ListState, Paragraph, ScrollbarState},
    Frame,
};

use crate::{
    emotes::Emotes,
    handlers::{
        config::SharedCompleteConfig,
        storage::SharedStorage,
        user_input::events::{Event, Key},
    },
    terminal::TerminalAction,
    ui::{components::Component, statics::LINE_BUFFER_CAPACITY},
    utils::text::{get_cursor_position, title_line, TitleStyle},
};

use super::InputWidget;

pub struct SearchWidget<T: Default> {
    config: SharedCompleteConfig,
    focused: bool,
    items: T,
    filtered_items: Option<T>,

    list_state: ListState,
    search_input: InputWidget,
    vertical_scroll_state: ScrollbarState,
    vertical_scroll: usize,
}

impl<T> SearchWidget<T>
where
    T: Default,
{
    pub fn new(config: SharedCompleteConfig) -> Self {
        let search_input = InputWidget::new(config.clone(), "Search", None, None, None);

        Self {
            config,
            focused: false,
            items: T::default(),
            filtered_items: None,
            list_state: ListState::default(),
            search_input,
            vertical_scroll_state: ScrollbarState::default(),
            vertical_scroll: 0,
        }
    }
}

impl<T> Component for SearchWidget<T>
where
    T: Default,
{
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, emotes: Option<&mut Emotes>) {
        todo!()
    }

    fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            match key {
                Key::Char('q') => return Some(TerminalAction::Quit),
                Key::Esc => return Some(TerminalAction::BackOneLayer),
                Key::Ctrl('p') => panic!("Manual panic triggered by user."),
                _ => {}
            }
        }

        None
    }
}

// impl SearchWidget {
//     pub fn new(
//         config: SharedCompleteConfig,
//         title: &str,
//         input_validator: Option<InputValidator>,
//     ) -> Self {
//         let search_input = InputWidget::new(config.clone(), "Search", None, None, None);

//         Self {
//             config,
//             search_input,
//             title: title.to_string(),
//             focused: false,
//             input_validator,
//         }
//     }

//     // pub fn update(&mut self, s: &str) {
//     //     self.input.update(s, 0);
//     // }

//     // pub const fn is_focused(&self) -> bool {
//     //     self.focused
//     // }

//     // pub fn toggle_focus(&mut self) {
//     //     self.focused = !self.focused;
//     // }

//     // pub fn toggle_focus_with(&mut self, s: &str) {
//     //     self.focused = !self.focused;
//     //     self.input.update(s, 1);
//     // }

//     // pub fn is_valid(&self) -> bool {
//     //     self.input_validator
//     //         .as_ref()
//     //         .map_or(true, |validator| validator(self.input.to_string()))
//     // }
// }

// impl Component for SearchWidget {
//     fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, emotes: Option<&mut Emotes>) {
//         self.search_input.draw(f, area, emotes);
//     }

//     fn event(&mut self, event: &Event) -> Option<TerminalAction> {
//         self.search_input.event(event)
//     }
// }
