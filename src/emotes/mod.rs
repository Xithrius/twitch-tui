use crate::emotes::downloader::get_emotes;
use crate::handlers::config::{CompleteConfig, FrontendConfig};
use crate::handlers::data::EmoteData;
use crate::utils::pathing::cache_path;
use log::{info, warn};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use tokio::sync::mpsc::Sender;
use tui::text::{Span, Spans};
use unicode_width::UnicodeWidthStr;

mod downloader;
pub mod kitty;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Json(json::JsonError),
    Io(std::io::Error),
    Str(String),
}

impl From<reqwest::Error> for Error {
    fn from(src: reqwest::Error) -> Self {
        Self::Reqwest(src)
    }
}

impl From<json::JsonError> for Error {
    fn from(src: json::JsonError) -> Self {
        Self::Json(src)
    }
}

impl From<std::io::Error> for Error {
    fn from(src: std::io::Error) -> Self {
        Self::Io(src)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LoadedEmote {
    /// Hash of the emote filename, used as an ID for kitty
    pub hash: u32,
    /// Number of emotes loaded
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
    /// Emotes loaded by kitty
    pub loaded: HashMap<String, LoadedEmote>,
    /// Emotes currently displayed: (id, placement id), (col, row)
    pub displayed: HashMap<(u32, u32), (u32, u32)>,
    /// Terminal cell size in pixels:  (width, height)
    pub cell_size: (u32, u32),
}

impl Emotes {
    pub async fn new(config: &CompleteConfig, cell_size: (u32, u32)) -> Result<Self> {
        let emotes = get_emotes(config).await?;

        Ok(Self {
            emotes,
            loaded: HashMap::new(),
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
    if !emotes_enabled(&config.frontend) {
        drop(tx);
        return;
    }
    info!("Starting emotes download.");
    if let Ok(emotes) = Emotes::new(&config, terminal_cell_size).await {
        info!("Emotes downloaded.");
        tx.send(emotes).await.unwrap();
    }

    drop(tx);
}

pub fn show_emotes(
    emotes: &Vec<EmoteData>,
    prefix_width: usize,
    previous_span_width: usize,
    span_end_position: usize,
    row: usize,
    span: &mut Span,
    displayed: &mut HashMap<(u32, u32), (u32, u32)>,
) {
    if emotes.is_empty() {
        return;
    }

    let mut words: Vec<String> = span.content.split(' ').map(ToString::to_string).collect();
    let mut position = 0;
    let mut word_idx = 0;

    for emote in emotes {
        if emote.string_position > span_end_position {
            break;
        }
        if emote.string_position < previous_span_width {
            continue;
        }

        let kitty_position = (
            (emote.string_position + prefix_width - previous_span_width) as u32,
            row as u32,
        );

        if displayed.get(&emote.kitty_id) != Some(&kitty_position) {
            if let Err(err) = kitty::display(&mut std::io::stdout(), emote, kitty_position) {
                warn!("Unable to display emote {}: {err:?}", emote.name);
            }
        }

        // Replace placeholder string in span by spaces.
        let p = emote.string_position - previous_span_width;
        while word_idx < words.len() {
            let word = &mut words[word_idx];
            word_idx += 1;

            if position == p {
                *word = " ".repeat(emote.width as usize);
                position += word.width() + 1;
                break;
            }
            position += word.width() + 1;
        }
        displayed.insert(emote.kitty_id, kitty_position);
    }
    *span.content.to_mut() = words.join(" ");
}

pub fn delete_emotes(
    emotes: &Vec<EmoteData>,
    displayed: &mut HashMap<(u32, u32), (u32, u32)>,
    pos: usize,
) {
    for emote in emotes {
        if displayed.contains_key(&emote.kitty_id) && emote.string_position < pos {
            let (emote_id, placement_id) = emote.kitty_id;
            if let Err(err) = kitty::clear(&mut std::io::stdout(), emote_id, placement_id) {
                warn!("Unable to delete emote {}: {err}", emote.name);
            } else {
                displayed.remove(&emote.kitty_id);
            }
        }
    }
}

pub fn load_emote(
    word: &str,
    filename: &str,
    loaded_emotes: &mut HashMap<String, LoadedEmote>,
    cell_size: (u32, u32),
) -> Result<LoadedEmote, Box<dyn std::error::Error>> {
    if let Some(emote) = loaded_emotes.get_mut(word) {
        emote.n += 1;
        Ok(*emote)
    } else {
        let mut hasher = DefaultHasher::new();
        word.hash(&mut hasher);
        let hash = hasher.finish() as u32;

        // Tells kitty to load the image for later use
        let (mut width_px, height_px) =
            kitty::load(&mut std::io::stdout(), hash, &cache_path(filename))?;
        // Resize width to fit image in 1 cell of height
        width_px = (width_px as f32 * cell_size.1 as f32 / height_px as f32).ceil() as u32;
        // Offset the image on the left side, otherwise kitty will stretch it
        let offset = (width_px % cell_size.0).abs_diff(cell_size.0) % cell_size.0;
        let width_cell = (width_px as f32 / cell_size.0 as f32).ceil() as u32;
        let emote = LoadedEmote {
            hash,
            n: 1,
            width: width_cell,
            offset,
        };

        loaded_emotes.insert(word.to_string(), emote);
        Ok(emote)
    }
}

pub fn reload_emotes(emotes: &Emotes) {
    for (word, LoadedEmote { hash, .. }) in &emotes.loaded {
        if let Some(filename) = emotes.emotes.get(word) {
            if let Err(err) = kitty::load(&mut std::io::stdout(), *hash, &cache_path(filename)) {
                warn!("Unable to reload emote {word}: {err}");
            }
        }
    }
}

pub fn calculate_emote_position(
    span: &Spans,
    payload: &mut String,
) -> Result<(usize, usize), Error> {
    let span_width = span.0.iter().map(|s| s.content.width()).sum::<usize>();

    if let Some(last_span) = span.0.last() {
        let span_end = payload.width() - 1;

        if let Some(str) = payload
            .trim_end()
            .strip_suffix(last_span.content.trim_end())
        {
            *payload = str.to_string();
            return Ok((span_width, span_end));
        }
    }
    Err(Error::Str("Error calculating emote position.".to_string()))
}
