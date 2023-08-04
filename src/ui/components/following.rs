use std::ops::Index;

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use once_cell::sync::Lazy;
use tokio::{runtime::Handle, task};
use tui::{
    backend::Backend,
    layout::Rect,
    prelude::{Alignment, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        block::Position, scrollbar, Block, Borders, Clear, List, ListItem, ListState, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};

use crate::{
    emotes::Emotes,
    handlers::{
        config::SharedCompleteConfig,
        user_input::events::{Event, Key},
    },
    terminal::TerminalAction,
    twitch::{channels::FollowingList, TwitchAction},
    ui::components::Component,
    utils::text::{title_line, TitleStyle},
};

use super::utils::{centered_rect, InputWidget};

static FUZZY_FINDER: Lazy<SkimMatcherV2> = Lazy::new(SkimMatcherV2::default);

pub struct FollowingWidget {
    config: SharedCompleteConfig,
    focused: bool,
    channels: FollowingList,
    filtered_channels: Option<Vec<String>>,
    list_state: ListState,
    search_input: InputWidget,
    vertical_scroll_state: ScrollbarState,
    vertical_scroll: usize,
}

impl FollowingWidget {
    pub fn new(config: SharedCompleteConfig) -> Self {
        let search_input = InputWidget::new(config.clone(), "Search", None, None, None);

        // let channels = get_followed_channels(config.borrow().twitch.clone());
        let channels = FollowingList::default();

        Self {
            config,
            focused: false,
            channels,
            list_state: ListState::default().with_selected(Some(0)),
            filtered_channels: None,
            search_input,
            vertical_scroll_state: ScrollbarState::default(),
            vertical_scroll: 0,
        }
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if let Some(filtered) = &self.filtered_channels {
                    if i >= filtered.len().saturating_sub(1) {
                        filtered.len().saturating_sub(1)
                    } else {
                        i + 1
                    }
                } else if i >= self.channels.data.len() - 1 {
                    self.channels.data.len() - 1
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
            FollowingList::get_followed_channels(self.config.borrow().twitch.clone());
        }

        self.focused = !self.focused;
    }
}

impl Component for FollowingWidget {
    fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        area: Option<Rect>,
        emotes: Option<&mut Emotes>,
    ) {
        let r = area.map_or_else(|| centered_rect(60, 60, 20, f.size()), |a| a);

        let mut items = vec![];
        let current_input = self.search_input.to_string();

        if current_input.is_empty() {
            for channel in self.channels.clone().data {
                items.push(ListItem::new(channel.broadcaster_name.clone()));
            }

            self.filtered_channels = None;
        } else {
            let channel_filter = |c: String| -> Vec<usize> {
                FUZZY_FINDER
                    .fuzzy_indices(&c, &current_input)
                    .map(|(_, indices)| indices)
                    .unwrap_or_default()
            };

            let mut matched = vec![];

            for channel in self.channels.clone().data {
                let matched_indices = channel_filter(channel.broadcaster_name.clone());

                if matched_indices.is_empty() {
                    continue;
                }

                let search_theme = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);

                let line = channel
                    .broadcaster_name
                    .chars()
                    .enumerate()
                    .map(|(i, c)| {
                        if matched_indices.contains(&i) {
                            Span::styled(c.to_string(), search_theme)
                        } else {
                            Span::raw(c.to_string())
                        }
                    })
                    .collect::<Vec<Span>>();

                items.push(ListItem::new(vec![Line::from(line)]));
                matched.push(channel.broadcaster_name);
            }

            self.filtered_channels = Some(matched);
        }

        let title_binding = [TitleStyle::Single("Following")];

        let list = List::new(items.clone())
            .block(
                Block::default()
                    .title(title_line(
                        &title_binding,
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_type(self.config.borrow().frontend.border_type.clone().into()),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(Clear, r);
        f.render_stateful_widget(list, r, &mut self.list_state);

        self.vertical_scroll_state = self
            .vertical_scroll_state
            .content_length(items.len() as u16);

        f.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .begin_symbol(None)
                .end_symbol(None),
            r.inner(&Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.vertical_scroll_state,
        );

        let title_binding = format!(
            "{} / {}",
            self.list_state.selected().map_or(1, |i| i + 1),
            if let Some(v) = &self.filtered_channels {
                v.len()
            } else {
                self.channels.data.len()
            }
        );

        let title = [TitleStyle::Single(&title_binding)];

        let bottom_block = Block::default()
            .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
            .border_type(self.config.borrow().frontend.border_type.clone().into())
            .title(title_line(&title, Style::default()))
            .title_position(Position::Bottom)
            .title_alignment(Alignment::Right);

        let rect = Rect::new(r.x, r.bottom() - 1, r.width, 1);

        f.render_widget(bottom_block, rect);

        let input_rect = Rect::new(rect.x, rect.bottom(), rect.width, 3);

        self.search_input.draw(f, Some(input_rect), emotes);
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
                Key::Enter => {
                    if let Some(i) = self.list_state.selected() {
                        let selected_channel = if let Some(v) = self.filtered_channels.clone() {
                            if v.is_empty() {
                                return None;
                            }

                            v.index(i).to_string()
                        } else {
                            self.channels.data.index(i).broadcaster_name.to_string()
                        }
                        .to_lowercase();

                        self.toggle_focus();

                        self.unselect();

                        self.config.borrow_mut().twitch.channel = selected_channel.clone();

                        return Some(TerminalAction::Enter(TwitchAction::Join(selected_channel)));
                    }
                }
                _ => {
                    self.search_input.event(event);

                    // Assuming that the user inputted something that modified the input
                    if let Some(v) = &self.filtered_channels {
                        if !v.is_empty() {
                            self.list_state.select(Some(0));
                        }
                    }
                }
            }
        }

        None
    }
}
