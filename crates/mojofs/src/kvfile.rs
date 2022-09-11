
use mojokv::Bucket;
use crate::Error;

pub struct KVFile {
    pub bucket: Bucket,
    opt: KVFileOpt,
}

#[derive(Clone)]
pub struct KVFileOpt {
    pub page_sz: u32,
    pub pps: u32,
    pub ver: u32,
}


impl KVFile {
    pub fn open(bucket: Bucket, opt: KVFileOpt) -> Result<Self, Error> {
        Ok(KVFile{
            bucket,
            opt,
        })
    }

    pub fn pread(&self, buf: &mut [u8], off: i64) -> Result<usize, Error> {
        log::debug!("kv pread o={}, blen={}", off, buf.len());

        if buf.len() > self.opt.page_sz as usize {
            return Err(Error::new(crate::MOJOFS_ERR_LARGE_PAGE, 
                format!("buf larger ({}) than page size", buf.len())));
        }

        let page_off = off % self.opt.page_sz as i64;
        let key = off / self.opt.page_sz as i64;

        let n = match self.bucket.get(key as u32, page_off as u64, buf) {
            Ok(n) => n,
            Err(err) => {
                if let mojokv::Error::KeyNotFoundErr(_) = err {
                    0
                }else{
                    return Err(err.into());
                }
            }
        };

        if n<buf.len() {
            log::debug!("after kv pread o={}, blen={} n={}", off, buf.len(), n);
            let _ = &mut buf[n..].fill(0);
        }

        Ok(n)
    }

    fn pwrite_page(&mut self, key: u32, page_off: u32, buf: &[u8]) -> Result<(), Error> {
        log::debug!("kv pwrite page key={}, po={} blen={}", key, page_off, buf.len());

        self.bucket.put(key, page_off as u64, buf)?;

        Ok(())
    }

    pub fn pwrite(&mut self, off: i64, buf: &[u8]) -> Result<(), Error> {
        log::debug!("kv pwrite o={}, blen={}", off, buf.len());

        let mut po = off % self.opt.page_sz as i64;
        let mut key = off / self.opt.page_sz as i64;
        let mut s = 0usize;
        let buflen = buf.len();

        while s < buflen {
            let e = (buflen-s).min(self.opt.page_sz as usize - po as usize);
            self.pwrite_page(key as u32, po as u32, &buf[s..s+e])?;
            s += e;
            po = 0;
            key += self.opt.page_sz as i64;
        }

        Ok(())
    }

    pub fn close(self) -> Result<(), Error> {
        self.bucket.close()?;
        Ok(())
    }

    pub fn sync(&mut self) -> Result<(), Error> {
        self.bucket.sync()?;
        Ok(())
    }

    pub fn filesize(&self) -> Result<u64, Error> {
        Ok(self.bucket.logical_size())
    }

    pub fn truncate(&mut self, new_sz: u64) -> Result<(), Error> {
        log::debug!("kv truncate {}", new_sz);
        self.bucket.truncate(new_sz as usize)?;
        Ok(())
    }
}
