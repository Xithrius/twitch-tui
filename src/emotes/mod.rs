use log::{info, warn};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

use crate::{
    emotes::{
        downloader::get_emotes,
        graphics_protocol::{ApplyCommand, Size},
    },
    handlers::config::CompleteConfig,
    twitch::TwitchAction,
    utils::{emotes::get_emote_offset, pathing::cache_path},
};
use color_eyre::Result;
use tokio::sync::{broadcast::Receiver, mpsc::Sender};

mod downloader;
pub mod graphics_protocol;

// HashMap of emote name, emote filename, and if the emote is an overlay
pub type DownloadedEmotes = HashMap<String, (String, bool)>;

#[derive(Copy, Clone, Debug)]
pub struct EmoteData {
    pub width: u32,
    pub id: u32,
    pub pid: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct LoadedEmote {
    /// Hash of the emote filename, used as an ID for displaying the image
    pub hash: u32,
    /// Number of emotes that have been displayed
    pub n: u32,
    /// Width of the emote in pixels
    pub width: u32,
    /// If the emote should be displayed over the previous emote, if no text is between them.
    pub overlay: bool,
}

#[derive(Default, Debug, Clone)]
pub struct Emotes {
    /// Map of emote name, filename, and if the emote is an overlay
    pub emotes: DownloadedEmotes,
    /// Info about loaded emotes
    pub info: HashMap<String, LoadedEmote>,
    /// Terminal cell size in pixels: (width, height)
    pub cell_size: (f32, f32),
}

impl Emotes {
    pub fn unload(&mut self) {
        graphics_protocol::Clear.apply().unwrap_or_default();
        self.emotes.clear();
        self.info.clear();
    }
}

impl From<LoadedEmote> for EmoteData {
    fn from(LoadedEmote { hash, n, width, .. }: LoadedEmote) -> Self {
        Self {
            id: hash,
            pid: n,
            width,
        }
    }
}

pub async fn send_emotes(config: &CompleteConfig, tx: &Sender<DownloadedEmotes>, channel: &str) {
    info!("Starting emotes download.");
    match get_emotes(config, channel).await {
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
    tx: Sender<DownloadedEmotes>,
    mut rx: Receiver<TwitchAction>,
) {
    send_emotes(&config, &tx, &config.twitch.channel).await;

    loop {
        if let Ok(TwitchAction::Join(channel)) = rx.recv().await {
            send_emotes(&config, &tx, &channel).await;
        }
    }
}

pub fn load_emote(
    word: &str,
    filename: &str,
    overlay: bool,
    info: &mut HashMap<String, LoadedEmote>,
    cell_size: (f32, f32),
) -> Result<LoadedEmote> {
    if let Some(emote) = info.get_mut(word) {
        emote.n += 1;
        Ok(*emote)
    } else {
        let mut hasher = DefaultHasher::new();
        word.hash(&mut hasher);
        // ID is encoded on 3 bytes, discard the first one.
        let hash = hasher.finish() as u32 & 0x00FF_FFFF;

        // Tells the terminal to load the image for later use
        let loaded_image = graphics_protocol::Load::new(hash, &cache_path(filename), cell_size)?;
        let width = loaded_image.width();
        loaded_image.apply()?;

        let emote = LoadedEmote {
            hash,
            n: 1,
            width,
            overlay,
        };

        info.insert(word.to_string(), emote);
        Ok(emote)
    }
}

pub fn display_emote(id: u32, pid: u32, cols: u16) -> Result<()> {
    graphics_protocol::Display::new(id, pid, cols).apply()
}

pub fn overlay_emote(
    parent: (u32, u32),
    emote: EmoteData,
    layer: u32,
    cols: u16,
    root_col_offset: u16,
    cell_width: u16,
) -> Result<()> {
    // Center the overlay on top of the root emote.
    let (pixel_offset, col_offset) = get_emote_offset(emote.width as u16, cell_width, cols);

    let relative_col_offset = i32::from(root_col_offset) - i32::from(col_offset);

    graphics_protocol::Chain::new(
        emote.id,
        emote.pid,
        parent,
        layer,
        relative_col_offset,
        pixel_offset,
    )
    .apply()
}
