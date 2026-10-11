use std::time::Duration;

use color_eyre::{Result, eyre::Context};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};
use tui::crossterm::event::{
    self, Event as CEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent,
    MouseEventKind,
};

use crate::events::{Event, key::Key};

pub struct Events {
    rx: Receiver<Event>,
}

impl Events {
    pub fn new(delay: u64, tx: Sender<Event>, rx: Receiver<Event>) -> Self {
        let tick_rate = Duration::from_millis(delay);
        let actor = EventsThread::new(tx, tick_rate);
        tokio::task::spawn(async move { actor.run().await });

        Self { rx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}

pub struct EventsThread {
    tx: Sender<Event>,
    tick_rate: Duration,
}

impl EventsThread {
    pub const fn new(tx: Sender<Event>, tick_rate: Duration) -> Self {
        Self { tx, tick_rate }
    }

    async fn run(&self) -> Result<()> {
        let mut last_tick = Instant::now();

        loop {
            let timeout = self
                .tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout)? {
                match event::read() {
                    Ok(CEvent::Key(KeyEvent {
                        code,
                        kind: KeyEventKind::Press,
                        modifiers,
                        state: _,
                    })) => self.translate_key_event(code, modifiers).await?,
                    Ok(CEvent::Mouse(key)) => self.translate_mouse_event(key).await?,
                    _ => (),
                }
            }

            if last_tick.elapsed() >= self.tick_rate {
                self.tx.send(Event::Tick).await?;
                last_tick = Instant::now();
            }
        }
    }

    async fn translate_key_event(&self, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        let key = match code {
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Esc => Key::Esc,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::Delete => Key::Delete,
            KeyCode::Insert => Key::Insert,
            KeyCode::Tab => Key::Tab,
            KeyCode::Enter => Key::Enter,
            KeyCode::Char(c) => match modifiers {
                KeyModifiers::NONE | KeyModifiers::SHIFT => Key::Char(c),
                KeyModifiers::CONTROL => Key::Ctrl(c),
                KeyModifiers::ALT => Key::Alt(c),
                _ => return Ok(()),
            },
            _ => return Ok(()),
        };

        self.send(Event::Input(key)).await?;

        Ok(())
    }

    async fn translate_mouse_event(&self, key: MouseEvent) -> Result<()> {
        let key = match key.kind {
            MouseEventKind::ScrollDown => Key::ScrollDown,
            MouseEventKind::ScrollUp => Key::ScrollUp,
            _ => return Ok(()),
        };

        self.send(Event::Input(key)).await?;

        Ok(())
    }

    async fn send(&self, event: Event) -> Result<()> {
        self.tx.send(event).await.context("Failed to send event")
    }
}
