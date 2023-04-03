use std::string::ToString;

use regex::Regex;
use tui::backend::Backend;

use crate::{
    ui::{
        components::{popups::centered_popup, render_insert_box},
        statics::NAME_RESTRICTION_REGEX,
        WindowAttributes,
    },
    utils::text::first_similarity,
};

pub fn render_channel_switcher<T: Backend>(window: WindowAttributes<T>, channel_suggestions: bool) {
    let WindowAttributes {
        frame,
        app,
        layout: _,
        show_state_tabs: _,
    } = &window;

    let input_buffer = &app.input_buffer;

    let input_rect = centered_popup(frame.size(), frame.size().height);

    let suggestion = if channel_suggestions {
        first_similarity(
            &app.storage
                .get("channels")
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
            input_buffer,
        )
    } else {
        None
    };

    render_insert_box(
        window,
        "Channel",
        Some(input_rect),
        suggestion,
        Some(Box::new(|s: String| -> bool {
            Regex::new(&NAME_RESTRICTION_REGEX)
                .unwrap()
                .is_match(s.as_str())
        })),
    );
}
