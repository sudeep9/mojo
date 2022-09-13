use std::io::Write;

use serde::Deserialize;
use serde::Serialize;

use crate::value::Value;
use crate::keymap::KeyMap;
use crate::Error;
use crate::utils;
use super::IndexHeader;


//TODO: Reserve some space for additional data
#[derive(Serialize, Deserialize)]
pub struct MemIndex {
    header: IndexHeader,
    pub kmap: KeyMap
}

impl MemIndex {
    pub fn new(pps: usize) -> Self {
        MemIndex {
            header: IndexHeader::new(pps),
            kmap: KeyMap::new(pps),
        }
    }

    pub fn header(&self) -> &IndexHeader {
        &self.header
    }

    fn key_map(&self) -> &KeyMap {
        &self.kmap
    }

    pub fn set_active_ver(&mut self, ver: u32) {
        self.header.active_ver = ver;
    }

    pub fn active_ver(&self) -> u32 {
        self.header.active_ver
    }

    pub fn max_key(&self) -> isize {
        self.header.max_key
    }

    pub fn put(&mut self, key: u32, off: u32) -> Result<(), Error> {
        let mut val = Value::new();
        val.put_off(off);
        val.put_ver(self.header.active_ver);

        log::debug!("index put val:{:?}", val);
        self.header.max_key = self.header.max_key.max(key as isize);
        self.kmap.put(key, val);
        Ok(())
    }

    pub fn get(&self, key: u32) -> Result<Option<&Value>, Error> {
        Ok(self.kmap.get(key))
    }

    pub fn truncate(&mut self, key: u32) -> Result<(), Error> {
        self.kmap.truncate(key);
        self.header.max_key = key as isize -1;
        Ok(())
    }

    pub fn iter<'a>(&'a self, from_key: u32, to_key: u32) -> Box<dyn Iterator<Item=(u32, &'a Value)> + 'a > {
        let itr = MemIndexIterator {
            key: from_key,
            to_key,
            index: self
        };

        Box::new(itr)
    }

    pub fn serialize_to_path(&self, filepath: &std::path::Path) -> Result<(), Error> {
        let tmp_buf = rmp_serde::to_vec(&self)?;
        let cbuf = zstd::bulk::compress(&tmp_buf, 3)?;

        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(filepath)?;

        let cap_buf = tmp_buf.len().to_le_bytes();
        f.write_all(&cap_buf)?;
        f.write_all(&cbuf)?;
        f.sync_data()?;

        Ok(())    
    }

    pub fn deserialize_from_path(filepath: &std::path::Path) -> Result<(usize, usize, MemIndex), Error> {
        let mut b = Vec::new();
        utils::load_file(filepath, &mut b)?;

        let cap = usize::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);

        let buf = zstd::bulk::decompress(&b[8..], cap)?;

        let index = rmp_serde::from_slice(&buf)?;
        Ok((cap, b.len(), index))
    }

    /*
    pub fn deserialize_header_from_path(path: &std::path::Path) -> Result<IndexHeader, Error> {
        let f = std::fs::OpenOptions::new().read(true).open(path)?;

        let h: IndexHeader = rmp_serde::from_read(f)?;
        Ok(h)
    }
    */

}

pub struct MemIndexIterator<'a> {
    index: &'a MemIndex,
    key: u32,
    to_key: u32
}

impl<'a> MemIndexIterator<'a> {
    pub fn new(from_key: u32, to_key: u32, index: &'a MemIndex) -> Self {
        MemIndexIterator { 
            index,
            key: from_key,
            to_key,
        }
    }
}

impl<'a> Iterator for MemIndexIterator<'a> {
    type Item =  (u32, &'a Value);

    fn next(&mut self) -> Option<Self::Item> {
        if self.to_key > 0 && self.key >= self.to_key {
            return None;
        }

        let kmap_index = self.key/self.index.header.pps as u32;

        if kmap_index as usize >= self.index.key_map().slot_map.len() {
            return None;
        }

        let slot_map = &self.index.key_map().slot_map[kmap_index as usize];

        let ret = match slot_map {
            Some(map) => {
                let slot_index = (self.key as usize) %self.index.header.pps;
                if slot_index >= map.len() {
                    return None;
                }

                let val = &map[slot_index];
                if val.is_allocated() {
                    Some((self.key, val))
                }else{
                    None
                }
            },
            None => None
        };


        self.key += 1;

        ret
    }
}

//#[derive(Serialize, Deserialize)]
