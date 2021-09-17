use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use termion::{event::Key, input::TermRead};
use tokio::{sync::mpsc, task::unconstrained, task::JoinHandle};

use futures::FutureExt;

pub enum Event<I> {
    Input(I),
    Tick,
}

#[allow(dead_code)]
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    input_handle: JoinHandle<()>,
    ignore_exit_key: Arc<AtomicBool>,
    tick_handle: JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Events {
    pub async fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel(1);
        let ignore_exit_key = Arc::new(AtomicBool::new(false));
        let input_handle = {
            let tx = tx.clone();
            let ignore_exit_key = ignore_exit_key.clone();
            tokio::spawn(async move {
                let stdin = io::stdin();
                for key in stdin.keys().flatten() {
                    if let Err(err) = tx.send(Event::Input(key)).await {
                        eprintln!("{}", err);
                        return;
                    }
                    if !ignore_exit_key.load(Ordering::Relaxed) && key == config.exit_key {
                        return;
                    }
                }
            })
        };
        let tick_handle = {
            tokio::spawn(async move {
                loop {
                    if tx.send(Event::Tick).await.is_err() {
                        break;
                    }
                    tokio::time::sleep(config.tick_rate).await;
                }
            })
        };
        Events {
            rx,
            ignore_exit_key,
            input_handle,
            tick_handle,
        }
    }

    pub async fn next(&mut self) -> Option<Event<Key>> {
        unconstrained(self.rx.recv()).now_or_never().and_then(|f| f)
    }
}
