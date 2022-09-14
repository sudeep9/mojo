
use crate::Error;
use mojoio::nix::NixFile;
use crate::utils;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StateInner {
    pub format_ver: u32,
    pub min_ver: u32,
    pub max_ver: u32,
    pub active_ver: u32,
    pub pps: u32,
    pub page_sz: u32,
    pub file_header_len: u32,
    pub file_page_sz: u32,

    //TODO: add timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    inner: Arc<RwLock<StateInner>>,

    #[serde(skip)]
    pub commit_lock: Arc<RwLock<bool>>,
}

impl State {

    pub fn new(page_sz: u32, pps: u32) -> Self {

        let inner = StateInner {
            format_ver: 1,
            min_ver: 1,
            max_ver: 1,
            active_ver: 1,
            pps,
            page_sz,
            file_header_len: NixFile::header_len() as u32,
            file_page_sz: page_sz + NixFile::header_len() as u32,
        };

        let state = State {
            inner: Arc::new(RwLock::new(inner)),
            commit_lock: Arc::new(RwLock::new(false)),
        };

        state
    }

    pub fn format_ver(&self) -> u32 {
        let inner = self.inner.read();
        inner.format_ver
    }

    pub fn active_ver(&self) -> u32 {
        let inner = self.inner.read();
        inner.active_ver
    }

    pub fn page_size(&self) -> u32 {
        let inner = self.inner.read();
        inner.page_sz
    }

    pub fn file_page_sz(&self) -> u32 {
        let inner = self.inner.read();
        inner.file_page_sz
    }

    pub fn pps(&self) -> u32 {
        let inner = self.inner.read();
        inner.pps
    }

    pub fn min_ver(&self) -> u32 {
        let inner = self.inner.read();
        inner.min_ver
    }

    pub fn max_ver(&self) -> u32 {
        let inner = self.inner.read();
        inner.max_ver
    }

    pub fn advance_ver(&self) -> u32 {
        let mut inner = self.inner.write();
        inner.active_ver += 1;
        inner.max_ver = inner.max_ver.max(inner.active_ver);

        inner.active_ver
    }

    pub fn serialize_to_path(&self, filepath: &std::path::Path) -> Result<(), Error> {
        let buf = rmp_serde::to_vec_named(&self)?;

        utils::write_file(filepath, &buf)?;

        Ok(())    
    }

    pub fn deserialize_from_path(filepath: &std::path::Path) -> Result<State, Error> {
        let mut buf = Vec::new();
        utils::load_file(filepath, &mut buf)?;

        let state = rmp_serde::from_slice(&buf)?;
        Ok(state)
    }
}