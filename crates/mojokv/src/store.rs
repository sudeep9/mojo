use std::path::{Path, PathBuf};
use crate::{Error, utils};
use crate::state::KVState;
use crate::bucket::Bucket;
use crate::bmap::BucketMap;
use crate::index::mem::MemIndex;
use fslock::LockFile;

pub struct KVStore {
    root_path: PathBuf,
    state: KVState,
    is_write: bool,
    bmap: BucketMap,
}

impl KVStore {
    pub fn exists(&self, name: &str) -> bool {
        self.bmap.exists(name)
    }

    pub fn open(&mut self, name: &str, mode: BucketOpenMode) -> Result<Bucket, Error> {
        log::debug!("store bucket open name={} mode writable={} store is write: {}", name, mode.is_write(), self.is_write);

        if !self.is_write && mode.is_write() {
            return Err(Error::StoreNotWritableErr);
        }

        let mut b = match self.bmap.get(name) {
            Some(v) => {
                log::debug!("Bucket name={} exists at ver={}", name, v);
                Bucket::load(&self.root_path, name, self.state.clone(), self.bmap.clone(), v)?
            },
            None => {
                log::debug!("Bucket name={} does not exists", name);
                if !self.is_write {
                    return Err(Error::StoreNotWritableErr);
                }
                Bucket::new(&self.root_path, name, self.state.clone(), self.bmap.clone())?
            }
        };

        if self.is_write && mode.is_write() {
            log::debug!("setting bucket={} to writable", name);
            b.set_writable();
            b.sync()?;
        }

        if mode.is_write() {
            self.sync_bmap()?;
        }

        Ok(b)
    }

    pub fn delete(&mut self, name: &str) -> Result<(), Error> {
        self.bmap.delete(&self.root_path, name, self.state.active_ver())?;
        self.sync_bmap()
    }

    pub fn commit(&mut self) -> Result<u32, Error> {
        log::debug!("committing store ver={}", self.state.active_ver());

        let _ = self.state.commit_lock.write();

        log::debug!("about to acquire commit file lock ver={}", self.state.active_ver());
        let mut commit_lock_file = Self::create_lock_file(&self.root_path)?;

        if !commit_lock_file.try_lock_with_pid()? {
            return Err(Error::CommitLockedErr);
        }

        let new_ver = self.state.advance_ver();
        self.sync_state()?;
        self.sync_bmap()?;

        log::debug!("committing store done");
        Ok(new_ver)
    }

    pub fn active_ver(&self) -> u32 {
        self.state.active_ver()
    }

    pub fn load_state(rootpath: &Path) -> Result<KVState, Error> {
        let state_path = rootpath.join("mojo.state");
        log::debug!("loading state from {:?}", state_path);
        let state = KVState::deserialize_from_path(&state_path)?;
        Ok(state)
    }

    pub fn readonly(root_path: &Path, ver: u32) -> Result<Self, Error> {
        log::debug!("opening store in readonly mode at ver={}", ver);
        let state = Self::load_state(root_path)?;
        Self::load_store(root_path, state, ver)
    }

    pub fn writable(rootpath: &Path, create: bool, page_sz: Option<u32>, pps: Option<u32>) -> Result<KVStore, Error> {
        let init_path = rootpath.join("mojo.init");

        if create && (page_sz.is_none() || pps.is_none()) {
            log::debug!("Missing mandatory params page_sz:{:?} pps:{:?}", page_sz, pps);
            return Err(Error::MissingArgsErr);
        }

        let mut store = if !init_path.exists() {
            if !create {
                return Err(Error::StoreNotFoundErr);
            }

            log::debug!("Store does not exists. Initing now");
            let mut store = KVStore::new(rootpath, page_sz.unwrap(), pps.unwrap())?;
            store.init()?;
            log::debug!("Store init successfull");
            store
        }else{
            let state = Self::load_state(rootpath)?;
            let aver = state.active_ver();
            Self::load_store(rootpath, state, aver)?
        };

        store.is_write = true;
               
        Ok(store)
    }

    fn load_store(root_path: &Path, state: KVState, ver: u32) -> Result<KVStore, Error> {
        log::debug!("loading store at ver={}", ver);
        let bmap = BucketMap::load(root_path, ver)?;

        let store = KVStore {
            root_path: root_path.to_owned(),
            state,
            is_write: false,
            bmap,
        };

        Ok(store)
    }

    pub fn get_index(&self, name: &str) -> Result<Option<(usize, usize, MemIndex)>, Error> {
        match self.bmap.get(name) {
            Some(v) => {
                let ret = Bucket::load_index(&self.root_path, name, v)?;
                Ok(Some(ret))
            },
            None => {
                log::debug!("Bucket name={} does not exists", name);
                return Ok(None)
            }
        }
    }

    fn new(root_path: &Path, page_sz: u32, pps: u32) -> Result<Self, Error> {
        let state = KVState::new(page_sz, pps);

        let store = KVStore {
            root_path: root_path.to_owned(),
            state,
            is_write: false,
            bmap: BucketMap::default(),
        };

        Ok(store)
    }

    fn init(&mut self) -> Result<(), Error> {
        std::fs::create_dir_all(&self.root_path)?;
        self.sync()?;
        let touch_file = self.root_path.join("mojo.init");
        utils::touch_file(&touch_file)?;
        Ok(())
    }

    fn sync(&mut self) -> Result<(), Error> {
        self.sync_state()?;
        self.sync_bmap()?;
        Ok(())
    }

    fn sync_bmap(&mut self) -> Result<(), Error> {
        log::debug!("syncing bmap at ver={}", self.state.active_ver());

        let bmap_path = self.root_path.join(&format!("mojo.bmap.{}", self.state.active_ver()));

        self.bmap.serialize_to_path(&bmap_path)?;

        Ok(())
    }


    fn sync_state(&mut self) -> Result<(), Error> {
        let file_path = self.root_path.join("mojo.state");

        log::debug!("syncing state ver={} {:?}", self.state.active_ver(), file_path);
        
        self.state.serialize_to_path(&file_path)?;

        log::debug!("syncing state done");
        Ok(())
    }

    fn create_lock_file(root_path: &Path) -> Result<LockFile, Error> {
        let lock_path = root_path.join("mojo.lock");
        log::debug!("creating lock file: {:?}", lock_path);
        Ok(LockFile::open(&lock_path)?)
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum BucketOpenMode {
    Read,
    Write,
}

impl BucketOpenMode {
    pub fn is_write(&self) -> bool {
        *self == Self::Write
    }
}