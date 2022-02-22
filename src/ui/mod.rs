pub mod chunks;
pub mod popups;
pub mod statics;

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
        app::{App, BufferName::MessageHighlighter, State},
        config::CompleteConfig,
    },
    utils::styles,
};

#[derive(Debug, Clone)]
pub struct Verticals {
    pub chunks: Vec<Rect>,
    pub constraints: Vec<Constraint>,
}

impl Verticals {
    pub fn new(chunks: Vec<Rect>, constraints: Vec<Constraint>) -> Self {
        Self {
            chunks,
            constraints,
        }
    }
}

pub fn draw_ui<T: Backend>(frame: &mut Frame<T>, app: &mut App, config: &CompleteConfig) {
    let table_widths = app.table_constraints.as_ref().unwrap();

    let mut vertical_chunk_constraints = vec![Constraint::Min(1)];

    // A little chunk to show you different commands when in insert mode
    if let State::MessageInput = app.state {
        if app
            .input_buffers
            .get(&app.selected_buffer)
            .unwrap()
            .as_str()
            .starts_with('/')
        {
            vertical_chunk_constraints.push(Constraint::Length(9));
        }
    }

    // Allowing the input box to exist in different modes
    if let State::MessageInput | State::MessageSearch = app.state {
        vertical_chunk_constraints.extend(vec![Constraint::Length(3)]);
    }

    let margin = if config.frontend.padding { 1 } else { 0 };

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(margin)
        .constraints(vertical_chunk_constraints.as_ref())
        .split(frame.size());

    let verticals = Verticals::new(vertical_chunks, vertical_chunk_constraints);

    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(margin)
        .constraints(table_widths.as_ref())
        .split(frame.size());

    // 0'th index because no matter what index is obtained, they're the same height.
    let general_chunk_height = verticals.chunks[0].height as usize - 3;

    // The chunk furthest to the right is the messages, that's the one we want.
    let message_chunk_width = horizontal_chunks[table_widths.len() - 1].width as usize - 4;

    // Making sure that messages do have a limit and don't eat up all the RAM.
    app.messages.truncate(config.terminal.maximum_messages);

    // Accounting for not all heights of rows to be the same due to text wrapping,
    // so extra space needs to be used in order to scroll correctly.
    let mut total_row_height: usize = 0;
    let mut display_rows = std::collections::VecDeque::new();

    let mut scroll_offset = app.scroll_offset;

    'outer: for data in app.messages.iter() {
        if scroll_offset > 0 {
            scroll_offset -= 1;

            continue;
        }

        let buffer = app.current_buffer();

        let rows = if !buffer.is_empty() {
            data.to_row(
                &config.frontend,
                &message_chunk_width,
                match app.selected_buffer {
                    MessageHighlighter => Some(buffer.to_string()),
                    _ => None,
                },
            )
        } else {
            data.to_row(&config.frontend, &message_chunk_width, None)
        };

        for row in rows.iter().rev() {
            if total_row_height < general_chunk_height {
                display_rows.push_front(row.to_owned());

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

    let chat_title_format = || -> Spans {
        if config.frontend.title_shown {
            Spans::from(vec![
                Span::raw("[ "),
                Span::styled(
                    "Time",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(
                    ": {} ] [ ",
                    Local::now().format(config.frontend.date_format.as_str())
                )),
                Span::styled(
                    "Channel",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(": {} ]", config.twitch.channel)),
            ])
        } else {
            Spans::default()
        }
    };

    let table = Table::new(display_rows)
        .header(
            Row::new(app.column_titles.as_ref().unwrap().to_owned()).style(styles::COLUMN_TITLE),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(chat_title_format())
                .style(styles::BORDER_NAME),
        )
        .widths(table_widths.as_ref())
        .column_spacing(1);

    frame.render_widget(table, verticals.chunks[0]);

    match app.state {
        // States of the application that require a chunk of the main window
        State::MessageInput => chunks::chatting::message_input(frame, app, verticals),
        State::MessageSearch => chunks::message_search::search_messages(frame, app, verticals),

        // States that require popups
        State::Help => popups::help::show_keybinds(frame),
        State::ChannelSwitch => popups::channels::switch_channels(frame, app),
        _ => {}
    }
}
