use lazy_static::lazy_static;

lazy_static! {
    pub static ref COLUMN_TITLES: Vec<&'static str> = vec!["Keybind", "Description"];
    pub static ref NORMAL_MODE: Vec<Vec<&'static str>> = vec![
        vec!["c", "Chat window"],
        vec!["?", "Bring up this window"],
        vec!["q", "Quit this application"],
        vec!["Esc", "Drop back to previous window layer"],
    ];
    pub static ref INSERT_MODE: Vec<Vec<&'static str>> = vec![
        vec!["Ctrl + f", "Move cursor to the right"],
        vec!["Ctrl + b", "Move cursor to the left"],
        vec!["Ctrl + a", "Move cursor to the start"],
        vec!["Ctrl + e", "Move cursor to the end"],
        vec!["Alt + f", "Move to the end of the next word"],
        vec!["Alt + b", "Move to the start of the previous word"],
        vec!["Ctrl + t", "Swap previous item with current item"],
        vec!["Alt + t", "Swap previous word with current word"],
        vec!["Ctrl + u", "Remove everything before the cursor"],
        vec!["Ctrl + k", "Remove everything after the cursor"],
        vec!["Ctrl + w", "Remove the previous word"],
        vec!["Ctrl + d", "Remove item to the right"],
        vec!["Esc", "Drop back to previous window layer"],
    ];
}
