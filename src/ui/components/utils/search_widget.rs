use std::iter::Iterator;

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

pub trait ItemGetter<T>
where
    T: Default,
{
    fn get_items(&mut self) -> T;
}

pub struct SearchWidget<T: Default, F> {
    config: SharedCompleteConfig,
    focused: bool,

    item_getter: F,
    items: T,
    filtered_items: Option<T>,

    list_state: ListState,
    search_input: InputWidget,
    vertical_scroll_state: ScrollbarState,
    vertical_scroll: usize,
}

impl<T, F> SearchWidget<T, F>
where
    T: Default + Iterator + Copy,
    F: ItemGetter<T>,
{
    pub fn new(config: SharedCompleteConfig, item_getter: F) -> Self {
        let search_input = InputWidget::new(config.clone(), "Search", None, None, None);

        Self {
            config,
            focused: false,
            item_getter,
            items: T::default(),
            filtered_items: None,
            list_state: ListState::default(),
            search_input,
            vertical_scroll_state: ScrollbarState::default(),
            vertical_scroll: 0,
        }
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if let Some(filtered) = &self.filtered_items {
                    if i >= filtered.count().saturating_sub(1) {
                        filtered.count().saturating_sub(1)
                    } else {
                        i + 1
                    }
                } else if i >= self.items.count() - 1 {
                    self.items.count() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.list_state.select(Some(i));

        self.vertical_scroll = self.vertical_scroll.saturating_add(1);
        self.vertical_scroll_state = self
            .vertical_scroll_state
            .position(self.vertical_scroll as u16);
    }

    fn previous(&mut self) {
        let i = self
            .list_state
            .selected()
            .map_or(0, |i| if i == 0 { 0 } else { i - 1 });
        self.list_state.select(Some(i));

        self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
        self.vertical_scroll_state = self
            .vertical_scroll_state
            .position(self.vertical_scroll as u16);
    }

    fn unselect(&mut self) {
        self.list_state.select(None);
    }

    pub const fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn toggle_focus(&mut self) {
        if !self.focused {
            self.items = self.item_getter.get_items();
        }

        self.focused = !self.focused;
    }
}

impl<T, F> Component for SearchWidget<T, F>
where
    T: Default + Iterator + Copy,
    F: ItemGetter<T>,
{
    fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        area: Option<Rect>,
        emotes: Option<&mut Emotes>,
    ) {
        todo!()
    }

    fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            match key {
                Key::Esc => {
                    if self.list_state.selected().is_some() {
                        self.unselect();
                    } else {
                        self.toggle_focus();
                    }
                }
                Key::ScrollDown => self.next(),
                Key::ScrollUp => self.previous(),
                _ => {
                    self.search_input.event(event);

                    // Assuming that the user inputted something that modified the input
                    if let Some(v) = &self.filtered_items {
                        if v.count() > 0 {
                            self.list_state.select(Some(0));
                        }
                    }
                }
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
