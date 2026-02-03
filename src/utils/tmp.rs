use std::{
    fs::{File, remove_file},
    io::Write,
    mem::drop,
    path::{Path, PathBuf},
};

use color_eyre::Result;

pub fn create_tmp_file(prefix: &str) -> Result<(File, PathBuf)> {
    let (tempfile, pathbuf) = tempfile::Builder::new()
        .prefix(prefix)
        .rand_bytes(5)
        .tempfile()?
        .keep()?;

    Ok((tempfile, pathbuf))
}

pub fn save_in_tmp_file(buffer: &[u8], file: &mut File) -> Result<()> {
    file.write_all(buffer)?;
    file.flush()?;
    Ok(())
}

pub fn remove_tmp_file(path: &Path) {
    drop(remove_file(path));
}
