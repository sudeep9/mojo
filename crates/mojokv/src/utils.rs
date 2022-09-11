use std::path::Path;
use std::io::{Read, Write};

use crate::Error;

pub fn load_file(path: &Path, buf: &mut Vec<u8>) -> Result<(), Error> {
    let mut f = std::fs::OpenOptions::new().read(true).open(path)?;
    f.read_to_end(buf)?;
    Ok(())
}

pub fn write_file(path: &Path, buf: &[u8]) -> Result<(), Error> {
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    f.write_all(buf)?;
    f.sync_data()?;

    Ok(())
}

pub fn touch_file(path: &Path) -> Result<(), Error> {
    log::debug!("creating init file: {:?}", path);
    let _ = std::fs::File::create(path)?;

    log::debug!("creating init file done");
    Ok(())
}

/*
pub fn read_le_u32<R: std::io::Read>(r: &mut R) -> Result<u32, Error> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub fn read_le_isize<R: std::io::Read>(r: &mut R) -> Result<isize, Error> {
    let mut buf = [0u8; std::mem::size_of::<isize>()];
    r.read_exact(&mut buf)?;
    Ok(isize::from_le_bytes(buf))
}

pub fn read_le_usize<R: std::io::Read>(r: &mut R) -> Result<usize, Error> {
    let mut buf = [0u8; std::mem::size_of::<isize>()];
    r.read_exact(&mut buf)?;
    Ok(usize::from_le_bytes(buf))
}
*/