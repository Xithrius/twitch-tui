use crate::handlers::data::EmoteData;
use crate::utils::pathing::{create_temp_file, save_in_temp_file};
use base64::Engine;
use dialoguer::console::{Key, Term};
use image::codecs::gif::GifDecoder;
use image::io::Reader;
use image::{AnimationDecoder, ImageDecoder, ImageFormat};
use std::env;
use std::fs::File;
use std::io::{BufReader, Write};

pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const KITTY_PREFIX: &str = "twt.tty-graphics-protocol.";
const ESC: char = '\x1b';
const PROTOCOL_START: &str = "\x1b_G";
const PROTOCOL_END: &str = "\x1b\\";

fn query_terminal(command: &[u8]) -> Result<String> {
    let c = *command.last().ok_or("")? as char;
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
    let mut res = query_terminal(format!("{ESC}[14t").as_bytes())?;
    // Response is in format <height>;<width>t
    res.pop();
    let mut values = res.split(';');
    let height = values
        .next()
        .ok_or("Invalid response from terminal")?
        .parse::<u32>()?;
    let width = values
        .next()
        .ok_or("Invalid response from terminal")?
        .parse::<u32>()?;

    let size = crossterm::terminal::size()?;

    Ok((width / u32::from(size.0), height / u32::from(size.1)))
}

pub fn support_kitty() -> Result<bool> {
    Ok(
        (env::var("TERM")? == "xterm-kitty" || env::var("TERM_PROGRAM")? == "WezTerm")
            && query_terminal(
                format!("{PROTOCOL_START}i=31,s=1,v=1,a=q,t=d,f=24;AAAA{PROTOCOL_END}{ESC}[c")
                    .as_bytes(),
            )?
            .contains("OK"),
    )
}

/// See <https://sw.kovidgoyal.net/kitty/graphics-protocol/> for documentation of the terminal graphics protocol
fn send_graphics_command(stdout: &mut impl Write, command: &str, payload: Option<&str>) -> Result {
    let data = base64::engine::general_purpose::STANDARD.encode(payload.unwrap_or_default());
    let command = format!("{PROTOCOL_START}{command};{data}{PROTOCOL_END}");

    stdout.write_all(command.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

fn move_cursor(stdout: &mut impl Write, col: u32, row: u32) -> Result {
    let binding = format!("\x1b[{};{}H", row + 1, col + 1);
    stdout.write_all(binding.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

fn load_animated(
    stdout: &mut impl Write,
    id: u32,
    image: Reader<BufReader<File>>,
) -> Result<(u32, u32)> {
    let decoder = GifDecoder::new(image.into_inner())?;
    let (width, height) = decoder.dimensions();

    let mut frames = decoder.into_frames();

    let first = frames.next();
    if let Some(frame) = first {
        let frame = frame?;
        let (mut tempfile, pathbuf) = create_temp_file(KITTY_PREFIX)?;
        save_in_temp_file(frame.buffer().as_raw(), &mut tempfile)?;

        let command = format!("a=t,t=t,f=32,s={width},v={height},i={id},q=2");
        send_graphics_command(stdout, &command, pathbuf.to_str())?;
        let delay = frame.delay().numer_denom_ms().0;
        let command = format!("a=a,i={id},r=1,z={delay},q=2");
        send_graphics_command(stdout, &command, None)?;
    }

    let frames = frames.collect_frames()?;
    for frame in &frames {
        let (mut tempfile, pathbuf) = create_temp_file(KITTY_PREFIX)?;
        save_in_temp_file(frame.buffer().as_raw(), &mut tempfile)?;

        let delay = frame.delay().numer_denom_ms().0;
        let command = format!("a=f,t=t,f=32,s={width},v={height},i={id},z={delay},q=2");
        send_graphics_command(stdout, &command, pathbuf.to_str())?;
    }
    let command = format!("a=a,i={id},s=3,v=1,q=2");
    send_graphics_command(stdout, &command, None)?;
    Ok((width, height))
}

pub fn load(stdout: &mut impl Write, id: u32, path: &str) -> Result<(u32, u32)> {
    let path = std::path::PathBuf::from(path);
    let image = Reader::open(path)?.with_guessed_format()?;

    match image.format() {
        Some(ImageFormat::Gif) => load_animated(stdout, id, image),
        Some(ImageFormat::WebP) => Err("WebP image format not supported, skipping...".into()),
        Some(_) => {
            let image = image.decode()?.to_rgba8();
            let (width, height) = image.dimensions();
            let (mut tempfile, pathbuf) = create_temp_file(KITTY_PREFIX)?;
            save_in_temp_file(image.as_raw(), &mut tempfile)?;

            let command = format!("a=t,t=t,f=32,s={width},v={height},i={id},q=2");
            send_graphics_command(stdout, &command, pathbuf.to_str())?;
            Ok((width, height))
        }
        None => Err("Could not guess image format, skipping...".into()),
    }
}

pub fn display(
    stdout: &mut impl Write,
    EmoteData {
        kitty_id: (id, pid),
        width,
        offset,
        ..
    }: &EmoteData,
    (x, y): (u32, u32),
) -> Result {
    move_cursor(stdout, x, y)?;

    send_graphics_command(
        stdout,
        &format!("a=p,i={id},p={pid},r=1,c={width},X={offset},q=2"),
        None,
    )
}

pub fn clear(stdout: &mut impl Write, id: u32, placement: u32) -> Result {
    send_graphics_command(stdout, &format!("a=d,d=i,i={id},p={placement},q=2"), None)
}
