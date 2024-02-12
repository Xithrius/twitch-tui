use base64::{engine::general_purpose::STANDARD, Engine};
use color_eyre::{
    eyre::{anyhow, ContextCompat},
    Result,
};
use crossterm::{csi, queue, Command};
use dialoguer::console::{Key, Term};
use image::{
    codecs::gif::GifDecoder, imageops::FilterType, io::Reader, AnimationDecoder, DynamicImage,
    GenericImageView, ImageDecoder, ImageFormat, RgbaImage,
};
use std::{
    env, fmt,
    fs::File,
    io::{BufReader, Read, Write},
    path::PathBuf,
};
use webp_animation::Decoder;

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

struct StaticDecoder(DynamicImage);
struct AnimatedDecoder(GifDecoder<BufReader<File>>);
struct WebPDecoder(Vec<u8>);

trait IntoFrames: Send {
    fn frames(self: Box<Self>) -> Box<dyn Iterator<Item = Result<(RgbaImage, u32)>>>;
}

impl IntoFrames for StaticDecoder {
    fn frames(self: Box<Self>) -> Box<dyn Iterator<Item = Result<(RgbaImage, u32)>>> {
        Box::new(std::iter::once(Ok((self.0.to_rgba8(), 0))))
    }
}

impl IntoFrames for AnimatedDecoder {
    fn frames(self: Box<Self>) -> Box<dyn Iterator<Item = Result<(RgbaImage, u32)>>> {
        Box::new(self.0.into_frames().map(|f| {
            let frame = f?;

            let delay = frame.delay().numer_denom_ms().0;

            Ok((frame.into_buffer(), delay))
        }))
    }
}

impl IntoFrames for WebPDecoder {
    fn frames(self: Box<Self>) -> Box<dyn Iterator<Item = Result<(RgbaImage, u32)>>> {
        let Ok(decoder) = Decoder::new(&self.0) else {
            return Box::new(std::iter::empty());
        };

        let mut timestamp = 0;
        Box::new(
            decoder
                .into_iter()
                .collect::<Vec<_>>()
                .into_iter()
                .map(move |frame| {
                    let current_timestamp = frame.timestamp();

                    let delay = (current_timestamp - timestamp) as u32;

                    timestamp = current_timestamp;

                    Ok((frame.into_rgba_image()?, delay))
                }),
        )
    }
}

pub struct DecodedImage {
    width: u32,
    height: u32,
    path: PathBuf,
    delay: u32,
}

pub struct DecodedEmote {
    id: u32,
    cols: u16,
    images: Vec<DecodedImage>,
}

impl DecodedEmote {
    #[allow(unused)]
    pub const fn id(&self) -> u32 {
        self.id
    }

    #[allow(unused)]
    pub const fn cols(&self) -> u16 {
        self.cols
    }
}

impl Command for DecodedEmote {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        if self.images.is_empty() {
            return Err(fmt::Error);
        }

        let mut frames = self.images.iter();

        // Sending a static image and an animated one is done with the same command for the first frame.
        // We can unwrap here because we checked above if frames was empty.
        let DecodedImage {
            path,
            delay,
            width,
            height,
        } = frames.next().unwrap();

        write!(
            f,
            gp!("a=t,t=t,f=32,s={width},v={height},i={id},q=2;{path}"),
            id = self.id,
            width = width,
            height = height,
            path = STANDARD.encode(pathbuf_try_to_string(path).map_err(|_| fmt::Error)?)
        )?;

        if self.images.len() == 1 {
            return Ok(());
        }

        // r=1: First frame
        write!(
            f,
            gp!("a=a,i={id},r=1,z={delay},q=2;"),
            id = self.id,
            delay = delay,
        )?;

