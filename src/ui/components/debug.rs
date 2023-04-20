use std::slice::Iter;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::Frame,
    widgets::Paragraph,
};

use crate::handlers::{app::App, config::CompleteConfig};

fn render_config_values_widget<T: Backend>(frame: &mut Frame<T>, v_chunks: &mut Iter<Rect>) {
    let w = Paragraph::new("Base of debug menu");

    frame.render_widget(w, *v_chunks.next().unwrap());
}

pub fn render_debug_ui<T: Backend>(frame: &mut Frame<T>, _app: &mut App, _config: &CompleteConfig) {
    let v_chunk_binding = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            // Config values
            Constraint::Length(5),
        ])
        .margin(2)
        .split(frame.size());

    let mut v_chunks = v_chunk_binding.iter();

    render_config_values_widget(frame, &mut v_chunks);
}
