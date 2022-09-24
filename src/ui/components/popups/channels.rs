use std::string::ToString;

use regex::Regex;
use tui::backend::Backend;

use crate::{
    ui::{
        components::popups::centered_popup, insert_box_chunk, statics::CHANNEL_NAME_REGEX,
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

    let input_rect = centered_popup(frame.size(), frame.size().height);

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
            Regex::new(&CHANNEL_NAME_REGEX)
                .unwrap()
                .is_match(s.as_str())
        })),
    );
}
