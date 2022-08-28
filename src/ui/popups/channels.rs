use tui::backend::Backend;

use crate::{
    ui::{
        insert_box_chunk,
        popups::{centered_popup, WindowType},
        WindowAttributes,
    },
    utils::text::suggestion_query,
};

pub fn ui_switch_channels<'a: 'b, 'b, 'c, T: Backend>(
    window: WindowAttributes<'a, 'b, 'c, T>,
    channel_suggestions: bool,
) {
    let WindowAttributes { frame, app, layout } = window;

    let input_buffer = app.current_buffer();

    let input_rect = centered_popup(WindowType::Input(frame.size().height), frame.size());

    let suggestion = if channel_suggestions {
        suggestion_query(
            input_buffer.to_string(),
            app.storage
                .get("channels".to_string())
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
        )
    } else {
        None
    };

    insert_box_chunk(
        frame,
        app,
        layout,
        Some(input_rect),
        suggestion.clone(),
        None,
    );
}
