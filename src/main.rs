use anyhow::Result;

use crate::utils::app::App;

mod tui;
mod twitch;
mod utils;

fn main() -> Result<()> {
    let app = App::default();

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        twitch::twitch_irc(&tx);
    });

    tui::tui(app, rx).unwrap();

    Ok(())
}
