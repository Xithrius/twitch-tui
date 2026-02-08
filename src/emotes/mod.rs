use std::{
    cell::{OnceCell, RefCell},
    collections::{BTreeMap, HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    rc::Rc,
    sync::OnceLock,
    thread,
};

use color_eyre::{Result, eyre::anyhow};
use tokio::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
    oneshot::{Receiver as OSReceiver, Sender as OSSender},
};
use tracing::{error, info, warn};

use crate::{
    config::{CoreConfig, persistence::get_cache_dir},
    emotes::{downloader::get_emotes, graphics_protocol::Image},
    utils::emotes::get_emote_offset,
};

mod downloader;
mod graphics_protocol;

pub use downloader::get_twitch_emote;
pub use graphics_protocol::{ApplyCommand, DecodedEmote, support_graphics_protocol};

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
    /// Map of emote name, filename, and if the emote is an overlay.
    /// We keep track of both emotes that can be used by the current user, and emotes that can be used and received.
    /// `user_emotes` is only used in the emote picker, and when the current user sends a message.
    /// `global_emotes` is used everywhere.
    pub user_emotes: RefCell<DownloadedEmotes>,
    pub global_emotes: RefCell<DownloadedEmotes>,
    /// Info about loaded emotes
    pub info: RefCell<HashMap<String, LoadedEmote>>,
    /// Terminal cell size in pixels: (width, height)
    pub cell_size: OnceCell<(f32, f32)>,
    /// Are emotes enabled?
    pub enabled: bool,
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
    pub fn new(enabled: bool) -> Self {
        Self {
            user_emotes: RefCell::default(),
            global_emotes: RefCell::default(),
            info: RefCell::default(),
            cell_size: OnceCell::default(),
            enabled,
        }
    }

    pub fn unload(&self) {
        self.info
            .borrow()
            .iter()
            .for_each(|(_, LoadedEmote { hash, .. })| {
                graphics_protocol::Clear(*hash).apply().unwrap_or_default();
            });
        self.user_emotes.borrow_mut().clear();
        self.global_emotes.borrow_mut().clear();
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

pub fn query_emotes(
    config: &CoreConfig,
    channel: String,
) -> OSReceiver<(DownloadedEmotes, DownloadedEmotes)> {
    let (tx, mut rx) = tokio::sync::oneshot::channel();

    if config.frontend.is_emotes_enabled() {
        let config = config.clone();
        tokio::spawn(async move { send_emotes(&config, tx, channel).await });
    } else {
        rx.close();
    }

    rx
}

pub async fn send_emotes(
    config: &CoreConfig,
    tx: OSSender<(DownloadedEmotes, DownloadedEmotes)>,
    channel: String,
) {
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
        let path = get_cache_dir().join(filename);
        let image = Image::builder()
            .id(hash)
            .name(word.to_string())
            .path(path)
            .overlay(overlay)
            .cell_w(cell_size.0)
            .cell_h(cell_size.1)
            .build()?;

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
        let path = get_cache_dir().join(filename);
        let image = Image::builder()
            .id(hash)
            .name(word.to_string())
            .path(path)
            .overlay(overlay)
            .cell_w(cell_size.0)
            .cell_h(cell_size.1)
            .build()?;

        let width = image.width;

        // Decode emote in another thread, to avoid blocking main thread as decoding images is slow.
        DECODE_EMOTE_SENDER
            .get()
            .ok_or_else(|| anyhow!("Decoding channel has not been initialized."))?
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

/// Initialize the emote decoder if emotes are enabled.
/// Returns a receiver for decoded emotes, or None if emotes are disabled or initialization failed.
#[allow(clippy::type_complexity)]
pub fn initialize_emote_decoder(
    config: &mut CoreConfig,
) -> Option<(mpsc::Receiver<Result<DecodedEmote, String>>, (f32, f32))> {
    if !config.frontend.is_emotes_enabled() {
        return None;
    }

    // We need to probe the terminal for it's size before starting the tui,
    // as writing on stdout on a different thread can interfere.
    match tui::crossterm::terminal::window_size() {
        Ok(size) => {
            let cell_size = (
                f32::from(size.width / size.columns),
                f32::from(size.height / size.rows),
            );

            let (decoder_tx, decoder_rx) = mpsc::channel(100);
            DECODE_EMOTE_SENDER.get_or_init(|| decoder_tx);

            let (decoded_tx, decoded_rx) = mpsc::channel(100);

            // As decoding an image is a blocking task, spawn a separate thread to handle it.
            // We cannot use tokio tasks here as it will create noticeable freezes.
            thread::spawn(move || decoder(decoder_rx, &decoded_tx));

            Some((decoded_rx, cell_size))
        }
        Err(e) => {
            config.frontend.twitch_emotes = false;
            config.frontend.betterttv_emotes = false;
            config.frontend.seventv_emotes = false;
            config.frontend.frankerfacez_emotes = false;
            warn!("Unable to query terminal for it's dimensions, disabling emotes. {e}");
            None
        }
    }
}
