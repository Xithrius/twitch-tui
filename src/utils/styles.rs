use tui::style::{Color, Modifier, Style};

#[allow(dead_code)]
pub const BORDER_NAME_DARK: Style = Style {
    fg: Some(Color::White),
    bg: None,
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

#[allow(dead_code)]
pub const BORDER_NAME_LIGHT: Style = Style {
    fg: Some(Color::Black),
    bg: None,
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

pub const DATETIME_DARK: Style = Style {
    fg: Some(Color::Rgb(173, 173, 184)),
    bg: None,
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

pub const DATETIME_LIGHT: Style = Style {
    fg: Some(Color::Rgb(83, 83, 95)),
    bg: None,
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

pub const HIGHLIGHT_NAME_DARK: Style = Style {
    fg: Some(Color::Black),
    bg: Some(Color::White),
    underline_color: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const HIGHLIGHT_NAME_LIGHT: Style = Style {
    fg: Some(Color::White),
    bg: Some(Color::Black),
    underline_color: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};

pub const COLUMN_TITLE: Style = Style {
    fg: Some(Color::LightCyan),
    bg: None,
    underline_color: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};

pub const SYSTEM_CHAT: Style = Style {
    fg: Some(Color::Red),
    bg: None,
    underline_color: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};

pub const DASHBOARD_TITLE_COLOR: Style = Style {
    fg: Some(Color::Rgb(135, 120, 165)),
    bg: None,
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};
