use std::slice::Iter;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{List, ListItem, Paragraph},
};

use crate::{
    handlers::{
        app::{App, State},
        config::CompleteConfig,
    },
    ui::{
        components::{dashboard::DASHBOARD_TITLE, render_channel_switcher},
        WindowAttributes,
    },
    utils::styles::DASHBOARD_TITLE_COLOR,
};

fn create_interactive_list_widget(items: &[String], index_offset: usize) -> List<'_> {
    List::new(
        items
            .iter()
            .enumerate()
            .map(|(i, s)| {
                ListItem::new(Spans::from(vec![
                    Span::raw("["),
                    Span::styled(
                        (i + index_offset).to_string(),
                        Style::default().fg(Color::LightMagenta),
                    ),
                    Span::raw("] "),
                    Span::raw(s),
                ]))
            })
            .collect::<Vec<ListItem>>(),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
}

fn render_dashboard_title_widget<T: Backend>(frame: &mut Frame<T>, v_chunks: &mut Iter<Rect>) {
    let w = Paragraph::new(
        DASHBOARD_TITLE
            .iter()
            .map(|&s| Spans::from(vec![Span::raw(s)]))
            .collect::<Vec<Spans>>(),
    )
    .style(DASHBOARD_TITLE_COLOR);

    frame.render_widget(w, *v_chunks.next().unwrap());
}

fn render_channel_selection_widget<T: Backend>(
    frame: &mut Frame<T>,
    v_chunks: &mut Iter<Rect>,
    app: &App,
    current_channel: String,
    default_channels: &[String],
) {
    frame.render_widget(
        Paragraph::new("Currently selected channel").style(Style::default().fg(Color::LightRed)),
        *v_chunks.next().unwrap(),
    );

    let current_channel_selection = Paragraph::new(Spans::from(vec![
        Span::raw("["),
        Span::styled(
            "ENTER".to_string(),
            Style::default().fg(Color::LightMagenta),
        ),
        Span::raw("] "),
        Span::raw(current_channel),
    ]));

    frame.render_widget(current_channel_selection, *v_chunks.next().unwrap());

    frame.render_widget(
        Paragraph::new("Configured default channels").style(Style::default().fg(Color::LightRed)),
        *v_chunks.next().unwrap(),
    );

    if default_channels.is_empty() {
        frame.render_widget(Paragraph::new("None"), *v_chunks.next().unwrap());
    } else {
        let default_channels_widget = create_interactive_list_widget(default_channels, 0);

        frame.render_widget(default_channels_widget, *v_chunks.next().unwrap());
    }

    frame.render_widget(
        Paragraph::new("Most recent channels").style(Style::default().fg(Color::LightRed)),
        *v_chunks.next().unwrap(),
    );

    let recent_channels = app.storage.get_last_n("channels", 5, true);

    if recent_channels.is_empty() {
        frame.render_widget(Paragraph::new("None"), *v_chunks.next().unwrap());
    } else {
        let recent_channels_widget =
            create_interactive_list_widget(&recent_channels, default_channels.len());

        frame.render_widget(recent_channels_widget, *v_chunks.next().unwrap());
    }
}

fn render_quit_selection_widget<T: Backend>(frame: &mut Frame<T>, v_chunks: &mut Iter<Rect>) {
    let quit_option = Paragraph::new(Spans::from(vec![
        Span::raw("["),
        Span::styled("q", Style::default().fg(Color::LightMagenta)),
        Span::raw("] "),
        Span::raw("Quit"),
    ]));

    frame.render_widget(quit_option, *v_chunks.next().unwrap());
}

pub fn render_dashboard_ui<T: Backend>(
    frame: &mut Frame<T>,
    app: &mut App,
    config: &CompleteConfig,
) {
    let start_screen_channels_len = config.frontend.start_screen_channels.len() as u16;

    let recent_channels_len = app.storage.get("channels").len() as u16;

    let v_chunk_binding = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            // Twitch-tui ASCII logo
            Constraint::Min(DASHBOARD_TITLE.len() as u16 + 2),
            // Currently selected channel title, content
            Constraint::Length(2),
            Constraint::Min(2),
            // Configured default channels title, content
            Constraint::Length(2),
            Constraint::Min(if start_screen_channels_len == 0 {
                2
            } else {
                start_screen_channels_len + 1
            }),
            // Recent channel title, content
            Constraint::Length(2),
            Constraint::Min(if recent_channels_len == 0 {
                2
            } else {
                recent_channels_len + 1
            }),
            // Quit
            Constraint::Min(1),
        ])
        .margin(2)
        .split(frame.size());

    let mut v_chunks = v_chunk_binding.iter();

    render_dashboard_title_widget(frame, &mut v_chunks);

    render_channel_selection_widget(
        frame,
        &mut v_chunks,
        app,
        config.twitch.channel.clone(),
        &config.frontend.start_screen_channels.clone(),
    );

    render_quit_selection_widget(frame, &mut v_chunks);

    if Some(State::Dashboard) == app.get_previous_state() {
        render_channel_switcher(
            WindowAttributes::new(frame, app, None, config.frontend.clone()),
            config.storage.channels,
        );
    }
}
