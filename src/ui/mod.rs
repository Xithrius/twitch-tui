#![allow(clippy::too_many_lines)]

use std::{collections::VecDeque, vec};

use chrono::offset::Local;
use log::warn;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem},
};
use unicode_width::UnicodeWidthStr;

use crate::emotes::{
    emotes_enabled, hide_all_emotes, hide_message_emotes, is_in_rect, show_span_emotes, Emotes,
};
use crate::ui::components::popups::centered_popup;
use crate::{
    handlers::{
        app::{App, State},
        config::{CompleteConfig, Theme},
    },
    utils::{
        styles::{BORDER_NAME_DARK, BORDER_NAME_LIGHT},
        text::{title_spans, TitleStyle},
    },
};

pub mod components;
pub mod error;
pub mod statics;

#[derive(Debug, Clone)]
pub struct LayoutAttributes {
    constraints: Vec<Constraint>,
    chunks: Vec<Rect>,
}

impl LayoutAttributes {
    pub fn new(constraints: Vec<Constraint>, chunks: Vec<Rect>) -> Self {
        Self {
            constraints,
            chunks,
        }
    }

    pub fn first_chunk(&self) -> Rect {
        self.chunks[0]
    }

    pub fn last_chunk(&self) -> Rect {
        self.chunks[self.chunks.len() - 1]
    }
}

pub struct WindowAttributes<'a, 'b, 'c, T: Backend> {
    frame: &'a mut Frame<'b, T>,
    app: &'c mut App,
    layout: LayoutAttributes,
    show_state_tabs: bool,
}

impl<'a, 'b, 'c, T: Backend> WindowAttributes<'a, 'b, 'c, T> {
    pub fn new(
        frame: &'a mut Frame<'b, T>,
        app: &'c mut App,
        layout: LayoutAttributes,
        show_state_tabs: bool,
    ) -> Self {
        Self {
            frame,
            app,
            layout,
            show_state_tabs,
        }
    }
}

pub fn draw_ui<T: Backend>(
    frame: &mut Frame<T>,
    app: &mut App,
    config: &CompleteConfig,
    emotes: &mut Emotes,
) {
    // Constraints for different states of the application.
    // Modify this in order to create new layouts.
    let mut v_constraints = match app.get_state() {
        State::Insert | State::MessageSearch => vec![Constraint::Min(1), Constraint::Length(3)],
        _ => vec![Constraint::Min(1)],
    };

    if config.frontend.state_tabs {
        v_constraints.push(Constraint::Length(1));
    }

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(config.frontend.margin)
        .constraints(v_constraints.as_ref())
        .split(frame.size());

    let layout = LayoutAttributes::new(v_constraints, v_chunks.to_vec());

    if app.messages.len() > config.terminal.maximum_messages {
        for data in app.messages.range(config.terminal.maximum_messages..) {
            hide_message_emotes(&data.emotes, &mut emotes.displayed, data.payload.width());
        }
        app.messages.truncate(config.terminal.maximum_messages);
    }

    // If we show the help screen, no need to get any messages
    let messages = if app.get_state() == State::Help {
        hide_all_emotes(emotes);
        VecDeque::new()
    } else {
        get_messages(frame, app, config, emotes, &layout)
    };

    let current_time = Local::now()
        .format(&config.frontend.date_format)
        .to_string();

    let spans = [
        TitleStyle::Combined("Time", &current_time),
        TitleStyle::Combined("Channel", config.twitch.channel.as_str()),
        TitleStyle::Custom(Span::styled(
            if app.filters.reversed() {
                "retliF"
            } else {
                "Filter"
            },
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(if app.filters.enabled() {
                    Color::Green
                } else {
                    Color::Red
                }),
        )),
    ];

    let chat_title = if config.frontend.title_shown {
        Spans::from(title_spans(
            &spans,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))
    } else {
        Spans::default()
    };

    let mut final_messages = vec![];

    for item in messages {
        final_messages.push(ListItem::new(Text::from(item)));
    }

    let list = List::new(final_messages)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(chat_title)
                .style(match app.theme {
                    Theme::Light => BORDER_NAME_LIGHT,
                    _ => BORDER_NAME_DARK,
                }),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(list, layout.first_chunk());

    if config.frontend.state_tabs {
        components::render_state_tabs(frame, &layout, &app.get_state());
    }

    let window = WindowAttributes::new(frame, app, layout, config.frontend.state_tabs);

    match window.app.get_state() {
        // States of the application that require a chunk of the main window
        State::Insert => components::render_chat_box(window, config.storage.mentions),
        State::MessageSearch => {
            let checking_func = |s: String| -> bool { !s.is_empty() };

            components::render_insert_box(
                window,
                "Message Search",
                None,
                None,
                Some(Box::new(checking_func)),
            );
        }

        // States that require popups
        State::Help => components::render_help_window(window),
        State::ChannelSwitch => {
            components::render_channel_switcher(window, config.storage.channels);
        }
        State::Normal => {}
    }
}

