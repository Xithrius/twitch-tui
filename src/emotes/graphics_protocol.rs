use std::{env, fmt, io::Write, path::PathBuf};

use base64::{Engine, engine::general_purpose::STANDARD};
use color_eyre::{
    Result,
    eyre::{ContextCompat, anyhow},
};
use dialoguer::console::{Key, Term};
use image::{
    AnimationDecoder, DynamicImage, GenericImageView, ImageDecoder, ImageFormat, ImageReader, Rgba,
    RgbaImage,
    codecs::{gif::GifDecoder, webp::WebPDecoder},
    imageops::FilterType,
};
use tui::crossterm::{Command, csi, queue};

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
struct AnimatedDecoder<T: for<'a> AnimationDecoder<'a> + Send>(T);

trait IntoFrames<'a>: Send {
    fn frames(self: Box<Self>) -> Box<dyn Iterator<Item = Result<(RgbaImage, u32)>> + 'a>;
}

impl<'a> IntoFrames<'a> for StaticDecoder {
    fn frames(self: Box<Self>) -> Box<dyn Iterator<Item = Result<(RgbaImage, u32)>> + 'a> {
        Box::new(std::iter::once(Ok((self.0.to_rgba8(), 0))))
    }
}

impl<'a, T: for<'b> AnimationDecoder<'b> + Send> IntoFrames<'a> for AnimatedDecoder<T> {
    fn frames(self: Box<Self>) -> Box<dyn Iterator<Item = Result<(RgbaImage, u32)>> + 'a> {
        Box::new(self.0.into_frames().map(|f| {
            let frame = f?;

            let delay = frame.delay().numer_denom_ms().0;

            Ok((frame.into_buffer(), delay))
        }))
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
    pub const fn id(&self) -> u32 {
        self.id
    }

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

pub struct Image<'a> {
    pub name: String,
    id: u32,
    pub width: u32,
    ratio: f32,
    overlay: bool,
    pub cols: u16,
    decoder: Box<dyn IntoFrames<'a> + 'a>,
}

impl<'a> Image<'a> {
    pub fn new(
        id: u32,
        name: String,
        path: &str,
        overlay: bool,
        (cell_w, cell_h): (f32, f32),
    ) -> Result<Self> {
        let path = std::path::PathBuf::from(path);
        let image = ImageReader::open(path)?.with_guessed_format()?;

        let (width, height, decoder) = match image.format() {
            None => return Err(anyhow!("Could not guess image format.")),
            Some(ImageFormat::WebP) => {
                let mut decoder = WebPDecoder::new(image.into_inner())?;
                let (width, height) = decoder.dimensions();

                if decoder.has_animation() {
                    decoder.set_background_color(Rgba([0; 4]))?;

                    (
                        width,
                        height,
                        Box::new(AnimatedDecoder(decoder)) as Box<dyn IntoFrames + 'a>,
                    )
                } else {
                    let img = DynamicImage::from_decoder(decoder)?;

                    (
                        width,
                        height,
                        Box::new(StaticDecoder(img)) as Box<dyn IntoFrames + 'a>,
                    )
                }
            }
            Some(ImageFormat::Gif) => {
                let decoder = GifDecoder::new(image.into_inner())?;
                let (width, height) = decoder.dimensions();

                (
                    width,
                    height,
                    Box::new(AnimatedDecoder(decoder)) as Box<dyn IntoFrames + 'a>,
                )
            }
            Some(_) => {
                let image = image.decode()?;
                let (width, height) = image.dimensions();

                (
                    width,
                    height,
                    Box::new(StaticDecoder(image)) as Box<dyn IntoFrames + 'a>,
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
    // While the structs that implement this trait can be sent between thread safely, this function is not thread safe,
    // and needs to be called from the main thread.
    // A solution would be to call it with `std::io::stdout().lock()`, but this creates noticeable freezes when commands
    // are issued and the user is typing.
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

const SUPPORTED_TERMINALS: [&str; 2] = ["xterm-kitty", "xterm-ghostty"];

/// First check if the terminal is `kitty`, this is the only terminal that supports the graphics protocol using unicode placeholders as of 2023-07-13.
/// Then check that it supports the graphics protocol using temporary files, by sending a graphics protocol request followed by a request for terminal attributes.
/// If we receive the terminal attributes without receiving the response for the graphics protocol, it does not support it.
pub fn support_graphics_protocol() -> Result<bool> {
    Ok(
        env::var("TERM").is_ok_and(|term| SUPPORTED_TERMINALS.contains(&term.as_str()))
            && query_terminal(
                format!(
                    concat!(gp!("i=31,s=1,v=1,a=q,t=d,f=24;{}"), csi!("c")),
                    STANDARD.encode("AAAA"),
                )
                .as_bytes(),
            )?
            .contains("OK"),
    )
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
