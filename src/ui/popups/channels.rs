use std::string::ToString;

use regex::Regex;
use tui::backend::Backend;

use crate::{
    ui::{
        insert_box_chunk,
        popups::{centered_popup, WindowType},
        statics::CHANNEL_NAME_REGEX,
        WindowAttributes,
    },
    utils::text::suggestion_query,
};

pub fn ui_switch_channels<T: Backend>(window: WindowAttributes<T>, channel_suggestions: bool) {
    let WindowAttributes {
        frame,
        app,
        layout: _,
    } = &window;

    let input_buffer = &app.input_buffer;

    let input_rect = centered_popup(WindowType::Input(frame.size().height), frame.size());

    let suggestion = if channel_suggestions {
        suggestion_query(
            input_buffer,
            app.storage
                .get("channels")
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        )
    } else {
        None
    };

    insert_box_chunk(
        window,
        "Channel",
        Some(input_rect),
        suggestion,
        Some(Box::new(|s: String| -> bool {
            Regex::new(*CHANNEL_NAME_REGEX)
                .unwrap()
                .is_match(s.as_str())
        })),
    );
}
