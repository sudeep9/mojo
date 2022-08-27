
use std::path::{PathBuf, Path};
use crate::{error, Error};
use crate::open_options::*;
use std::collections::HashMap;
use crate::kvfile::{KVFile, KVFileOpt};
use crate::vfsfile::FileImpl;
use mojokv::{KVStore, BucketOpenMode};

use crate::vfsfile::VFSFile;

#[derive(Debug)]
pub enum AccessCheck {
    Exists,
    ReadWrite,
}

#[derive(Default)]
pub struct VFS {
    store: Option<KVStore>,
    file_counter: usize,
    fopt: FSOptions,
}

impl VFS {
    pub fn name(&self) -> String {
        return "mojo".to_owned()
    }

    pub fn fs_options(&self) -> FSOptions {
        self.fopt.clone()
    }

    pub fn active_ver(&self) -> u32 {
        self.store.as_ref().unwrap().active_ver()
    }

    pub fn init(&mut self, root_path: &str, params: &HashMap<String, String>, opt: OpenOptions) -> Result<(), Error> {
        log::debug!("init: root_path={} params={:?}", root_path, params);

        self.fopt = FSOptions::parse(params)?;
        let root_path = Path::new(root_path);
        if opt.access == OpenAccess::Read {
            self.store = Some(KVStore::readonly(root_path, self.fopt.ver)?);
        }else{
            self.store = Some(KVStore::writable(root_path, true, Some(self.fopt.pagesz), Some(self.fopt.pps))?);
        }

        Ok(())
    }

    pub fn open(&mut self, filepath: &str, opt: OpenOptions, _out_opt: &mut OpenOptions) -> Result<Box<VFSFile>, Error> {
        log::debug!("open: file={} opt={:?}", filepath, opt);

        self.file_counter += 1;
        let id = self.file_counter;
        let file_path = if filepath.len() == 0{
            std::path::PathBuf::from(format!("mojo.tmp.{}", id))
        }else{
            std::path::PathBuf::from(filepath)
        };

        let store = self.store.as_mut().unwrap();

        let bucket_name = Self::bucket_name(&file_path);
        let bmode = if let OpenAccess::Read = opt.access {
            BucketOpenMode::Read
        }else{
            BucketOpenMode::Write
        };

        let b = store.open_bucket(bucket_name, bmode)?;
        let kvfileopt = self.fopt.to_kvfile_opt();

        let f = KVFile::open(b, kvfileopt)?;
        let fimpl = FileImpl::KV(f);

        log::debug!("open: file={} id={} done", filepath, id);
        Ok(Box::new(VFSFile::new(id, bucket_name, opt, fimpl)))
    }

    pub fn fullpath(&mut self, filepath: &str) -> Result<PathBuf, Error> {
        log::debug!("fullpath filepath={}", filepath);

        let filepath_rs = std::path::Path::new(filepath);
        if filepath_rs.is_absolute() {
            Ok(filepath_rs.to_owned())
        }else{
            let mut cwd = std::env::current_dir()?;
            cwd.push(filepath_rs);
            Ok(cwd)
        }
    }

    fn bucket_name(path: &Path) -> &str {
        path.file_name().unwrap().to_str().unwrap()
    }

    //TODO: add sync dir
    pub fn delete(&mut self, path: &std::path::Path) -> Result<(), Error> {
        log::debug!("delete path={:?}", path);

        let name = Self::bucket_name(path);
        let store = self.store.as_mut().unwrap();
        store.delete(name)?;

        Ok(())
    }

    pub fn access(&self, path: &std::path::Path, req: AccessCheck) -> Result<bool, Error> {
        log::debug!("access path={:?} req={:?}", path, req);

        let name = Self::bucket_name(path);
        let store = self.store.as_ref().unwrap();
        let status = store.exists(name);

        log::debug!("access path={:?} status={}", path, status);
        Ok(status)
    }

    pub fn close(&mut self, f: VFSFile) -> Result<(), Error> {
        log::debug!("close id={}", f.id());

        let bucket_name = f.bucket.clone();
        let opt = f.opt();

        f.close()?;

        let store = self.store.as_mut().unwrap();
        if opt.delete_on_close {
            store.delete(bucket_name.as_str())?;
        }

        Ok(())
    }

    pub fn commit(&mut self) -> Result<(), Error> {
        let store = self.store.as_mut().unwrap();
        store.commit()?;
        Ok(())
    }

}


#[derive(Default, Clone)]
pub struct FSOptions {
    pub ver: u32,
    pub pagesz: u32,
    pub pps: u32,
}

impl FSOptions {
    fn parse(map: &HashMap<String, String>) -> Result<FSOptions, Error> {
        let mut opt = FSOptions {ver: 0, pagesz: 4096, pps: 0};

        opt.ver = match map.get("ver") {
            Some(s) => s.parse()?,
            None => 1
        };

        let s = map.get("pagesz").ok_or(Error::new(error::MOJOFS_ERR_ARG_PAGESZ_MISSING, 
                "fs arg page size missing".to_owned()))?;
        opt.pagesz = s.parse()?;

        opt.pps = match map.get("pps") {
            Some(s) => s.parse()?,
            None => 65536
        };

        Ok(opt)
    }

    fn to_kvfile_opt(&self) -> KVFileOpt {
        KVFileOpt {
            ver: self.ver,
            page_sz: self.pagesz,
            pps: self.pps,
        }
    }
}