        for DecodedImage {
            path,
            delay,
            width,
            height,
        } in frames
        {
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

pub struct Image {
    pub name: String,
    id: u32,
    pub width: u32,
    ratio: f32,
    overlay: bool,
    pub cols: u16,
    decoder: Box<dyn IntoFrames>,
}

impl Image {
    pub fn new(
        id: u32,
        name: String,
        path: &str,
        overlay: bool,
        (cell_w, cell_h): (f32, f32),
    ) -> Result<Self> {
        let path = std::path::PathBuf::from(path);
        let image = Reader::open(path)?.with_guessed_format()?;

        let (width, height, decoder) = match image.format() {
            None => return Err(anyhow!("Could not guess image format.")),
            Some(ImageFormat::WebP) => {
                let mut reader = image.into_inner();
                let mut buffer = vec![];
                reader.read_to_end(&mut buffer)?;

                let decoder = Decoder::new(&buffer)?;
                let (width, height) = decoder.dimensions();

                (
                    width,
                    height,
                    Box::new(WebPDecoder(buffer)) as Box<dyn IntoFrames>,
                )
            }
            Some(ImageFormat::Gif) => {
                let decoder = GifDecoder::new(image.into_inner())?;
                let (width, height) = decoder.dimensions();

                (
                    width,
                    height,
                    Box::new(AnimatedDecoder(decoder)) as Box<dyn IntoFrames>,
                )
            }
            Some(_) => {
                let image = image.decode()?;
                let (width, height) = image.dimensions();

                (
                    width,
                    height,
                    Box::new(StaticDecoder(image)) as Box<dyn IntoFrames>,
                )
            }
        };

        let ratio = cell_h / height as f32;
        let width = (width as f32 * ratio).round() as u32;
        let cols = (width as f32 / cell_w).ceil() as u16;

        Ok(Self {
            name,
            id,
            width,
            ratio,
            overlay,
            cols,
            decoder,
        })
    }

    pub fn decode(self) -> Result<DecodedEmote> {
        let frames = self.decoder.frames().map(|f| {
            let (image, delay) = f?;
            let image = if self.overlay {
                let (w, h) = image.dimensions();
                image::imageops::resize(
                    &image,
                    (w as f32 * self.ratio).round() as u32,
                    (h as f32 * self.ratio).round() as u32,
                    FilterType::Lanczos3,
                )
            } else {
                image
            };

            let (width, height) = image.dimensions();
            let (mut tempfile, path) = create_temp_file(GP_PREFIX)?;
            if let Err(e) = save_in_temp_file(image.as_raw(), &mut tempfile) {
                remove_temp_file(&path);
                return Err(e);
            }

            Ok(DecodedImage {
                width,
                height,
                path,
                delay,
            })
        });

        let mut images = vec![];
        for f in frames {
            match f {
                Ok(i) => images.push(i),
                Err(e) => {
                    for DecodedImage { path, .. } in images {
                        remove_temp_file(&path);
                    }

                    return Err(anyhow!("Unable to decode frame: {e}"));
                }
            }
        }

        if images.is_empty() {
            return Err(anyhow!("Image has no frames"));
        }

        Ok(DecodedEmote {
            id: self.id,
            cols: self.cols,
            images,
        })
    }
}

pub struct Clear(pub u32);

impl Command for Clear {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        write!(f, gp!("a=d,d=I,i={id},q=2;"), id = self.0)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_static_image() {
        let mut s = String::new();

        let path = "/tmp/foo/bar.baz";
        let image = DecodedImage {
            width: 30,
            height: 42,
            path: path.into(),
            delay: 0,
        };

        let emote = DecodedEmote {
            id: 1,
            cols: 3,
            images: vec![image],
        };
        emote.write_ansi(&mut s).unwrap();

        assert_eq!(
            s,
            format!(
                gp!("a=t,t=t,f=32,s=30,v=42,i=1,q=2;{path}"),
                path = STANDARD.encode(path)
            )
        );
    }

    #[test]
    fn load_animated_image() {
        let mut s = String::new();

        let paths = ["/tmp/foo/bar.baz", "a/b/c", "foo bar-baz/123"];

        let images = vec![
            DecodedImage {
                width: 30,
                height: 42,
                path: paths[0].into(),
                delay: 0,
            },
            DecodedImage {
                width: 12,
                height: 74,
                path: paths[1].into(),
                delay: 89,
            },
            DecodedImage {
                width: 54,
                height: 45,
                path: paths[2].into(),
                delay: 4,
            },
        ];

        let emote = DecodedEmote {
            id: 1,
            cols: 3,
            images,
        };

        emote.write_ansi(&mut s).unwrap();

        assert_eq!(
            s,
            format!(
                gp!("a=t,t=t,f=32,s=30,v=42,i=1,q=2;{path}"),
                path = STANDARD.encode(paths[0])
            ) + gp!("a=a,i=1,r=1,z=0,q=2;")
                + &format!(
                    gp!("a=f,t=t,f=32,s=12,v=74,i=1,z=89,q=2;{path}"),
                    path = STANDARD.encode(paths[1])
                )
                + &format!(
                    gp!("a=f,t=t,f=32,s=54,v=45,i=1,z=4,q=2;{path}"),
                    path = STANDARD.encode(paths[2])
                )
                + gp!("a=a,i=1,s=3,v=1,q=2;")
        );
    }

    #[test]
    fn clear_image() {
        let mut s = String::new();

        Clear(1).write_ansi(&mut s).unwrap();

        assert_eq!(s, gp!("a=d,d=I,i=1,q=2;"));
    }

    #[test]
    fn display_image() {
        let mut s = String::new();

        Display::new(1, 2, 3).write_ansi(&mut s).unwrap();

        assert_eq!(s, gp!("a=p,U=1,i=1,p=2,r=1,c=3,q=2;"));
    }

    #[test]
    fn overlay_image() {
        let mut s = String::new();
        Chain::new(1, 2, (3, 4), 1, 1, 4)
            .write_ansi(&mut s)
            .unwrap();

        assert_eq!(s, gp!("a=p,i=1,p=2,P=3,Q=4,z=1,H=1,X=4,q=2;"));
    }
}
