use std::{env, sync::LazyLock};

use tui::style::{Color, Modifier, Style};

pub static NO_COLOR: LazyLock<bool> = LazyLock::new(|| env::var("NO_COLOR").is_ok());
pub static BOLD: LazyLock<Modifier> = LazyLock::new(|| {
    if *NO_COLOR {
        Modifier::empty()
    } else {
        Modifier::BOLD
    }
});

macro_rules! color {
    ($color:expr) => {
        if *NO_COLOR { None } else { Some($color) }
    };
}

macro_rules! define_style {
    ($name:ident, $($key:ident: $value:expr),*) => {
        pub static $name: std::sync::LazyLock<Style> = std::sync::LazyLock::new(|| Style {
            $(
                $key: $value,
            )*
            ..Style::default()
        });
    };
}

define_style!(
    BOLD_STYLE,
    add_modifier: *BOLD
);

define_style!(STATE_TABS_STYLE,
    fg: color!(Color::Gray),
    add_modifier: if *NO_COLOR {
        Modifier::empty()
    } else {
        Modifier::DIM
    }
);

define_style!(TEXT_DARK_STYLE,
    fg: color!(Color::White)
);

define_style!(DASHBOARD_SECTION_STYLE,
    fg: color!(Color::LightRed)
);

define_style!(DATETIME_DARK_STYLE,
    fg: color!(Color::Rgb(173, 173, 184))
);

define_style!(DATETIME_LIGHT_STYLE,
    fg: color!(Color::Rgb(83, 83, 95))
);

define_style!(HIGHLIGHT_NAME_DARK_STYLE,
    fg: color!(Color::Black),
    bg: color!(Color::White),
    add_modifier: *BOLD
);

define_style!(HIGHLIGHT_NAME_LIGHT_STYLE,
    fg: color!(Color::White),
    bg: color!(Color::Black),
    add_modifier: *BOLD
);

define_style!(COLUMN_TITLE_STYLE,
    fg: color!(Color::LightCyan),
    add_modifier: *BOLD
);

define_style!(SYSTEM_CHAT_STYLE,
    fg: color!(Color::Red),
    add_modifier: *BOLD
);

define_style!(DASHBOARD_TITLE_COLOR_STYLE,
    fg: color!(Color::Rgb(135, 120, 165))
);

define_style!(SEARCH_STYLE,
    fg: color!(Color::Red),
    add_modifier: *BOLD
);

define_style!(TITLE_STYLE,
    fg: color!(Color::Red),
    add_modifier: *BOLD
);
