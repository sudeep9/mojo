
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::Arc;
use crate::bucket::Bucket;
use crate::Error;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct BucketMap {
    map: Arc<RwLock<HashMap<String, u32>>>,
}

impl BucketMap {
    pub fn add(&mut self, name: &str, ver: u32) {
        log::debug!("add name={} ver={} {:?}", name, ver, self.map);
        let mut map = self.map.write();
        //self.buckets.insert(name.to_owned(), b);
        map.insert(name.to_owned(), ver);
    }

    pub fn exists(&self, name: &str) -> bool {
        let map = self.map.read();
        map.contains_key(name)
    } 

    pub fn get(&self, name: &str) -> Option<u32>{
        log::debug!("get name={}", name);
        let map = self.map.read();
        map.get(name).map(|v| *v)
    }

    pub fn delete(&mut self, root_path: &Path, name: &str, ver: u32) -> Result<(), Error> {
        log::debug!("delete name={} {:?}", name, self.map);
        let mut map = self.map.write();

        map.remove(name);

        Bucket::delete_ver(root_path, name, ver)?;

        Ok(())
    }

    pub fn map(&self) -> Result<HashMap<String, u32>, Error> {
        let map = self.map.read();

        Ok(map.clone())
    }

    pub fn serialize_to_path(&self, path: &Path) -> Result<(), Error> {
        let buf = serde_json::to_vec(&self)?;
        log::debug!("serializing bmap={:?}", std::str::from_utf8(&buf));
        crate::utils::write_file(path, &buf)?;
        Ok(())
    }

    pub fn deserialize_from_path(path: &Path) -> Result<Self, Error> {
        let mut buf = Vec::new();
        crate::utils::load_file(path, &mut buf)?;

        let map = serde_json::from_slice(&buf)?;
        Ok(map)
    }

    fn bmap_path(root_path: &Path, ver: u32) -> PathBuf {
        root_path.join(&format!("mojo.bmap.{}", ver))
    }

    pub fn load(root_path: &Path, ver: u32) -> Result<Self, Error> {
        let bmap_path = Self::bmap_path(root_path, ver);
        log::debug!("loading bmap from path={:?}", bmap_path);
        let bmap = Self::deserialize_from_path(&bmap_path)?;

        Ok(bmap)
    }
}