fn get_messages<'a, T: Backend>(
    frame: &mut Frame<T>,
    app: &'a App,
    config: &CompleteConfig,
    emotes: &mut Emotes,
    layout: &LayoutAttributes,
) -> VecDeque<Spans<'a>> {
    // Accounting for not all heights of rows to be the same due to text wrapping,
    // so extra space needs to be used in order to scroll correctly.
    let mut total_row_height: usize = 0;

    let mut messages = VecDeque::new();

    let mut scroll_offset = app.scrolling.get_offset();

    let general_chunk_height = layout.first_chunk().height as usize - 2;

    // Horizontal chunks represents the list within the main chat window.
    let h_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1)])
        .split(frame.size());

    let message_chunk_width = h_chunk[0].width as usize;

    let channel_switcher = if app.get_state() == State::ChannelSwitch {
        Some(centered_popup(frame.size(), frame.size().height))
    } else {
        None
    };

    let is_behind_channel_switcher = |a, b| channel_switcher.map_or(false, |r| is_in_rect(r, a, b));

    'outer: for data in &app.messages {
        if app.filters.contaminated(data.payload.clone().as_str()) {
            continue;
        }

        // Offsetting of messages for scrolling through said messages
        if scroll_offset > 0 {
            scroll_offset -= 1;
            hide_message_emotes(&data.emotes, &mut emotes.displayed, data.payload.width());

            continue;
        }

        let username_highlight: Option<&str> = if config.frontend.username_highlight {
            Some(&config.twitch.username)
        } else {
            None
        };

        let spans = data.to_spans(
            &config.frontend,
            message_chunk_width,
            if app.input_buffer.is_empty() {
                None
            } else {
                match app.get_state() {
                    State::MessageSearch => Some(app.input_buffer.as_str()),
                    _ => None,
                }
            },
            username_highlight,
        );

        let mut payload = " ".to_string();
        payload.push_str(&data.payload);

        for span in spans.iter().rev() {
            let mut span = span.clone();

            if total_row_height < general_chunk_height {
                if !data.emotes.is_empty() {
                    let current_row = general_chunk_height - total_row_height;
                    match show_span_emotes(
                        &data.emotes,
                        &mut span,
                        emotes,
                        &payload,
                        config.frontend.margin as usize,
                        current_row as u16,
                        is_behind_channel_switcher,
                    ) {
                        Ok(p) => payload = p,
                        Err(e) => warn!("Unable to display some emotes: {e}"),
                    }
                }

                messages.push_front(span);
                total_row_height += 1;
            } else {
                if !emotes_enabled(&config.frontend) || emotes.displayed.is_empty() {
                    break 'outer;
                }

                // If the current message already had all its emotes deleted, the following messages should
                // also have had their emotes deleted
                hide_message_emotes(&data.emotes, &mut emotes.displayed, payload.width());
                if !data.emotes.is_empty()
                    && !data
                        .emotes
                        .iter()
                        .all(|e| !emotes.displayed.contains_key(&(e.id, e.pid)))
                {
                    break 'outer;
                }
            }
        }
    }

    // Padding with empty rows so chat can go from bottom to top.
    if general_chunk_height > total_row_height {
        for _ in 0..(general_chunk_height - total_row_height) {
            messages.push_front(Spans::from(vec![Span::raw("")]));
        }
    }

    messages
}
