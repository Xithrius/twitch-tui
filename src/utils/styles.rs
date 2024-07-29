use once_cell::sync::Lazy;
use std::env;
use tui::style::{Color, Modifier, Style};

pub static NO_COLOR: Lazy<bool> = Lazy::new(|| env::var("NO_COLOR").is_ok());
pub static BOLD: Lazy<Modifier> = Lazy::new(|| {
    if *NO_COLOR {
        Modifier::empty()
    } else {
        Modifier::BOLD
    }
});

macro_rules! color {
    ($color:expr) => {
        if *NO_COLOR {
            None
        } else {
            Some($color)
        }
    };
}

pub static BOLD_STYLE: Lazy<Style> = Lazy::new(|| Style {
    fg: None,
    bg: None,
    underline_color: None,
    add_modifier: *BOLD,
    sub_modifier: Modifier::empty(),
});

pub static STATE_TABS_STYLE: Lazy<Style> = Lazy::new(|| Style {
    fg: color!(Color::Gray),
    bg: None,
    underline_color: None,
    add_modifier: if *NO_COLOR {
        Modifier::empty()
    } else {
        Modifier::DIM
    },
    sub_modifier: Modifier::empty(),
});

pub static TEXT_DARK_STYLE: Lazy<Style> = Lazy::new(|| Style {
    fg: color!(Color::White),
    bg: None,
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
});

pub static DASHBOARD_SECTION_STYLE: Lazy<Style> = Lazy::new(|| Style {
    fg: color!(Color::LightRed),
    bg: None,
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
});

#[allow(dead_code)]
pub static BORDER_NAME_DARK_STYLE: Lazy<Style> = Lazy::new(|| Style {
    fg: color!(Color::White),
    bg: None,
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
});

#[allow(dead_code)]
pub static BORDER_NAME_LIGHT_STYLE: Lazy<Style> = Lazy::new(|| Style {
    fg: color!(Color::Black),
    bg: None,
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
});

pub static DATETIME_DARK_STYLE: Lazy<Style> = Lazy::new(|| Style {
    fg: color!(Color::Rgb(173, 173, 184)),
    bg: None,
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
});

pub static DATETIME_LIGHT_STYLE: Lazy<Style> = Lazy::new(|| Style {
    fg: color!(Color::Rgb(83, 83, 95)),
    bg: None,
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
});

pub static HIGHLIGHT_NAME_DARK_STYLE: Lazy<Style> = Lazy::new(|| Style {
    fg: color!(Color::Black),
    bg: color!(Color::White),
    underline_color: None,
    add_modifier: *BOLD,
    sub_modifier: Modifier::empty(),
});

pub static HIGHLIGHT_NAME_LIGHT_STYLE: Lazy<Style> = Lazy::new(|| Style {
    fg: color!(Color::White),
    bg: color!(Color::Black),
    underline_color: None,
    add_modifier: *BOLD,
    sub_modifier: Modifier::empty(),
});

pub static COLUMN_TITLE_STYLE: Lazy<Style> = Lazy::new(|| Style {
    fg: color!(Color::LightCyan),
    bg: None,
    underline_color: None,
    add_modifier: *BOLD,
    sub_modifier: Modifier::empty(),
});

pub static SYSTEM_CHAT_STYLE: Lazy<Style> = Lazy::new(|| Style {
    fg: color!(Color::Red),
    bg: None,
    underline_color: None,
    add_modifier: *BOLD,
    sub_modifier: Modifier::empty(),
});

pub static DASHBOARD_TITLE_COLOR_STYLE: Lazy<Style> = Lazy::new(|| Style {
    fg: color!(Color::Rgb(135, 120, 165)),
    bg: None,
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
});

pub static SEARCH_STYLE: Lazy<Style> = Lazy::new(|| Style {
    fg: color!(Color::Red),
    bg: None,
    underline_color: None,
    add_modifier: *BOLD,
    sub_modifier: Modifier::empty(),
});

pub static TITLE_STYLE: Lazy<Style> = Lazy::new(|| Style {
    fg: color!(Color::Red),
    bg: None,
    underline_color: None,
    add_modifier: *BOLD,
    sub_modifier: Modifier::empty(),
});
