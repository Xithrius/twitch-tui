use std::vec;

use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    handlers::app::State,
    ui::WindowAttributes,
    utils::text::{get_cursor_position, title_spans, TitleStyle},
};

pub use chunks::{chatting::render_chat_box, help::render_help_window, states::render_state_tabs};
pub use popups::channels::render_channel_switcher;

use self::popups::centered_popup;
