use base64::{engine::general_purpose::STANDARD, Engine};
use color_eyre::{
    eyre::{anyhow, ContextCompat, Error},
    Result,
};
use crossterm::{csi, queue, Command};
use dialoguer::console::{Key, Term};
use image::{
    codecs::{gif::GifDecoder, webp::WebPDecoder},
    imageops::FilterType,
    io::Reader,
    AnimationDecoder, DynamicImage, GenericImageView, ImageDecoder, ImageFormat, Rgba,
};
use std::{
    env, fmt, fs,
    fs::File,
    io::{BufReader, Write},
    path::PathBuf,
};

use crate::utils::pathing::{
    create_temp_file, pathbuf_try_to_string, remove_temp_file, save_in_temp_file,
};

/// Macro to add the graphics protocol escape sequence around a command.
/// See <https://sw.kovidgoyal.net/kitty/graphics-protocol/> for documentation of the terminal graphics protocol
macro_rules! gp {
    ($c:expr) => {
        concat!("\x1B_G", $c, "\x1b\\")
    };
}

/// The temporary files created for the graphics protocol need to have the `tty-graphics-protocol`
/// string to be deleted by the terminal.
const GP_PREFIX: &str = "twt.tty-graphics-protocol.";

pub trait Size {
    fn width(&self) -> u32;

    fn calculate_resize_ratio(height: u32, cell_h: f32) -> f32 {
        cell_h / height as f32
    }
}

pub struct StaticImage {
    id: u32,
    width: u32,
    height: u32,
    path: PathBuf,
}

impl StaticImage {
    pub fn new(id: u32, image: Reader<BufReader<File>>, cell_size: (f32, f32)) -> Result<Self> {
        let image = image.decode()?;
        let (width, height) = image.dimensions();
        let ratio = Self::calculate_resize_ratio(height, cell_size.1);

        let image = image.resize(
            (width as f32 * ratio) as u32,
            cell_size.1 as u32,
            FilterType::Lanczos3,
        );
        let (width, height) = image.dimensions();

        let (mut tempfile, pathbuf) = create_temp_file(GP_PREFIX)?;
        if let Err(e) = save_in_temp_file(image.to_rgba8().as_raw(), &mut tempfile) {
            remove_temp_file(&pathbuf);
            return Err(e);
        }

        Ok(Self {
            id,
            width,
            height,
            path: pathbuf,
        })
    }
}

impl Command for StaticImage {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        write!(
            f,
            gp!("a=t,t=t,f=32,s={width},v={height},i={id},q=2;{path}"),
            width = self.width,
            height = self.height,
            id = self.id,
            path = STANDARD.encode(pathbuf_try_to_string(&self.path).map_err(|_| fmt::Error)?)
        )
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> std::result::Result<(), std::io::Error> {
        panic!("Windows version not supported.")
    }
}

impl Size for StaticImage {
    fn width(&self) -> u32 {
        self.width
    }
}

pub struct AnimatedImage {
    id: u32,
    width: u32,
    frames: Vec<(PathBuf, u32, u32, u32)>,
}

impl AnimatedImage {
    pub fn new<'a>(
        id: u32,
        decoder: impl ImageDecoder<'a> + AnimationDecoder<'a>,
        cell_size: (f32, f32),
    ) -> Result<Self> {
        let (width, height) = decoder.dimensions();
        let resize_ratio = Self::calculate_resize_ratio(height, cell_size.1);
        let frames = decoder.into_frames();

        let (ok, err): (Vec<_>, Vec<_>) = frames
            .map(|f| {
                let frame = f?;
                let delay = frame.delay().numer_denom_ms().0;
                let image = DynamicImage::from(frame.into_buffer());
                let (w, h) = image.dimensions();
                let image = image.resize(
                    (w as f32 * resize_ratio).ceil() as u32,
                    (h as f32 * resize_ratio).ceil() as u32,
                    FilterType::Lanczos3,
                );
                let (w, h) = image.dimensions();
                let (mut tempfile, pathbuf) = create_temp_file(GP_PREFIX)?;
                save_in_temp_file(image.to_rgba8().as_raw(), &mut tempfile)?;

                Ok::<(PathBuf, u32, u32, u32), Error>((pathbuf, delay, w, h))
            })
            .partition(Result::is_ok);

        let frames: Vec<(PathBuf, u32, u32, u32)> = ok.into_iter().flatten().collect();

        // If we had any error, we need to delete the temp files, as the terminal won't do it for us.
        if !err.is_empty() {
            for (path, ..) in &frames {
                drop(fs::remove_file(path));
            }
            return Err(anyhow!("Invalid frame in gif."));
        }

        if frames.is_empty() {
            Err(anyhow!("Image has no frames"))
        } else {
            Ok(Self {
                id,
                width: (width as f32 * resize_ratio) as u32,
                frames,
            })
        }
    }
}

