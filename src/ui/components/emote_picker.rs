use std::cmp::max;

use fuzzy_matcher::FuzzyMatcher;
use tracing::warn;
use tui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use super::utils::popup_area;
use crate::{
    emotes::{SharedEmotes, load_picker_emote},
    handlers::{
        config::SharedCoreConfig,
        user_input::events::{Event, Key},
    },
    terminal::TerminalAction,
    twitch::TwitchAction,
    ui::{
        components::{Component, utils::InputWidget},
        statics::TWITCH_MESSAGE_LIMIT,
    },
    utils::{
        colors::u32_to_color,
        emotes::UnicodePlaceholder,
        search::FUZZY_FINDER,
        styles::{NO_COLOR, SEARCH_STYLE, TITLE_STYLE},
        text::{TitleStyle, first_similarity_iter, title_line},
    },
};

pub struct EmotePickerWidget {
    config: SharedCoreConfig,
    emotes: SharedEmotes,
    input: InputWidget<SharedEmotes>,
    search_theme: Style,
    list_state: ListState,
    filtered_emotes: Vec<String>,
}

impl EmotePickerWidget {
    pub fn new(config: SharedCoreConfig, emotes: SharedEmotes) -> Self {
        let input_validator = Box::new(|emotes: SharedEmotes, s: String| -> bool {
            !s.is_empty()
                && s.len() < TWITCH_MESSAGE_LIMIT
                && (emotes.user_emotes.borrow().contains_key(&s)
                    || emotes.global_emotes.borrow().contains_key(&s))
        });

        let input_suggester = Box::new(|emotes: SharedEmotes, s: String| -> Option<String> {
            first_similarity_iter(
                emotes
                    .user_emotes
                    .borrow()
                    .keys()
                    .chain(emotes.global_emotes.borrow().keys()),
                &s,
            )
        });

        let input = InputWidget::new(
            config.clone(),
            "Emote",
            Some((emotes.clone(), input_validator)),
            None,
            Some((emotes.clone(), input_suggester)),
        );

        Self {
            config,
            emotes,
            input,
            search_theme: *SEARCH_STYLE,
            list_state: ListState::default(),
            filtered_emotes: vec![],
        }
    }
    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.filtered_emotes.len() - 1 {
                    self.filtered_emotes.len() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = self
            .list_state
            .selected()
            .map_or(0, |i| if i == 0 { 0 } else { i - 1 });

        self.list_state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.list_state.select(None);
    }
    pub const fn is_focused(&self) -> bool {
        self.input.is_focused()
    }

    pub const fn toggle_focus(&mut self) {
        self.input.toggle_focus();
    }
}

