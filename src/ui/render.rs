use std::{collections::VecDeque, rc::Rc, vec};

use chrono::offset::Local;
use log::warn;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem},
};

use crate::{
    emotes::{
        emotes_enabled, hide_all_emotes, hide_message_emotes, is_in_rect, show_span_emotes, Emotes,
    },
    handlers::{
        app::App,
        config::{CompleteConfig, Theme},
        state::{NormalMode, State},
    },
    ui::components::utils::centered_rect,
    utils::{
        styles::{BORDER_NAME_DARK, BORDER_NAME_LIGHT},
        text::{title_spans, TitleStyle},
    },
};

// pub fn render_chat_ui<T: Backend>(
//     f: &mut Frame<T>,
//     app: &mut App,
//     config: &CompleteConfig,
//     emotes: &mut Emotes,
// ) {

// }