impl Command for AnimatedImage {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        if self.frames.is_empty() {
            return Err(fmt::Error);
        }

        let mut frames = self.frames.iter();

        // We need to send the data for the first frame as a normal image.
        // We can unwrap here because we checked above if frames was empty.
        let (path, delay, width, height) = frames.next().unwrap();

        write!(
            f,
            gp!("a=t,t=t,f=32,s={width},v={height},i={id},q=2;{path}"),
            id = self.id,
            width = width,
            height = height,
            path = STANDARD.encode(pathbuf_try_to_string(path).map_err(|_| fmt::Error)?)
        )?;
        // r=1: First frame
        write!(
            f,
            gp!("a=a,i={id},r=1,z={delay},q=2;"),
            id = self.id,
            delay = delay,
        )?;

        for (path, delay, width, height) in frames {
            write!(
                f,
                gp!("a=f,t=t,f=32,s={width},v={height},i={id},z={delay},q=2;{path}"),
                id = self.id,
                width = width,
                height = height,
                delay = delay,
                path = STANDARD.encode(pathbuf_try_to_string(path).map_err(|_| fmt::Error)?)
            )?;
        }

        // s=3: Start animation, v=1: Loop infinitely
        write!(f, gp!("a=a,i={id},s=3,v=1,q=2;"), id = self.id)
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> std::result::Result<(), std::io::Error> {
        panic!("Windows version not supported.")
    }
}

impl Size for AnimatedImage {
    fn width(&self) -> u32 {
        self.width
    }
}

pub enum Load {
    Static(StaticImage),
    Animated(AnimatedImage),
}

impl Load {
    pub fn new(id: u32, path: &str, cell_size: (f32, f32)) -> Result<Self> {
        let path = std::path::PathBuf::from(path);
        let image = Reader::open(&path)?.with_guessed_format()?;

        match image.format() {
            None => Err(anyhow!("Could not guess image format.")),
            Some(ImageFormat::WebP) => {
                let mut decoder = WebPDecoder::new(image.into_inner())?;

                if decoder.has_animation() {
                    // Some animated webp images have a default white background color
                    // We replace it by a transparent background
                    decoder.set_background_color(Rgba([0, 0, 0, 0]))?;
                    Ok(Self::Animated(AnimatedImage::new(id, decoder, cell_size)?))
                } else {
                    let image = Reader::open(&path)?.with_guessed_format()?;

                    Ok(Self::Static(StaticImage::new(id, image, cell_size)?))
                }
            }
            Some(ImageFormat::Gif) => {
                let decoder = GifDecoder::new(image.into_inner())?;
                Ok(Self::Animated(AnimatedImage::new(id, decoder, cell_size)?))
            }
            Some(_) => Ok(Self::Static(StaticImage::new(id, image, cell_size)?)),
        }
    }
}

impl Command for Load {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        match self {
            Self::Static(s) => s.write_ansi(f),
            Self::Animated(a) => a.write_ansi(f),
        }
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> std::result::Result<(), std::io::Error> {
        panic!("Windows version not supported.")
    }
}

impl Size for Load {
    fn width(&self) -> u32 {
        match self {
            Self::Static(s) => s.width(),
            Self::Animated(a) => a.width(),
        }
    }
}

