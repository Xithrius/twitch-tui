use color_eyre::{eyre::anyhow, Result};
use log::{error, info, warn};
use std::{
    cell::{OnceCell, RefCell},
    collections::{hash_map::DefaultHasher, BTreeMap, HashMap},
    hash::{Hash, Hasher},
    rc::Rc,
    sync::OnceLock,
};
use tokio::sync::{
    mpsc::{Receiver, Sender},
    oneshot::{Receiver as OSReceiver, Sender as OSSender},
};

use crate::{
    emotes::{downloader::get_emotes, graphics_protocol::Image},
    handlers::config::CompleteConfig,
    utils::{
        emotes::{emotes_enabled, get_emote_offset},
        pathing::cache_path,
    },
};

mod downloader;
mod graphics_protocol;

pub use graphics_protocol::{support_graphics_protocol, ApplyCommand, DecodedEmote};

// HashMap of emote name, emote filename, and if the emote is an overlay
pub type DownloadedEmotes = BTreeMap<String, (String, bool)>;

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
    /// Width of the emote in pixels (resized so that it's height is equal to cell height)
    pub width: u32,
    /// If the emote should be displayed over the previous emote, if no text is between them.
    pub overlay: bool,
}

#[derive(Default, Debug)]
pub struct Emotes {
    /// Map of emote name, filename, and if the emote is an overlay
    pub emotes: RefCell<DownloadedEmotes>,
    /// Info about loaded emotes
    pub info: RefCell<HashMap<String, LoadedEmote>>,
    /// Terminal cell size in pixels: (width, height)
    pub cell_size: OnceCell<(f32, f32)>,
}

pub type SharedEmotes = Rc<Emotes>;

// This Drop impl is only here to cleanup in case of panics.
// The unload method should be called before exiting the alternate screen instead of relying on the drop impl.
impl Drop for Emotes {
    fn drop(&mut self) {
        self.unload();
    }
}

impl Emotes {
    pub fn unload(&self) {
        self.info
            .borrow()
            .iter()
            .for_each(|(_, LoadedEmote { hash, .. })| {
                graphics_protocol::Clear(*hash).apply().unwrap_or_default();
            });
        self.emotes.borrow_mut().clear();
        self.info.borrow_mut().clear();
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

pub fn query_emotes(config: &CompleteConfig, channel: String) -> OSReceiver<DownloadedEmotes> {
    let (tx, mut rx) = tokio::sync::oneshot::channel();

    if emotes_enabled(&config.frontend) {
        let config = config.clone();
        tokio::spawn(async move { send_emotes(&config, tx, channel).await });
    } else {
        rx.close();
    }

    rx
}

pub async fn send_emotes(config: &CompleteConfig, tx: OSSender<DownloadedEmotes>, channel: String) {
    info!("Starting emotes download.");
    match get_emotes(config, &channel).await {
        Ok(emotes) => {
            info!("Emotes downloaded.");
            if tx.send(emotes).is_err() {
                warn!("Unable to send emotes to main thread.");
            }
        }
        Err(e) => {
            warn!("Unable to download emotes: {e}");
        }
    }
}

pub static DECODE_EMOTE_SENDER: OnceLock<Sender<Image>> = OnceLock::new();

pub fn decoder(mut rx: Receiver<Image>, tx: &Sender<Result<DecodedEmote, String>>) {
    while let Some(emote) = rx.blocking_recv() {
        let name = emote.name.clone();

        let decoded = emote.decode().map_err(|_| name);

        if tx.blocking_send(decoded).is_err() {
            error!("Unable to send decoded emote to main thread.");
            return;
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
        let image = Image::new(
            hash,
            word.to_string(),
            &cache_path(filename),
            overlay,
            cell_size,
        )?;

        let width = image.width;
        let cols = image.cols;

        let decoded = image.decode()?;

        decoded.apply()?;

        // Emote with placement id 1 is reserved for emote picker
        // We tell kitty to display it now, but as it is a unicode placeholder,
        // it will only be displayed once we print the unicode placeholder.
        display_emote(hash, 1, cols)?;

        let emote = LoadedEmote {
            hash,
            n: 2,
            width,
            overlay,
        };

        info.insert(word.to_string(), emote);
        Ok(emote)
    }
}

pub fn load_picker_emote(
    word: &str,
    filename: &str,
    overlay: bool,
    info: &mut HashMap<String, LoadedEmote>,
    cell_size: (f32, f32),
) -> Result<LoadedEmote> {
    if let Some(emote) = info.get(word) {
        Ok(*emote)
    } else {
        let mut hasher = DefaultHasher::new();
        word.hash(&mut hasher);
        // ID is encoded on 3 bytes, discard the first one.
        let hash = hasher.finish() as u32 & 0x00FF_FFFF;

        // Tells the terminal to load the image for later use
        let image = Image::new(
            hash,
            word.to_string(),
            &cache_path(filename),
            overlay,
            cell_size,
        )?;

        let width = image.width;

        // Decode emote in another thread, to avoid blocking main thread as decoding images is slow.
        DECODE_EMOTE_SENDER
            .get()
            .ok_or(anyhow!("Decoding channel has not been initialized."))?
            .try_send(image)
            .map_err(|e| anyhow!("Unable to send emote to decoder thread. {e}"))?;

        // Emote with placement id 1 is reserved for emote picker
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