impl Component for EmotePickerWidget {
    fn draw(&mut self, f: &mut Frame, area: Option<Rect>) {
        let r = area.map_or_else(|| popup_area(f.area(), 60, 60), |a| a);

        // Only load the emotes that are actually being displayed, as loading every emote is not really possible.
        // Some channels can have multiple thousands emotes and decoding all of them takes a while.
        let max_len = max(
            self.list_state.selected().unwrap_or_default(),
            self.list_state.offset(),
        ) + r.height as usize;
        let mut items = Vec::with_capacity(max_len);
        let mut bad_emotes = vec![];

        let current_input = self.input.to_string();

        let cell_size = *self
            .emotes
            .cell_size
            .get()
            .expect("Terminal cell size should be set when emotes are enabled.");

        // Enter a new scope to drop the user/global emotes borrow when we don't need them anymore.
        {
            let user_emotes = self.emotes.user_emotes.borrow();
            let global_emotes = self.emotes.global_emotes.borrow();

            // First find all the emotes that match the input
            let mut matched_emotes = user_emotes
                .iter()
                .chain(global_emotes.iter())
                .filter_map(|(name, data)| {
                    Some((
                        name,
                        data,
                        FUZZY_FINDER.fuzzy_indices(&name.to_ascii_lowercase(), &current_input)?,
                    ))
                })
                .collect::<Vec<_>>();

            // Sort them by match score
            matched_emotes.sort_by(|a, b| b.2.0.cmp(&a.2.0));

            for (name, (filename, zero_width), (_, matched_indices)) in matched_emotes {
                if items.len() >= max_len {
                    break;
                }

                let Ok(loaded_emote) = load_picker_emote(
                    name,
                    filename,
                    *zero_width,
                    &mut self.emotes.info.borrow_mut(),
                    cell_size,
                )
                .map_err(|e| warn!("{e}")) else {
                    bad_emotes.push(name.clone());
                    continue;
                };

                let cols = (loaded_emote.width as f32 / cell_size.0).ceil() as u16;

                #[cfg(not(target_os = "windows"))]
                let underline_style = Style::default()
                    .fg(u32_to_color(loaded_emote.hash))
                    .underline_color(u32_to_color(1));

                #[cfg(target_os = "windows")]
                let underline_style = { Style::default().fg(u32_to_color(loaded_emote.hash)) };

                let mut row = name
                    .chars()
                    .enumerate()
                    .map(|(i, c)| {
                        if matched_indices.contains(&i) {
                            Span::styled(c.to_string(), self.search_theme)
                        } else {
                            Span::raw(c.to_string())
                        }
                    })
                    .collect::<Vec<Span>>();

                row.push(Span::raw(" - "));
                row.push(Span::styled(
                    UnicodePlaceholder::new(cols as usize).string(),
                    underline_style,
                ));

                items.push((name.clone(), ListItem::new(vec![Line::from(row)])));
            }
        }

        // Remove emotes that could not be loaded from list of emotes
        for emote in bad_emotes {
            self.emotes.info.borrow_mut().remove(&emote);
            self.emotes.user_emotes.borrow_mut().remove(&emote);
            self.emotes.global_emotes.borrow_mut().remove(&emote);
        }

        let (names, list_items) = items.into_iter().unzip();
        self.filtered_emotes = names;

        let title_binding = [TitleStyle::Single("Emotes")];

        let list = List::new::<Vec<ListItem>>(list_items)
            .block(
                Block::default()
                    .title(title_line(&title_binding, *TITLE_STYLE))
                    .borders(Borders::ALL)
                    .border_type(self.config.borrow().frontend.border_type.clone().into()),
            )
            .highlight_style(if *NO_COLOR {
                Style::default()
            } else {
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD)
            });

        f.render_widget(Clear, r);
        f.render_stateful_widget(list, r, &mut self.list_state);

        let bottom_block = Block::default()
            .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
            .border_type(self.config.borrow().frontend.border_type.clone().into());

        let rect = Rect::new(r.x, r.bottom() - 1, r.width, 1);

        f.render_widget(bottom_block, rect);

        let input_rect = Rect::new(r.x, r.bottom(), r.width, 3);

        self.input.draw(f, Some(input_rect));
    }

    async fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            match key {
                Key::Esc => self.toggle_focus(),
                Key::Ctrl('p') => panic!("Manual panic triggered by user."),
                Key::ScrollDown | Key::Down => self.next(),
                Key::ScrollUp | Key::Up => self.previous(),
                Key::Enter => {
                    if let Some(idx) = self.list_state.selected() {
                        let emote = self.filtered_emotes[idx].clone();

                        self.toggle_focus();
                        self.input.clear();
                        self.unselect();
                        self.filtered_emotes.clear();

                        return Some(TerminalAction::Enter(TwitchAction::Message(emote)));
                    }
                }
                _ => {
                    self.input.event(event).await;

                    // Assuming that the user inputted something that modified the input
                    match self.filtered_emotes.len() {
                        0 => self.unselect(),
                        _ => self.list_state.select(Some(0)),
                    }
                }
            }
        }

        None
    }
}
