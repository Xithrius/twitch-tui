use anyhow::{Context, Result};
use log::{info, warn};
use std::{
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::{Hash, Hasher},
    rc::Rc,
};
use tokio::sync::{broadcast::Receiver, mpsc::Sender};
use tui::{
    layout::Rect,
    text::{Span, Spans},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    emotes::{downloader::get_emotes, graphics_protocol::Size},
    handlers::{
        config::{CompleteConfig, FrontendConfig},
        data::EmoteData,
    },
    twitch::TwitchAction,
    utils::pathing::cache_path,
};

mod downloader;
pub mod graphics_protocol;

#[derive(Debug, Copy, Clone)]
pub struct LoadedEmote {
    /// Hash of the emote filename, used as an ID for displaying the image
    pub hash: u32,
    /// Number of emotes that have been displayed
    pub n: u32,
    /// Width in cells of the emote
    pub width: u16,
    /// Offset is the difference between the emote width in pixels, and the edge of the next cell
    pub offset: u16,
    /// If the emote should be displayed over the previous emote, if no text is between them.
    pub overlay: bool,
}

pub type SharedEmotes = Rc<RefCell<Emotes>>;

#[derive(Default, Debug, Clone)]
pub struct Emotes {
    /// Map of emote name, filename, and if the emote is an overlay
    pub emotes: HashMap<String, (String, bool)>,
    /// Emotes currently loaded
    pub loaded: HashSet<u32>,
    /// Info about loaded emotes
    pub info: HashMap<String, LoadedEmote>,
    /// Emotes currently displayed: (id, placement id), (col, row)
    pub displayed: HashMap<(u32, u32), (u16, u16)>,
    /// Terminal cell size in pixels: (width, height)
    pub cell_size: (u16, u16),
}

impl Emotes {
    pub async fn new(
        config: &CompleteConfig,
        channel: &str,
        cell_size: (u16, u16),
    ) -> Result<Self> {
        let emotes = get_emotes(config, channel).await?;

        Ok(Self {
            emotes,
            loaded: HashSet::new(),
            info: HashMap::new(),
            displayed: HashMap::new(),
            cell_size,
        })
    }
}

#[inline]
pub const fn emotes_enabled(frontend: &FrontendConfig) -> bool {
    frontend.twitch_emotes || frontend.betterttv_emotes || frontend.seventv_emotes
}

#[inline]
pub fn is_in_rect(rect: Rect, (x, y): (u16, u16), width: u16) -> bool {
    y < rect.bottom() && y > rect.top() - 1 && x < rect.right() && x + width > rect.left()
}

pub async fn send_emotes(
    config: &CompleteConfig,
    tx: &Sender<Emotes>,
    channel: &str,
    terminal_cell_size: (u16, u16),
) {
    info!("Starting emotes download.");
    match Emotes::new(config, channel, terminal_cell_size).await {
        Ok(emotes) => {
            info!("Emotes downloaded.");
            if let Err(e) = tx.send(emotes).await {
                warn!("Unable to send emotes to main thread: {e}");
            }
        }
        Err(e) => {
            warn!("Unable to download emotes: {e}");
        }
    }
}

pub async fn emotes(
    config: CompleteConfig,
    tx: Sender<Emotes>,
    mut rx: Receiver<TwitchAction>,
    terminal_cell_size: (u16, u16),
) {
    send_emotes(&config, &tx, &config.twitch.channel, terminal_cell_size).await;

    loop {
        match rx.recv().await {
            Ok(TwitchAction::Join(channel)) => {
                send_emotes(&config, &tx, &channel, terminal_cell_size).await;
            }
            Ok(_) | Err(_) => {}
        }
    }
}

pub fn show_emotes<F>(
    message_emotes: &Vec<EmoteData>,
    span: &mut Span,
    emotes: &mut Emotes,
    prefix_width: usize,
    // Payload start/end index in span content
    (start, end): (usize, usize),
    row: u16,
    hide: F,
) where
    F: Fn((u16, u16), u16) -> bool,
{
    let mut emotes_pos_in_span = HashSet::new();

    for emote in message_emotes {
        // Emote is on a further row, we can exit now
        if emote.index_in_message > end {
            break;
        }
        // Emote is on a previous row
        if emote.index_in_message < start {
            continue;
        }

        let col = (emote.index_in_message + prefix_width - start) as u16;

        emotes_pos_in_span.insert(emote.index_in_message - start);

        let info = if let Some(info) = emotes.info.get(&emote.name) {
            info
        } else {
            warn!("Unable to get info of emote {}", emote.name);
            continue;
        };

        // Delete emote if we don't need to display it
        if hide((col, row), info.width) {
            if let Err(err) =
                graphics_protocol::command(graphics_protocol::Clear(emote.id, emote.pid))
            {
                warn!("Unable to delete emote {}: {err}", emote.name);
            } else {
                emotes.displayed.remove(&(emote.id, emote.pid));
            }
            continue;
        }

        if !emotes.loaded.contains(&emote.id)
            && reload_emote(&emotes.emotes, &mut emotes.loaded, &emote.name, emote.id).is_err()
        {
            continue;
        }

        // Only send a command if the emote changed position
        if emotes.displayed.get(&(emote.id, emote.pid)) != Some(&(col, row)) {
            if let Err(err) = graphics_protocol::command(graphics_protocol::Display::new(
                (col, row),
                emote,
                info.width,
                info.offset,
            )) {
                warn!("Unable to display emote {}: {err:?}", emote.name);
            }
        }

        emotes.displayed.insert((emote.id, emote.pid), (col, row));
    }

    update_span_content(span, &emotes_pos_in_span);
}

