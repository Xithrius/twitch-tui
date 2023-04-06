use crate::emotes::downloader::get_emotes;
use crate::emotes::graphics_protocol::Size;
use crate::handlers::config::{CompleteConfig, FrontendConfig};
use crate::handlers::data::EmoteData;
use crate::utils::pathing::cache_path;
use anyhow::{Context, Result};
use log::{info, warn};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use tokio::sync::mpsc::Sender;
use tui::text::Span;
use unicode_width::UnicodeWidthStr;

mod downloader;
pub mod graphics_protocol;

#[derive(Debug, Copy, Clone)]
pub struct LoadedEmote {
    /// Hash of the emote filename, used as an ID for displaying the image
    pub hash: u32,
    /// Number of emotes that have been displayed
    pub n: u32,
    /// Width in cells of the emote
    pub width: u32,
    /// Offset is the difference between the emote width in pixels, and the edge of the next cell
    pub offset: u32,
}

#[derive(Default, Debug)]
pub struct Emotes {
    /// Map of emote name, filename
    pub emotes: HashMap<String, String>,
    /// Emotes currently loaded
    pub loaded: HashSet<u32>,
    /// Info about loaded emotes
    pub info: HashMap<String, LoadedEmote>,
    /// Emotes currently displayed: (id, placement id), (col, row)
    pub displayed: HashMap<(u32, u32), (u16, u16)>,
    /// Terminal cell size in pixels:  (width, height)
    pub cell_size: (u32, u32),
}

impl Emotes {
    pub async fn new(config: &CompleteConfig, cell_size: (u32, u32)) -> Result<Self> {
        let emotes = get_emotes(config).await?;

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

pub async fn emotes_setup(
    config: CompleteConfig,
    tx: Sender<Emotes>,
    terminal_cell_size: (u32, u32),
) {
    info!("Starting emotes download.");
    match Emotes::new(&config, terminal_cell_size).await {
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

pub fn show_emotes(
    message_emotes: &Vec<EmoteData>,
    prefix_width: usize,
    previous_span_width: usize,
    span_end_position: usize,
    row: usize,
    span: &mut Span,
    emotes: &mut Emotes,
) -> Result<()> {
    let mut words: Vec<String> = span.content.split(' ').map(ToString::to_string).collect();
    let mut string_position = 0;
    let mut word_idx = 0;

    for emote in message_emotes {
        // Emote is on a further row, we can exit now
        if emote.string_position > span_end_position {
            break;
        }
        // Emote is on a previous row
        if emote.string_position < previous_span_width {
            continue;
        }

        let terminal_position = (
            (emote.string_position + prefix_width - previous_span_width) as u16,
            row as u16,
        );

        if !emotes.loaded.contains(&emote.id) {
            reload_emote(emotes, &emote.name, emote.id)?;
        }

        let info = emotes.info.get(&emote.name).context("Emote not loaded")?;

        if emotes.displayed.get(&(emote.id, emote.pid)) != Some(&terminal_position) {
            if let Err(err) = graphics_protocol::command(graphics_protocol::Display::new(
                terminal_position,
                emote.id,
                emote.pid,
                info.width,
                info.offset,
            )) {
                warn!("Unable to display emote {}: {err:?}", emote.name);
            }
        }

        // Replace placeholder string in span by spaces.
        let p = emote.string_position - previous_span_width;
        while word_idx < words.len() {
            let word = &mut words[word_idx];
            word_idx += 1;

            if string_position == p {
                *word = " ".repeat(info.width as usize);
                string_position += word.width() + 1;
                break;
            }
            string_position += word.width() + 1;
        }

        emotes
            .displayed
            .insert((emote.id, emote.pid), terminal_position);
    }
    *span.content.to_mut() = words.join(" ");

    Ok(())
}

pub fn delete_emotes(
    emotes: &Vec<EmoteData>,
    displayed: &mut HashMap<(u32, u32), (u16, u16)>,
    pos: usize,
) {
    for emote in emotes {
        if displayed.contains_key(&(emote.id, emote.pid)) && emote.string_position < pos {
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
    info: &mut HashMap<String, LoadedEmote>,
    loaded: &mut HashSet<u32>,
    (cell_w, cell_h): (u32, u32),
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
        let (mut width_px, height_px) = loaded_image.size();
        graphics_protocol::command(loaded_image)?;

        // Resize width to fit image in 1 cell of height
        width_px = (width_px as f32 * cell_h as f32 / height_px as f32).ceil() as u32;
        // Offset the image on the left side, otherwise the terminal will stretch it
        let offset = (width_px % cell_w).abs_diff(cell_w) % cell_w;
        let width_cell = (width_px as f32 / cell_w as f32).ceil() as u32;
        let emote = LoadedEmote {
            hash,
            n: 1,
            width: width_cell,
            offset,
        };

        info.insert(word.to_string(), emote);
        loaded.insert(hash);
        Ok(emote)
    }
}

pub fn reload_emote(emotes: &mut Emotes, name: &str, hash: u32) -> Result<()> {
    let filename = emotes.emotes.get(name).context("Emote not found")?;
    graphics_protocol::command(graphics_protocol::Load::new(hash, &cache_path(filename))?)?;
    emotes.loaded.insert(hash);
    Ok(())
}
