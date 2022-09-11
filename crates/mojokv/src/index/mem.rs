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

    /*
    pub fn serialize<W: std::io::Write>(&self, w: &mut W) -> Result<(), Error> {
        w.write_all(self.header.magic.as_bytes())?;

        w.write_all(&self.header.format_ver.to_le_bytes())?;
        w.write_all(&self.header.min_ver.to_le_bytes())?;
        w.write_all(&self.header.active_ver.to_le_bytes())?;
        w.write_all(&(self.header.pps as u32).to_le_bytes())?;
        w.write_all(&self.header.max_key.to_le_bytes())?;

        let mut tmp_buf = Vec::new();
        self.kmap.serialize(&mut tmp_buf)?;

        w.write_all(&tmp_buf.len().to_le_bytes())?;
        let cbuf = zstd::bulk::compress(&tmp_buf, 4)?;

        w.write_all(&cbuf.len().to_le_bytes())?;
        w.write_all(&cbuf)?;

        Ok(())
    }

    pub fn deserialize_header<R: std::io::Read>(r: &mut R) -> Result<(usize, usize, IndexHeader), Error> {
        let mut magic_buf = [0; 10];
        r.read_exact(&mut magic_buf)?;
        if magic_buf != super::MOJO_INDEX_MAGIC.as_bytes() {
            return Err(Error::UnknownStr(format!("Invalid mojo index magic {:?}", magic_buf)));
        }

        let format_ver = utils::read_le_u32(r)?;
        if format_ver != 1 {
            return Err(Error::UnknownStr(format!("Invalid mojo index format version {:?}", format_ver)));
        }

        let min_ver = utils::read_le_u32(r)?;
        let active_ver = utils::read_le_u32(r)?;
        let pps = utils::read_le_u32(r)?;
        let max_key = utils::read_le_isize(r)?;

        let uncompressed_size = utils::read_le_usize(r)?;
        let compressed_size = utils::read_le_usize(r)?;

        let header = IndexHeader {
            magic: super::MOJO_INDEX_MAGIC,
            format_ver,
            min_ver,
            active_ver,
            max_key,
            pps: pps as usize,
        };

        Ok((compressed_size, uncompressed_size, header))
    }

    pub fn deserialize<R: std::io::Read>(r: &mut R) -> Result<MemIndex, Error> {
        let (comp_sz, uncomp_sz, header) = Self::deserialize_header(r)?;

        let mut buf = vec![0u8; comp_sz];

        r.read_exact(buf.as_mut_slice())?;
        let uncomp_buf = zstd::bulk::decompress(&buf, uncomp_sz)?;

        let kmap = KeyMap::deserialize(&mut std::io::Cursor::new(uncomp_buf))?;

        Ok(MemIndex{kmap, header})
    }*/

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
        //utils::write_file(filepath, &cbuf)?;

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
