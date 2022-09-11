
use std::path::{Path, PathBuf};
use nix::fcntl::{self, OFlag};
use nix::sys::stat::Mode;

use crate::open_options::*;
use crate::Error;

pub struct NativeFile {
    path: PathBuf,
    fd: i32,
    opt: OpenOptions,
}

impl NativeFile {
    pub fn open(path: &Path, opt: &OpenOptions) -> Result<Self, Error> {
        let open_flags = match opt.access {
            OpenAccess::Read => {
                OFlag::O_RDONLY                
            },
            OpenAccess::Write => {
                OFlag::O_RDWR    
            },
            OpenAccess::Create => {
                OFlag::O_CREAT|OFlag::O_RDWR
            },
            OpenAccess::CreateNew => {
                OFlag::O_CREAT|OFlag::O_RDWR|OFlag::O_EXCL
            }
        };

        let file_perm = Mode::all();
        let fd = fcntl::open(path, open_flags, file_perm)?;

        Ok(NativeFile{
            path: path.to_owned(),
            opt: opt.clone(),
            fd,
        })
    }

    pub fn pread(&self, buf: &mut [u8], off: i64) -> Result<usize, Error> {
        log::debug!("native pread fd={} o={}, blen={}", self.fd, off, buf.len());
        let mut i = 0;
        while i<buf.len() {
            let n = nix::sys::uio::pread(self.fd, &mut buf[i..], off + i as i64)?;
            if n == 0 {
                break;
            }
            i += n;
        }

        if i<buf.len() {
            let _ = &mut buf[i..].fill(0);
        }

        Ok(i)
    }

    pub fn pwrite(&mut self, off: i64, buf: &[u8]) -> Result<(), Error> {
        log::debug!("native pwrite fd={} o={}, blen={}", self.fd, off, buf.len());

        let mut i=0;
        while i<buf.len() {
            let n = nix::sys::uio::pwrite(self.fd, &buf[i..], off + i as i64)?;
            i += n;
        }
        Ok(())
    }

    pub fn close(self) -> Result<(), Error> {
        nix::unistd::close(self.fd)?;
        if self.opt.delete_on_close {
            nix::unistd::unlink(&self.path)?;
        }

        Ok(())
    }

    pub fn sync(&mut self) -> Result<(), Error> {
        nix::unistd::fsync(self.fd)?;
        Ok(())
    }

    pub fn filesize(&self) -> Result<u64, Error> {
        let st = nix::sys::stat::fstat(self.fd)?;
        Ok(st.st_size as u64)
    }

    pub fn truncate(&mut self, new_sz: u64) -> Result<(), Error> {
        log::debug!("truncate id={} {}", self.fd, new_sz);
        nix::unistd::ftruncate(self.fd, new_sz as i64)?;
        Ok(())
    }
}