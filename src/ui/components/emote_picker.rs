use log::warn;
use memchr::memmem;
use std::cmp::max;
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

use crate::{
    emotes::{load_picker_emote, SharedEmotes},
    handlers::{
        config::SharedCompleteConfig,
        user_input::events::{Event, Key},
    },
    terminal::TerminalAction,
    twitch::TwitchAction,
    ui::{
        components::{
            utils::{centered_rect, InputWidget},
            Component,
        },
        statics::TWITCH_MESSAGE_LIMIT,
    },
    utils::{
        colors::u32_to_color,
        emotes::UnicodePlaceholder,
        text::{first_similarity_iter, title_line, TitleStyle},
    },
};

pub struct EmotePickerWidget {
    config: SharedCompleteConfig,
    emotes: SharedEmotes,
    input: InputWidget<SharedEmotes>,
    search_theme: Style,
    list_state: ListState,
    filtered_emotes: Vec<String>,
}

impl EmotePickerWidget {
    pub fn new(config: SharedCompleteConfig, emotes: SharedEmotes) -> Self {
        let input_validator = Box::new(|emotes: SharedEmotes, s: String| -> bool {
            !s.is_empty()
                && s.len() < TWITCH_MESSAGE_LIMIT
                && emotes.emotes.borrow().contains_key(&s)
        });

        let input_suggester = Box::new(|emotes: SharedEmotes, s: String| -> Option<String> {
            first_similarity_iter(emotes.emotes.borrow().keys(), &s)
        });

        let input = InputWidget::new(
            config.clone(),
            "Emote",
            Some((emotes.clone(), input_validator)),
            None,
            Some((emotes.clone(), input_suggester)),
        );

        let search_theme = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);

        Self {
            config,
            emotes,
            input,
            search_theme,
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

    pub fn toggle_focus(&mut self) {
        self.input.toggle_focus();
    }
}

impl Component for EmotePickerWidget {
    fn draw(&mut self, f: &mut Frame, area: Option<Rect>) {
        let mut r = area.map_or_else(|| centered_rect(60, 60, 23, f.size()), |a| a);
        // Make sure we have space for the input widget, which has a height of 3.
        r.height -= 3;

        // Only load the emotes that are actually being displayed, as loading every emote is not really possible.
        // Some channels can have multiple thousands emotes and decoding all of them takes a while.
        let max_len = max(
            self.list_state.selected().unwrap_or_default(),
            self.list_state.offset(),
        ) + r.height as usize;
        let mut items = Vec::with_capacity(max_len);
        let mut bad_emotes = vec![];

        let mut current_input = self.input.to_string();

        let cell_size = *self
            .emotes
            .cell_size
            .get()
            .expect("Terminal cell size should be set when emotes are enabled.");

        let finder = if current_input.is_empty() {
            None
        } else {
            current_input.make_ascii_lowercase();
            Some(memmem::Finder::new(&current_input))
        };

        for (name, (filename, zero_width)) in self.emotes.emotes.borrow().iter() {
            if items.len() >= max_len {
                break;
            }

            // Skip emotes that do not contain the current input, if it is not empty.
            let Some(pos) = finder
                .as_ref()
                .map_or_else(|| Some(0), |f| f.find(name.to_ascii_lowercase().as_bytes()))
            else {
                continue;
            };

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

            let row = vec![
                Span::raw(name[0..pos].to_owned()),
                Span::styled(
                    name[pos..(pos + current_input.len())].to_owned(),
                    self.search_theme,
                ),
                Span::raw(name[(pos + current_input.len())..].to_owned()),
                Span::raw(" - "),
                Span::styled(
                    UnicodePlaceholder::new(cols as usize).string(),
                    Style::default()
                        .fg(u32_to_color(loaded_emote.hash))
                        .underline_color(u32_to_color(1)),
                ),
            ];

            items.push((name.clone(), ListItem::new(vec![Line::from(row)])));
        }

        // Remove emotes that could not be loaded from list of emotes
        for emote in bad_emotes {
            self.emotes.info.borrow_mut().remove(&emote);
            self.emotes.emotes.borrow_mut().remove(&emote);
        }

        let (names, list_items) = items.into_iter().unzip();
        self.filtered_emotes = names;

        let title_binding = [TitleStyle::Single("Emotes")];

        let list = List::new::<Vec<ListItem>>(list_items)
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
                        self.input.update("");
                        self.unselect();
                        self.filtered_emotes.clear();

                        return Some(TerminalAction::Enter(TwitchAction::Privmsg(emote)));
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