/// Replace span content at emote position with spaces
pub fn update_span_content(span: &mut Span, positions: &HashSet<usize>) {
    if positions.is_empty() {
        return;
    }

    let mut words: Vec<String> = span.content.split(' ').map(ToString::to_string).collect();
    let mut string_position = 0;
    let mut word_idx = 0;

    while word_idx < words.len() {
        let word = &mut words[word_idx];
        word_idx += 1;

        if positions.contains(&string_position) {
            *word = " ".repeat(word.width());
        }

        string_position += word.width() + 1;
    }

    *span.content.to_mut() = words.join(" ");
}

pub fn hide_message_emotes(
    emotes: &Vec<EmoteData>,
    displayed: &mut HashMap<(u32, u32), (u16, u16)>,
    pos: usize,
) {
    for emote in emotes {
        if displayed.contains_key(&(emote.id, emote.pid)) && emote.index_in_message < pos {
            if let Err(err) =
                graphics_protocol::command(graphics_protocol::Clear(emote.id, emote.pid))
            {
                warn!("Unable to delete emote {}: {err}", emote.name);
            } else {
                displayed.remove(&(emote.id, emote.pid));
            }
        }
    }
}

pub fn load_emote(
    word: &str,
    filename: &str,
    overlay: bool,
    info: &mut HashMap<String, LoadedEmote>,
    loaded: &mut HashSet<u32>,
    (cell_w, cell_h): (u16, u16),
) -> Result<LoadedEmote> {
    if let Some(emote) = info.get_mut(word) {
        emote.n += 1;
        Ok(*emote)
    } else {
        let mut hasher = DefaultHasher::new();
        word.hash(&mut hasher);
        let hash = hasher.finish() as u32;

        // Tells the terminal to load the image for later use
        let loaded_image = graphics_protocol::Load::new(hash, &cache_path(filename))?;
        let (width_px, height_px) = loaded_image.size();
        graphics_protocol::command(loaded_image)?;

        let mut width_px = width_px as f32;
        let cell_w = u32::from(cell_w);
        // Resize width to fit image in 1 cell of height
        width_px = (width_px * f32::from(cell_h) / height_px as f32).ceil();
        // Offset the image on the left side, otherwise the terminal will stretch it
        let offset = ((width_px as u32 % cell_w).abs_diff(cell_w) % cell_w) as u16;
        let width_cell = (width_px / cell_w as f32).ceil() as u16;
        let emote = LoadedEmote {
            hash,
            n: 1,
            width: width_cell,
            offset,
            overlay,
        };

        info.insert(word.to_string(), emote);
        loaded.insert(hash);
        Ok(emote)
    }
}

pub fn reload_emote(
    emote_list: &HashMap<String, (String, bool)>,
    loaded_emotes: &mut HashSet<u32>,
    name: &str,
    hash: u32,
) -> Result<()> {
    let (filename, _) = emote_list.get(name).context("Emote not found")?;
    graphics_protocol::command(graphics_protocol::Load::new(hash, &cache_path(filename))?)?;
    loaded_emotes.insert(hash);
    Ok(())
}

pub fn unload_all_emotes(emotes: &mut Emotes) {
    graphics_protocol::command(graphics_protocol::Clear(0, 0)).unwrap_or_default();
    emotes.emotes.clear();
    emotes.loaded.clear();
    emotes.info.clear();
    emotes.displayed.clear();
}

pub fn hide_all_emotes(emotes: &mut Emotes) {
    graphics_protocol::command(graphics_protocol::Clear(0, 1)).unwrap_or_default();
    emotes.displayed.clear();
}

pub fn show_span_emotes<F>(
    message_emotes: &Vec<EmoteData>,
    span: &mut Spans,
    emotes: &mut Emotes,
    payload: &str,
    margin: usize,
    current_row: u16,
    hide: F,
) -> Result<String>
where
    F: Fn((u16, u16), u16) -> bool,
{
    let span_width: usize = span.0.iter().map(|s| s.content.width()).sum();
    let last_span = span.0.last_mut().context("Span is empty")?;

    let p = payload
        .trim_end()
        .strip_suffix(last_span.content.trim_end())
        .context("Unable to find span content in payload")?;

    show_emotes(
        message_emotes,
        last_span,
        emotes,
        span_width + margin + 1 - last_span.content.width(),
        (p.width(), payload.width() - 1),
        current_row,
        hide,
    );

    Ok(p.to_string())
}
