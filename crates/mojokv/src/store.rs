use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::{Error, utils};
use crate::state::State;
use crate::bucket::Bucket;
use crate::bmap::BucketMap;
use crate::index::mem::MemIndex;
use parking_lot::RwLock;
use fslock::LockFile;

struct StoreInner {
    root_path: PathBuf,
    state: State,
    is_write: bool,
    bmap: BucketMap,
}
pub struct Store {
    inner: Arc<RwLock<StoreInner>>,
}

impl Store {
    pub fn exists(&self, name: &str) -> bool {
        let inner = self.inner.read();
        inner.bmap.exists(name)
    }

    pub fn open(&self, name: &str, mode: BucketOpenMode) -> Result<Bucket, Error> {
        let mut inner = self.inner.write();

        log::debug!("store bucket open name={} mode writable={} store is write: {}", name, mode.is_write(), inner.is_write);

        if !inner.is_write && mode.is_write() {
            return Err(Error::StoreNotWritableErr);
        }

        let mut b = match inner.bmap.get(name) {
            Some(v) => {
                log::debug!("Bucket name={} exists at ver={}", name, v);
                Bucket::load(&inner.root_path, name, inner.state.clone(), inner.bmap.clone(), v)?
            },
            None => {
                log::debug!("Bucket name={} does not exists", name);
                if !inner.is_write {
                    return Err(Error::StoreNotWritableErr);
                }
                Bucket::new(&inner.root_path, name, inner.state.clone(), inner.bmap.clone())?
            }
        };

        if inner.is_write && mode.is_write() {
            log::debug!("setting bucket={} to writable", name);
            b.set_writable();
            b.sync()?;
        }

        if mode.is_write() {
            inner.sync_bmap()?;
        }

        Ok(b)
    }

    pub fn delete(&self, name: &str) -> Result<(), Error> {
        let mut inner = self.inner.write();
        let aver = inner.state.active_ver();

        inner.bmap.delete(&inner.root_path, name, aver)?;
        inner.sync_bmap()
    }

    pub fn commit(&self) -> Result<u32, Error> {
        let mut inner = self.inner.write();

        log::debug!("committing store ver={}", inner.state.active_ver());

        let _ = inner.state.commit_lock.write();

        log::debug!("about to acquire commit file lock ver={}", inner.state.active_ver());
        let mut commit_lock_file = Self::create_lock_file(&inner.root_path)?;

        if !commit_lock_file.try_lock_with_pid()? {
            return Err(Error::CommitLockedErr);
        }

        let new_ver = inner.state.advance_ver();
        inner.sync_state()?;
        inner.sync_bmap()?;

        log::debug!("committing store done");
        Ok(new_ver)
    }

    pub fn active_ver(&self) -> u32 {
        let inner = self.inner.read();
        inner.state.active_ver()
    }

    pub fn load_state(rootpath: &Path) -> Result<State, Error> {
        let state_path = rootpath.join("mojo.state");
        log::debug!("loading state from {:?}", state_path);
        let state = State::deserialize_from_path(&state_path)?;
        Ok(state)
    }

    pub fn readonly(root_path: &Path, ver: u32) -> Result<Self, Error> {
        log::debug!("opening store in readonly mode at ver={}", ver);
        let state = Self::load_state(root_path)?;
        Self::load_store(root_path, state, ver)
    }

    pub fn writable(rootpath: &Path, create: bool, page_sz: Option<u32>, pps: Option<u32>) -> Result<Store, Error> {
        let init_path = rootpath.join("mojo.init");

        if create && (page_sz.is_none() || pps.is_none()) {
            log::debug!("Missing mandatory params page_sz:{:?} pps:{:?}", page_sz, pps);
            return Err(Error::MissingArgsErr);
        }

        let store = if !init_path.exists() {
            if !create {
                return Err(Error::StoreNotFoundErr);
            }

            log::debug!("Store does not exists. Initing now");
            let mut store = Store::new(rootpath, page_sz.unwrap(), pps.unwrap())?;
            store.init()?;
            log::debug!("Store init successfull");
            store
        }else{
            let state = Self::load_state(rootpath)?;
            let aver = state.active_ver();
            Self::load_store(rootpath, state, aver)?
        };

        {
            let mut inner = store.inner.write();
            inner.is_write = true;
        }
               
        Ok(store)
    }

    fn load_store(root_path: &Path, state: State, ver: u32) -> Result<Store, Error> {
        log::debug!("loading store at ver={}", ver);
        let bmap = BucketMap::load(root_path, ver)?;

        let inner = StoreInner {
            root_path: root_path.to_owned(),
            state,
            is_write: false,
            bmap,
        };

        let store = Store {inner: Arc::new(RwLock::new(inner))};

        Ok(store)
    }

    pub fn get_index(&self, name: &str) -> Result<Option<(usize, usize, MemIndex)>, Error> {
        let inner = self.inner.read();

        match inner.bmap.get(name) {
            Some(v) => {
                let ret = Bucket::load_index(&inner.root_path, name, v)?;
                Ok(Some(ret))
            },
            None => {
                log::debug!("Bucket name={} does not exists", name);
                return Ok(None)
            }
        }
    }

    fn new(root_path: &Path, page_sz: u32, pps: u32) -> Result<Self, Error> {
        let state = State::new(page_sz, pps);

        let inner = StoreInner {
            root_path: root_path.to_owned(),
            state,
            is_write: false,
            bmap: BucketMap::default(),
        };

        let store = Store {
            inner: Arc::new(RwLock::new(inner)),
        };

        Ok(store)
    }

    fn init(&mut self) -> Result<(), Error> {
        let mut inner = self.inner.write();

        std::fs::create_dir_all(&inner.root_path)?;
        inner.sync()?;
        let touch_file = inner.root_path.join("mojo.init");
        utils::touch_file(&touch_file)?;
        Ok(())
    }


    fn create_lock_file(root_path: &Path) -> Result<LockFile, Error> {
        let lock_path = root_path.join("mojo.lock");
        log::debug!("creating lock file: {:?}", lock_path);
        Ok(LockFile::open(&lock_path)?)
    }
}

impl StoreInner {
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