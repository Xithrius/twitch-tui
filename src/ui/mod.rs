pub mod chunks;
pub mod popups;
pub mod statics;

use std::collections::VecDeque;

use chrono::offset::Local;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::Spans,
    widgets::{Block, Borders, Cell, Row, Table},
};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerSmartWidget};

use crate::{
    handlers::{
        app::{App, BufferName, State},
        config::CompleteConfig,
        data::PayLoad,
    },
    utils::{styles, text::title_spans},
};

#[derive(Debug, Clone)]
pub struct Verticals {
    pub chunks: Vec<Rect>,
    pub constraints: VecDeque<Constraint>,
}

impl Verticals {
    pub fn new(chunks: Vec<Rect>, constraints: VecDeque<Constraint>) -> Self {
        Self {
            chunks,
            constraints,
        }
    }
}

pub fn draw_ui<T: Backend>(frame: &mut Frame<T>, app: &mut App, config: &CompleteConfig) {
    let table_widths = app.table_constraints.as_ref().unwrap();

    let mut vertical_chunk_constraints = VecDeque::from([Constraint::Min(1)]);

    // Allowing the input box to exist in different modes
    if let State::MessageInput | State::MessageSearch = app.state {
        vertical_chunk_constraints.push_back(Constraint::Length(3));
    }

    let mut logging_offset = 0;

    if config.frontend.logging {
        vertical_chunk_constraints.push_front(Constraint::Percentage(50));

        logging_offset = 1;
    }

    let margin = if config.frontend.padding { 1 } else { 0 };

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(margin)
        .constraints(vertical_chunk_constraints.clone())
        .split(frame.size());

    let verticals = Verticals::new(vertical_chunks, vertical_chunk_constraints);

    if config.frontend.logging {
        let logging_widget = TuiLoggerSmartWidget::default()
            .style_error(Style::default().fg(Color::Red))
            .style_debug(Style::default().fg(Color::Green))
            .style_warn(Style::default().fg(Color::Yellow))
            .style_trace(Style::default().fg(Color::Magenta))
            .style_info(Style::default().fg(Color::Cyan))
            .output_separator(':')
            .output_timestamp(Some("%H:%M:%S".to_string()))
            .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
            .output_target(true)
            .output_file(true)
            .output_line(true);

        frame.render_widget(logging_widget, verticals.chunks[0]);
    }

    let horizontal_table_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(margin)
        .constraints(table_widths.as_ref())
        .split(frame.size());

    // 0'th index because no matter what index is obtained, they're the same height.
    let general_chunk_height = verticals.chunks[logging_offset].height as usize - 3;

    // The chunk furthest to the right is the messages, that's the one we want.
    let message_chunk_width = horizontal_table_chunks[table_widths.len() - 1].width as usize - 4;

    // Making sure that messages do have a limit and don't eat up all the RAM.
    app.messages.truncate(config.terminal.maximum_messages);

    // Accounting for not all heights of rows to be the same due to text wrapping,
    // so extra space needs to be used in order to scroll correctly.
    let mut total_row_height: usize = 0;
    let mut display_rows = VecDeque::new();

    let mut scroll_offset = app.scroll_offset;

    'outer: for data in app.messages.iter() {
        if let PayLoad::Message(msg) = data.payload.clone() {
            if app.filters.contaminated(msg) {
                continue;
            }
        }

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
                    BufferName::MessageHighlighter => Some(buffer.to_string()),
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
            title_spans(
                vec![
                    vec![
                        "Time",
                        &Local::now()
                            .format(config.frontend.date_format.as_str())
                            .to_string(),
                    ],
                    vec!["Channel", config.twitch.channel.as_str()],
                    vec![
                        "Filters",
                        format!(
                            "{} / {}",
                            if app.filters.enabled() {
                                "enabled"
                            } else {
                                "disabled"
                            },
                            if app.filters.reversed() {
                                "reversed"
                            } else {
                                "static"
                            }
                        )
                        .as_str(),
                    ],
                ],
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )
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

    frame.render_widget(table, verticals.chunks[logging_offset]);

    match app.state {
        // States of the application that require a chunk of the main window
        State::MessageInput => {
            chunks::chatting::message_input(frame, app, verticals, config.storage.mentions)
        }
        State::MessageSearch => chunks::message_search::search_messages(frame, app, verticals),

        // States that require popups
        State::Help => popups::help::show_keybinds(frame),
        State::ChannelSwitch => {
            popups::channels::switch_channels(frame, app, config.storage.channels)
        }
        _ => {}
    }
}
