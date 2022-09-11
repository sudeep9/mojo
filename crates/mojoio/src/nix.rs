use std::path::Path;

use nix::fcntl::{self, OFlag};
use nix::sys::stat::Mode;

//use crate::value;
use crate::Error;

pub struct NixFile {
    file_fd: i32,
    curr_off: u64,
    page_header_buf: [u8; crate::PAGE_HEADER_LEN],
    page_header: PageHeader,
}

impl NixFile {
    pub fn open(filepath: &Path, _file_no: u32) -> Result<Self, Error> {
        let open_flags = OFlag::O_CREAT|OFlag::O_RDWR;
        let file_perm = Mode::all();
        let file_fd = fcntl::open(filepath, open_flags, file_perm)?;

        let curr_off = nix::unistd::lseek(file_fd, 0, nix::unistd::Whence::SeekEnd)? as u64;

        log::debug!("open path={:?} fd={}", filepath, file_fd);

        Ok(NixFile {
            file_fd,
            curr_off,
            page_header_buf: [0; crate::PAGE_HEADER_LEN],
            page_header: PageHeader::new(), 
        })
    }

    pub fn close(&mut self) -> Result<(), Error> {
        log::debug!("close fd={}", self.file_fd);
        nix::unistd::close(self.file_fd)?;
        Ok(())
    }

    pub fn write_buf_at(&mut self, off: u64, block_no: u32, buf: &[u8]) -> Result<(), Error> {
        self.page_header.block_no = block_no;
        self.page_header.encode(&mut self.page_header_buf);

        let header_io = std::io::IoSlice::new(&self.page_header_buf);
        let buf_io = std::io::IoSlice::new(buf);

        let io_bufs = [header_io, buf_io];

        log::debug!("file write at fd={} off={} {}", self.file_fd, off, buf.len());
        let n = nix::sys::uio::pwritev(self.file_fd, &io_bufs, off as i64)?;
        if n < header_io.len() + buf_io.len() {
            return Err(Error::UnknownStr("vectored write did not write all data".to_owned()));
        }

        Ok(())
    }

    pub fn write_buf(&mut self, block_no: u32, poff: u64, buf: &[u8]) -> Result<u64, Error> {
        self.write_buf_at(self.curr_off, block_no, buf)?;

        let page_off = self.curr_off;
        //let page_off = self.curr_off;
        self.curr_off += buf.len() as u64 + NixFile::header_len() as u64;
        self.curr_off += poff;
        //self.curr_off += buf.len() as u64;

        Ok(page_off)
    }

    pub fn header_len() -> usize {
        return crate::PAGE_HEADER_LEN;
    }

    fn read_all_at(&self, off: u64, buf: &mut [u8]) -> Result<usize, Error> {
        let mut i = 0;
        while i < buf.len() {
            //TODO: Should n==0 be handled?
            let n = nix::sys::uio::pread(self.file_fd, &mut buf[i..], off as i64 + i as i64)?;
            if n == 0 {
                break;
            }
            i += n;
        }
        Ok(i)
    }

    pub fn read_buf_at(&self, off: u64, buf: &mut [u8]) -> Result<usize, Error> {
        log::debug!("file read at fd={} off={} {}", self.file_fd, off, buf.len());
        let n = self.read_all_at(off, buf)?;
        Ok(n)
    }

    pub fn sync(&self) -> Result<(), Error> {
        nix::unistd::fsync(self.file_fd)?;
        Ok(())
    }
}


struct PageHeader {
    magic: &'static [u8],
    block_no: u32,
}

impl PageHeader {
    fn new() -> PageHeader {
        PageHeader { 
            magic: crate::BUFFER_MAGIC,
            block_no: 0,
        }
    }

    pub fn encode(&mut self, buf: &mut [u8; crate::PAGE_HEADER_LEN]) {
        let _ = &buf[..4].copy_from_slice(self.magic);
        let _ = &buf[4..8].copy_from_slice(&self.block_no.to_le_bytes());
        //let _ = &buf[12..13].copy_from_slice(&self.flags.to_le_bytes());
        //let _ = &buf[13..17].copy_from_slice(&self.file_no.to_le_bytes());
    }

    /*
    pub fn decode(buf: &[u8]) -> Result<PageHeader, Error> {
        let magic = &buf[..4];
        if magic != BUFFER_MAGIC {
            return Err(Error::UnknownStr("Invalid buffer magic".to_string()));
        }

        let mut tmp_buf = [0u8; 8];
        tmp_buf.copy_from_slice(&buf[4..12]);
        let block_no = u64::from_be_bytes(tmp_buf);

        let mut tmp_buf = [0u8; 2];
        tmp_buf.copy_from_slice(&buf[12..14]);
        let size = u16::from_be_bytes(tmp_buf);

        tmp_buf.copy_from_slice(&buf[12..14]);
        let flags = buf[15];

        let mut tmp_buf = [0u8; 4];
        tmp_buf.copy_from_slice(&buf[15..19]);
        let file_no = u32::from_be_bytes(tmp_buf);

        Ok(PageHeader{
            magic: BUFFER_MAGIC,
            block_no,
            size,
            flags,
            file_no,
        })
    }
    */
}