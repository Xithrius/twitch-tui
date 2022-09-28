#![allow(clippy::too_many_lines)]

use std::{collections::VecDeque, vec};

use chrono::offset::Local;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::{
    handlers::{
        app::{App, State},
        config::CompleteConfig,
        data::PayLoad,
    },
    utils::{
        styles,
        text::{align_text, title_spans, TitleStyle},
    },
};

pub mod components;
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

pub fn draw_ui<T: Backend>(frame: &mut Frame<T>, app: &mut App, config: &CompleteConfig) {
    let username_column_title = align_text(
        "Username",
        &config.frontend.username_alignment,
        config.frontend.maximum_username_length,
    );

    let mut column_titles = vec![username_column_title, "Message content".to_string()];

    let mut table_constraints = vec![
        Constraint::Length(config.frontend.maximum_username_length),
        Constraint::Percentage(100),
    ];

    if config.frontend.date_shown {
        column_titles.insert(0, "Time".to_string());

        table_constraints.insert(
            0,
            Constraint::Length(
                Local::now()
                    .format(config.frontend.date_format.as_str())
                    .to_string()
                    .len() as u16,
            ),
        );
    }

    // Constraints for different states of the application.
    // Modify this in order to create new layouts.
    let mut v_constraints = match app.state {
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

    let layout = LayoutAttributes::new(v_constraints, v_chunks);

    // Horizontal chunks represents the table within the main chat window.
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(table_constraints.clone())
        .split(frame.size());

    // 0'th index because no matter what index is obtained, they're the same height.
    let general_chunk_height = layout.first_chunk().height as usize - 3;

    // The chunk furthest to the right is the messages, that's the one we want.
    let message_chunk_width = h_chunks[table_constraints.len() - 1].width as usize - 4;

    // Making sure that messages do have a limit and don't eat up all the RAM.
    app.messages.truncate(config.terminal.maximum_messages);

    // Accounting for not all heights of rows to be the same due to text wrapping,
    // so extra space needs to be used in order to scroll correctly.
    let mut total_row_height: usize = 0;
    let mut display_rows = VecDeque::new();

    let mut scroll_offset = app.scroll_offset;

    'outer: for data in &app.messages {
        if let PayLoad::Message(msg) = data.payload.clone() {
            if app.filters.contaminated(msg.as_str()) {
                continue;
            }
        }

        // Offsetting of messages for scrolling through said messages
        if scroll_offset > 0 {
            scroll_offset -= 1;

            continue;
        }

        let buffer = &mut app.input_buffer;

        let username_highlight = if config.frontend.username_highlight {
            Some(config.twitch.username.clone())
        } else {
            None
        };

        let rows = if buffer.is_empty() {
            data.to_row(
                &config.frontend,
                message_chunk_width,
                None,
                username_highlight,
                app.theme_style,
            )
        } else {
            data.to_row(
                &config.frontend,
                message_chunk_width,
                match app.state {
                    State::MessageSearch => Some(buffer.to_string()),
                    _ => None,
                },
                username_highlight,
                app.theme_style,
            )
        };

        for row in rows.iter().rev() {
            if total_row_height < general_chunk_height {
                display_rows.push_front(row.clone());

                total_row_height += 1;
            } else {
                break 'outer;
            }
        }
    }

    // Padding with empty rows so chat can go from bottom to top.
    if general_chunk_height > total_row_height {
        for _ in 0..(general_chunk_height - total_row_height) {
            display_rows.push_front(Row::new(vec![Cell::from("")]));
        }
    }

    let current_time = Local::now()
        .format(&config.frontend.date_format)
        .to_string();

    let chat_title = if config.frontend.title_shown {
        Spans::from(title_spans(
            vec![
                TitleStyle::Combined("Time", &current_time),
                TitleStyle::Combined("Channel", config.twitch.channel.as_str()),
                TitleStyle::Custom(Span::styled(
                    "Filter",
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(if app.filters.enabled() {
                            Color::Green
                        } else {
                            Color::Red
                        }),
                )),
            ],
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))
    } else {
        Spans::default()
    };

    let table = Table::new(display_rows)
        .header(Row::new(column_titles.clone()).style(styles::COLUMN_TITLE))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(chat_title)
                .style(app.theme_style),
        )
        .widths(&table_constraints)
        .column_spacing(1);

    frame.render_widget(table, layout.first_chunk());

    if config.frontend.state_tabs {
        components::render_state_tabs(frame, layout.clone(), app.state.clone());
    }

    let window = WindowAttributes::new(frame, app, layout, config.frontend.state_tabs);

    match window.app.state {
        // States of the application that require a chunk of the main window
        State::Insert => components::render_chat_box(window, config.storage.mentions),
        State::MessageSearch => {
            components::render_insert_box(window, "Message Search", None, None, None);
        }

        // States that require popups
        State::Help => components::render_help_window(window),
        State::ChannelSwitch => {
            components::render_channel_switcher(window, config.storage.channels);
        }
        State::Normal => {}
    }
}
