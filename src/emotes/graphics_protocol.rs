use crate::utils::pathing::{
    create_temp_file, pathbuf_try_to_string, remove_temp_file, save_in_temp_file,
};
use anyhow::{anyhow, Context};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use crossterm::{csi, cursor::MoveTo, queue, Command};
use dialoguer::console::{Key, Term};
use image::codecs::gif::GifDecoder;
use image::io::Reader;
use image::{AnimationDecoder, ImageDecoder, ImageFormat};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::{env, fmt, fs};

/// Macro to add the graphics protocol escape sequence around a command.
/// See <https://sw.kovidgoyal.net/kitty/graphics-protocol/> for documentation of the terminal graphics protocol
macro_rules! gp {
    ($c:expr) => {
        concat!("\x1B_G", $c, "\x1b\\")
    };
}

const GP_PREFIX: &str = "twt.tty-graphics-protocol.";

type Result<T = ()> = anyhow::Result<T>;

pub trait Size {
    fn size(&self) -> (u32, u32);
}

pub struct Static {
    id: u32,
    width: u32,
    height: u32,
    path: PathBuf,
}

impl Static {
    pub fn new(id: u32, image: Reader<BufReader<File>>) -> Result<Self> {
        let image = image.decode()?.to_rgba8();
        let (width, height) = image.dimensions();
        let (mut tempfile, pathbuf) = create_temp_file(GP_PREFIX)?;
        if let Err(e) = save_in_temp_file(image.as_raw(), &mut tempfile) {
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

impl Command for Static {
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

impl Size for Static {
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

pub struct Gif {
    id: u32,
    width: u32,
    height: u32,
    frames: Vec<(PathBuf, u32)>,
}

impl Gif {
    pub fn new(id: u32, image: Reader<BufReader<File>>) -> Result<Self> {
        let decoder = GifDecoder::new(image.into_inner())?;
        let (width, height) = decoder.dimensions();
        let frames = decoder.into_frames().collect_frames()?;
        let iter = frames.iter();

        let (ok, err): (Vec<_>, Vec<_>) = iter
            .map(|f| {
                let (mut tempfile, pathbuf) = create_temp_file(GP_PREFIX)?;
                save_in_temp_file(f.buffer().as_raw(), &mut tempfile)?;
                let delay = f.delay().numer_denom_ms().0;

                Ok((pathbuf, delay))
            })
            .partition(Result::is_ok);

        let frames: Vec<(PathBuf, u32)> = ok.into_iter().filter_map(Result::ok).collect();

        // If we had any error, we need to delete the temp files, as the terminal won't do it for us.
        if !err.is_empty() {
            for (path, _) in &frames {
                let _ = fs::remove_file(path);
            }
            return Err(anyhow!("Invalid frame in gif."));
        }

        if frames.is_empty() {
            Err(anyhow!("Image has no frames"))
        } else {
            Ok(Self {
                id,
                width,
                height,
                frames,
            })
        }
    }
}

impl Command for Gif {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        if self.frames.is_empty() {
            return Err(fmt::Error);
        }

        let mut frames = self.frames.iter();

        // We need to send the data for the first frame as a normal image.
        // We can unwrap here because we checked above if frames was empty.
        let (path, delay) = frames.next().unwrap();
        write!(
            f,
            gp!("a=t,t=t,f=32,s={width},v={height},i={id},q=2;{path}"),
            id = self.id,
            width = self.width,
            height = self.height,
            path = STANDARD.encode(pathbuf_try_to_string(path).map_err(|_| fmt::Error)?)
        )?;
        // r=1: First frame
        write!(
            f,
            gp!("a=a,i={id},r=1,z={delay},q=2;"),
            id = self.id,
            delay = delay,
        )?;

        for (path, delay) in frames {
            write!(
                f,
                gp!("a=f,t=t,f=32,s={width},v={height},i={id},z={delay},q=2;{path}"),
                id = self.id,
                width = self.width,
                height = self.height,
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

impl Size for Gif {
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

pub enum Load {
    Static(Static),
    Gif(Gif),
}

impl Load {
    pub fn new(id: u32, path: &str) -> Result<Self> {
        let path = std::path::PathBuf::from(path);
        let image = Reader::open(path)?.with_guessed_format()?;

        match image.format() {
            None => Err(anyhow!("Could not guess image format.")),
            Some(ImageFormat::WebP) => Err(anyhow!("WebP image format is not supported.")),
            Some(ImageFormat::Gif) => Ok(Self::Gif(Gif::new(id, image)?)),
            Some(_) => Ok(Self::Static(Static::new(id, image)?)),
        }
    }
}

impl Command for Load {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        match self {
            Self::Static(s) => s.write_ansi(f),
            Self::Gif(g) => g.write_ansi(f),
        }
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> std::result::Result<(), std::io::Error> {
        panic!("Windows version not supported.")
    }
}

impl Size for Load {
    fn size(&self) -> (u32, u32) {
        match self {
            Self::Static(s) => s.size(),
            Self::Gif(g) => g.size(),
        }
    }
}

pub struct Display {
    id: u32,
    pid: u32,
    width: u32,
    offset: u32,
    x: u16,
    y: u16,
}

impl Display {
    pub const fn new((x, y): (u16, u16), id: u32, pid: u32, width: u32, offset: u32) -> Self {
        Self {
            id,
            pid,
            width,
            offset,
            x,
            y,
        }
    }
}

impl Command for Display {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        MoveTo(self.x, self.y).write_ansi(f)?;
        // r=1: Set height to 1 row
        write!(
            f,
            gp!("a=p,i={id},p={pid},r=1,c={width},X={offset},q=2;"),
            id = self.id,
            pid = self.pid,
            width = self.width,
            offset = self.offset
        )
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> std::result::Result<(), std::io::Error> {
        panic!("Windows version not supported.")
    }
}

#[derive(Eq, PartialEq)]
pub struct Clear(pub u32, pub u32);

impl Command for Clear {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        if *self == Self(0, 0) {
            // Delete all images
            write!(f, gp!("a=d,d=A,q=2;"))
        } else {
            write!(
                f,
                gp!("a=d,d=i,i={id},p={pid},q=2;"),
                id = self.0,
                pid = self.1,
            )
        }
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> std::result::Result<(), std::io::Error> {
        panic!("Windows version not supported.")
    }
}

fn query_terminal(command: &[u8]) -> Result<String> {
    let c = *command.last().context("Command is empty")? as char;
    let mut stdout = Term::stdout();
    stdout.write_all(command)?;
    stdout.flush()?;

    loop {
        let c = stdout.read_key()?;
        if let Key::UnknownEscSeq(_) = c {
            break;
        }
    }

    let mut response = String::new();
    while !response.contains(c) {
        match stdout.read_key() {
            Ok(Key::Char(chr)) => response.push(chr),
            Ok(Key::UnknownEscSeq(esc)) => response.extend(esc),
            Err(_) => break,
            _ => (),
        }
    }
    Ok(response)
}

pub fn get_terminal_cell_size() -> Result<(u32, u32)> {
    let mut res = query_terminal(csi!("14t").as_bytes())?;

    // Response is the terminal size in pixels, with format <height>;<width>t
    res.pop();
    let mut values = res.split(';');
    let height_px = values
        .next()
        .context("Invalid response from terminal")?
        .parse::<u32>()?;
    let width_px = values
        .next()
        .context("Invalid response from terminal")?
        .parse::<u32>()?;

    // Size of terminal: (columns, rows)
    let (ncols, nrows) = crossterm::terminal::size()?;

    Ok((width_px / u32::from(ncols), height_px / u32::from(nrows)))
}

pub fn support_graphics_protocol() -> Result<bool> {
    Ok(
        (env::var("TERM")? == "xterm-kitty" || env::var("TERM_PROGRAM")? == "WezTerm")
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

pub fn command(c: impl Command) -> Result {
    Ok(queue!(std::io::stdout(), c)?)
}