pub struct Clear;

impl Command for Clear {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        write!(f, gp!("a=d,d=A,q=2;"))
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> std::result::Result<(), std::io::Error> {
        panic!("Windows version not supported.")
    }
}

pub struct Display {
    id: u32,
    pid: u32,
    cols: u16,
}

impl Display {
    pub const fn new(id: u32, pid: u32, cols: u16) -> Self {
        Self { id, pid, cols }
    }
}

impl Command for Display {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        // r=1: Set height to 1 row
        write!(
            f,
            gp!("a=p,U=1,i={id},p={pid},r=1,c={cols},q=2;"),
            id = self.id,
            pid = self.pid,
            cols = self.cols
        )
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> std::result::Result<(), std::io::Error> {
        panic!("Windows version not supported.")
    }
}
pub struct Chain {
    id: u32,
    pid: u32,
    parent_id: u32,
    parent_placement_id: u32,
    z: u32,
    col_offset: i32,
    pixel_offset: u16,
}

impl Chain {
    pub const fn new(
        id: u32,
        pid: u32,
        (parent_id, parent_placement_id): (u32, u32),
        z: u32,
        col_offset: i32,
        pixel_offset: u16,
    ) -> Self {
        Self {
            id,
            pid,
            parent_id,
            parent_placement_id,
            z,
            col_offset,
            pixel_offset,
        }
    }
}

impl Command for Chain {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        write!(
            f,
            gp!("a=p,i={id},p={pid},P={parent_id},Q={parent_pid},z={z},H={co},X={pxo},q=2;"),
            id = self.id,
            pid = self.pid,
            parent_id = self.parent_id,
            parent_pid = self.parent_placement_id,
            z = self.z,
            co = self.col_offset,
            pxo = self.pixel_offset
        )
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> std::result::Result<(), std::io::Error> {
        panic!("Windows version not supported.")
    }
}

pub trait ApplyCommand: Command {
    fn apply(&self) -> Result<()> {
        Ok(queue!(std::io::stdout(), self)?)
    }
}

impl<T: Command> ApplyCommand for T {}

/// Send a csi query to the terminal. The terminal must respond in the format `<ESC>[(a)(r)(c)`,
/// where `(a)` can be any character, `(r)` is the terminal response, and `(c)` is the last character of the query.
/// If the terminal does not respond, or responds in a different format, this function will cause an infinite loop.
/// See [here](https://www.xfree86.org/current/ctlseqs.html) for information about xterm control sequences.
/// This function will strip the `<ESC>[(a)` and the last char `(c)` and return the response `(r)`.
fn query_terminal(command: &[u8]) -> Result<String> {
    let c = *command.last().context("Command is empty")? as char;
    let mut stdout = Term::stdout();
    stdout.write_all(command)?;
    stdout.flush()?;

    // Empty stdout buffer until we find the terminal response.
    loop {
        if let Key::UnknownEscSeq(_) = stdout.read_key()? {
            break;
        }
    }

    let mut response = String::new();
    loop {
        match stdout.read_key() {
            Ok(Key::Char(chr)) if chr == c => break,
            Ok(Key::Char(chr)) => response.push(chr),
            Err(_) => break,
            _ => (),
        }
    }
    Ok(response)
}

/// First check if the terminal is `kitty`, this is the only terminal that supports the graphics protocol using unicode placeholders as of 2023-07-13.
/// Then check that it supports the graphics protocol using temporary files, by sending a graphics protocol request followed by a request for terminal attributes.
/// If we receive the terminal attributes without receiving the response for the graphics protocol, it does not support it.
pub fn support_graphics_protocol() -> Result<bool> {
    Ok(env::var("TERM")? == "xterm-kitty"
        && query_terminal(
            format!(
                concat!(gp!("i=31,s=1,v=1,a=q,t=d,f=24;{}"), csi!("c")),
                STANDARD.encode("AAAA"),
            )
            .as_bytes(),
        )?
        .contains("OK"))
}
