use anyhow::Result;
use tokio::sync::mpsc;

use handlers::config::CompleteConfig;

use crate::handlers::app::App;

mod handlers;
mod terminal;
mod twitch;
mod ui;
mod utils;

const HELP: &str = "\
twitch-tui

USAGE:
  app [OPTIONS]
FLAGS:
  -h, --help            Prints help information
OPTIONS:
  --channel CHANNEL     Sets a channel(the streamer's name)
";

#[derive(Debug)]
struct AppArgs {
    channel: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };

    match CompleteConfig::new() {
        Ok(mut config) => {
            if let Some(c) = args.channel {
                config.twitch.channel = c;
            }

            let app = App::new(config.clone());

            let (twitch_tx, terminal_rx) = mpsc::channel(100);
            let (terminal_tx, twitch_rx) = mpsc::channel(100);

            let cloned_config = config.clone();

            tokio::task::spawn(async move {
                twitch::twitch_irc(config, twitch_tx, twitch_rx).await;
            });

            terminal::ui_driver(cloned_config, app, terminal_tx, terminal_rx)
                .await
                .unwrap();

            std::process::exit(0);
        }
        Err(err) => {
            println!("{}", err);
        }
    }

    Ok(())
}

fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let args = AppArgs {
        channel: pargs.opt_value_from_str("--channel")?,
    };

    pargs.finish();

    Ok(args)
}
