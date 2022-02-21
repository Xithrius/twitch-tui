pub mod popups;
pub mod statics;

use chrono::offset::Local;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::{
    handlers::{
        app::{App, State},
        config::CompleteConfig,
    },
    ui::statics::COMMANDS,
    utils::{styles, text::get_cursor_position},
};

pub fn draw_ui<T: Backend>(frame: &mut Frame<T>, app: &mut App, config: &CompleteConfig) {
    let table_widths = app.table_constraints.as_ref().unwrap();

    let mut vertical_chunk_constraints = vec![Constraint::Min(1)];

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

        vertical_chunk_constraints.extend(vec![Constraint::Length(3)])
    }

    let margin = if config.frontend.padding { 1 } else { 0 };

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(margin)
        .constraints(vertical_chunk_constraints.as_ref())
        .split(frame.size());

    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(margin)
        .constraints(table_widths.as_ref())
        .split(frame.size());

    // 0'th index because no matter what index is obtained, they're the same height.
    let general_chunk_height = vertical_chunks[0].height as usize - 3;

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

        let rows = data.to_row(&config.frontend, &message_chunk_width);

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

    frame.render_widget(table, vertical_chunks[0]);

    match app.state {
        State::MessageInput => {
            let input_buffer = app.current_buffer();

            if input_buffer.starts_with('/') {
                let suggested_commands = COMMANDS
                    .iter()
                    .map(|f| format!("/{}", f))
                    .filter(|f| f.starts_with(input_buffer.as_str()))
                    .collect::<Vec<String>>()
                    .join("\n");

                let suggestions_paragraph = Paragraph::new(suggested_commands)
                    .style(Style::default().fg(Color::Blue))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("[ Command suggestions ]"),
                    );

                frame.render_widget(suggestions_paragraph, vertical_chunks[1]);
            }

            let cursor_pos = get_cursor_position(input_buffer);
            let input_rect = vertical_chunks[vertical_chunk_constraints.len() - 1];

            frame.set_cursor(
                (input_rect.x + cursor_pos as u16 + 1)
                    .min(input_rect.x + input_rect.width.saturating_sub(2)),
                input_rect.y + 1,
            );

            let paragraph = Paragraph::new(input_buffer.as_str())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("[ Input ]"))
                .scroll((
                    0,
                    ((cursor_pos + 3) as u16).saturating_sub(input_rect.width),
                ));

            frame.render_widget(
                paragraph,
                vertical_chunks[vertical_chunk_constraints.len() - 1],
            );
        }
        State::Help => popups::help::show_keybinds(frame),
        State::ChannelSwitch => popups::channels::switch_channels(frame, app),
        _ => {}
    }
}
