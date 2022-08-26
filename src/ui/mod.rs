use std::collections::{HashMap, VecDeque};
use std::vec;

use chrono::offset::Local;
use color_eyre::eyre::ContextCompat;
use lazy_static::lazy_static;
use maplit::hashmap;
use tui::layout::Rect;
use tui::text::Span;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::Spans,
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::utils::text::TitleStyle;
use crate::{
    handlers::{
        app::{App, BufferName, State},
        config::CompleteConfig,
        data::PayLoad,
    },
    utils::{styles, text::title_spans},
};

pub mod chunks;
pub mod popups;
pub mod statics;

lazy_static! {
    pub static ref LAYOUTS: HashMap<State, Vec<Constraint>> = hashmap! {
        State::Normal => vec![Constraint::Min(1)],
        State::Insert => vec![Constraint::Min(1), Constraint::Length(3)],
        State::Help => vec![Constraint::Min(1)],
        State::ChannelSwitch => vec![Constraint::Min(1)]
    };
}

pub struct LayoutAttributes<'a> {
    constraints: &'a Vec<Constraint>,
    chunks: Vec<Rect>,
}

impl<'a> LayoutAttributes<'a> {
    pub fn new(constraints: &'a Vec<Constraint>, chunks: Vec<Rect>) -> Self {
        Self {
            constraints,
            chunks,
        }
    }
}

pub fn draw_ui<T: Backend>(frame: &mut Frame<T>, app: &mut App, config: &CompleteConfig) {
    let v_constraints = LAYOUTS
        .get(&app.state)
        .wrap_err(format!("Could not find layout {:?}.", &app.state))
        .unwrap();

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(config.frontend.margin)
        .constraints(v_constraints.as_ref())
        .split(frame.size());

    let layout = LayoutAttributes::new(v_constraints, v_chunks);

    let table_widths = app.table_constraints.as_ref().unwrap();

    // Horizontal chunks represents the table within the main chat window.
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(table_widths.as_ref())
        .split(frame.size());

    // 0'th index because no matter what index is obtained, they're the same height.
    let general_chunk_height = layout.chunks[0].height as usize - 3;

    // The chunk furthest to the right is the messages, that's the one we want.
    let message_chunk_width = h_chunks[table_widths.len() - 1].width as usize - 4;

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

        // Offsetting of messages for scrolling through said messages
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
                app.theme_style,
            )
        } else {
            data.to_row(
                &config.frontend,
                &message_chunk_width,
                None,
                app.theme_style,
            )
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
        .header(
            Row::new(app.column_titles.as_ref().unwrap().to_owned()).style(styles::COLUMN_TITLE),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(chat_title)
                .style(app.theme_style),
        )
        .widths(table_widths.as_ref())
        .column_spacing(1);

    frame.render_widget(table, layout.chunks[0]);

    match app.state {
        // States of the application that require a chunk of the main window
        State::Insert => {
            chunks::chatting::message_input(frame, app, layout, config.storage.mentions)
        }
        State::MessageSearch => chunks::message_search::search_messages(frame, app, layout),

        // States that require popups
        State::Help => popups::help::show_keybinds(frame, app.theme_style),
        State::ChannelSwitch => {
            popups::channels::switch_channels(frame, app, config.storage.channels)
        }
        _ => {}
    }
}
