use tui::style::{Color, Modifier, Style};

pub const BORDER_NAME_DARK: Style = Style {
    fg: Some(Color::White),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};
pub const BORDER_NAME_LIGHT: Style = Style {
    fg: Some(Color::Black),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};
pub const COLUMN_TITLE: Style = Style {
    fg: Some(Color::LightCyan),
    bg: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const SYSTEM_CHAT: Style = Style {
    fg: Some(Color::Red),
    bg: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
