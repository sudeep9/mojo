use crate::error::Error;

use crate::native_file::NativeFile;
use crate::kvfile::KVFile;
use crate::open_options::OpenOptions;

pub enum FileImpl {
    Reg(NativeFile),
    KV(KVFile)
}

pub struct VFSFile {
    pub bucket: String,
    id: usize,
    fimpl: FileImpl,
    opt: OpenOptions,
}


impl VFSFile {
    pub fn new(id: usize, name: &str, opt: OpenOptions, fimpl: FileImpl) -> Self {
        VFSFile{
            bucket: name.to_owned(),
            id,
            fimpl,
            opt,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn opt(&self) -> OpenOptions {
        self.opt.clone()
    }

    pub fn pread(&self, off: u64, buf: &mut [u8]) -> Result<usize, Error> {
        log::debug!("pread id={} o={}, blen={}", self.id, off, buf.len());

        let nread = match &self.fimpl {
            FileImpl::Reg(f) => {
                f.pread(buf, off as i64)?
            },
            FileImpl::KV(f) => {
                f.pread(buf, off as i64)?
            }
        };

        Ok(nread)
    }


    pub fn pwrite(&mut self, off: u64, buf: &[u8]) -> Result<(), Error> {
        log::debug!("pwrite id={} o={}, blen={}", self.id, off, buf.len());

        match &mut self.fimpl {
            FileImpl::Reg(f) => {
                f.pwrite(off as i64, buf)?;
            },
            FileImpl::KV(f) => {
                f.pwrite(off as i64, buf)?;
            }
        };

        Ok(())
    }

    pub fn close(self) -> Result<(), Error> {
        log::debug!("file close id={}", self.id);

        match self.fimpl {
            FileImpl::Reg(f) => {
                f.close()?;
            },
            FileImpl::KV(f) => {
                f.close()?;
            }
        };
        
        Ok(())
    }

    pub fn sync(&mut self, flags: i32) -> Result<(), Error> {
        log::debug!("sync id={} flags={}", self.id, flags);

        match &mut self.fimpl {
            FileImpl::Reg(f) => {
                f.sync()?;
            },
            FileImpl::KV(f) => {
                f.sync()?;
            }
        };

        Ok(())
    }

    pub fn filesize(&self) -> Result<u64, Error> {
        let sz = match &self.fimpl {
            FileImpl::Reg(f) => {
                f.filesize()?
            },
            FileImpl::KV(f) => {
                f.filesize()?
            }
        };

        Ok(sz)
    }

    pub fn truncate(&mut self, new_sz: u64) -> Result<(), Error> {
        log::debug!("truncate id={} {}", self.id, new_sz);

        match &mut self.fimpl {
            FileImpl::Reg(f) => {
                f.truncate(new_sz)?;
            },
            FileImpl::KV(f) => {
                f.truncate(new_sz)?;
            }
        };

        Ok(())
    }

    pub fn lock(&mut self, _flag: i32) -> Result<(), Error> {
        Ok(())
    }

    pub fn unlock(&mut self, _flag: i32) -> Result<(), Error> {
        Ok(())
    }

    pub fn check_reserved_lock(&self) -> Result<i32, Error> {
        Ok(0)
    }

    pub fn file_control(&mut self, _op: i32) -> Result<(), Error> {
        Ok(())
    }

    pub fn sector_size(&self) -> Result<i32, Error> {
        Ok(0)
    }

    pub fn device_char(&self) -> Result<(), Error> {
        Ok(())
    }
}
