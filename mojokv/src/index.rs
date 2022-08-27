use crate::value::Value;
use crate::keymap::KeyMap;
use crate::Error;
use crate::utils;

pub const MOJO_INDEX_MAGIC: &'static str = "mojo_index";

//TODO: Reserve some space for additional data
pub struct Index {
    pub format_ver: u32,
    pub min_ver: u32,
    pub active_ver: u32,
    pub max_key: isize,
    pub pps: usize,
    pub kmap: KeyMap
}

impl Index {
    pub fn new(pps: usize) -> Self {
        Index {
            format_ver: 1,
            min_ver: 1,
            active_ver: 1,
            kmap: KeyMap::new(pps),
            pps,
            max_key: -1,
        }
    }

    pub fn key_map(&self) -> &KeyMap {
        &self.kmap
    }

    pub fn set_min_ver(&mut self, min_ver: u32) {
        self.min_ver = min_ver;
    }

    pub fn get_min_ver(&self) -> u32 {
        self.min_ver
    }

    pub fn set_active_ver(&mut self, ver: u32) {
        self.active_ver = ver;
    }

    pub fn active_ver(&self) -> u32 {
        self.active_ver
    }

    pub fn max_key(&self) -> isize {
        self.max_key
    }

    pub fn put(&mut self, key: u32, off: u32) {
        let mut val = Value::new();
        val.put_off(off);
        val.put_ver(self.active_ver);

        self.max_key = self.max_key.max(key as isize);
        self.kmap.put(key, val)
    }

    pub fn get(&self, key: u32) -> Option<&Value> {
        self.kmap.get(key)
    }

    pub fn truncate(&mut self, key: u32) {
        self.kmap.truncate(key);
        self.max_key = key as isize -1;
    }

    pub fn iter<'a>(&'a self, from_key: u32, to_key: u32) -> IndexIterator<'a> {
        IndexIterator {
            key: from_key,
            to_key,
            index: self
        }
    }

    pub fn serialize<W: std::io::Write>(&self, w: &mut W) -> Result<(), Error> {
        w.write_all(MOJO_INDEX_MAGIC.as_bytes())?;

        w.write_all(&self.format_ver.to_le_bytes())?;
        w.write_all(&self.min_ver.to_le_bytes())?;
        w.write_all(&self.active_ver.to_le_bytes())?;
        w.write_all(&(self.pps as u32).to_le_bytes())?;
        w.write_all(&self.max_key.to_le_bytes())?;

        let mut tmp_buf = Vec::new();
        self.kmap.serialize(&mut tmp_buf)?;

        w.write_all(&tmp_buf.len().to_le_bytes())?;
        let cbuf = zstd::bulk::compress(&tmp_buf, 4)?;

        w.write_all(&cbuf.len().to_le_bytes())?;
        w.write_all(&cbuf)?;

        Ok(())
    }

    pub fn deserialize_header<R: std::io::Read>(r: &mut R) -> Result<(usize, usize, Index), Error> {
        let mut magic_buf = [0; 10];
        r.read_exact(&mut magic_buf)?;
        if magic_buf != MOJO_INDEX_MAGIC.as_bytes() {
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

        let index = Index {
            format_ver,
            min_ver,
            active_ver,
            max_key,
            pps: pps as usize,
            kmap: KeyMap::new(pps as usize),
        };

        Ok((compressed_size, uncompressed_size, index))
    }

    pub fn deserialize<R: std::io::Read>(r: &mut R) -> Result<Index, Error> {
        let (comp_sz, uncomp_sz, mut index) = Self::deserialize_header(r)?;

        let mut buf = vec![0u8; comp_sz];

        r.read_exact(buf.as_mut_slice())?;
        let uncomp_buf = zstd::bulk::decompress(&buf, uncomp_sz)?;

        index.kmap = KeyMap::deserialize(&mut std::io::Cursor::new(uncomp_buf))?;

        Ok(index)
    }

    pub fn serialize_to_path(&self, filepath: &std::path::Path) -> Result<(), Error> {
        let mut tmp_buf = Vec::new();
        self.serialize(&mut tmp_buf)?;

        utils::write_file(filepath, &tmp_buf)?;

        Ok(())    
    }

    pub fn deserialize_from_path(filepath: &std::path::Path) -> Result<Index, Error> {
        let mut buf = Vec::new();
        utils::load_file(filepath, &mut buf)?;

        let mut r = std::io::Cursor::new(buf);
       
        let index = Index::deserialize(&mut r)?;
        Ok(index)
    }

    pub fn deserialize_header_from_path(path: &std::path::Path) -> Result<(usize, usize, Index), Error> {
        let mut f = std::fs::OpenOptions::new().read(true).open(path)?;
        let (comp_sz, uncomp_sz, index) = Index::deserialize_header(&mut f)?;
        Ok((comp_sz, uncomp_sz, index))
    }

}

pub struct IndexIterator<'a> {
    index: &'a Index,
    key: u32,
    to_key: u32
}

impl<'a> IndexIterator<'a> {
    pub fn new(from_key: u32, to_key: u32, index: &'a Index) -> Self {
        IndexIterator { 
            index,
            key: from_key,
            to_key,
        }
    }
}

impl<'a> Iterator for IndexIterator<'a> {
    type Item =  (u32, &'a Value);

    fn next(&mut self) -> Option<Self::Item> {
        if self.to_key > 0 && self.key >= self.to_key {
            return None;
        }

        let kmap_index = self.key/self.index.pps as u32;

        if kmap_index as usize >= self.index.key_map().slot_map.len() {
            return None;
        }

        let slot_map = &self.index.key_map().slot_map[kmap_index as usize];

        let ret = match slot_map {
            Some(map) => {
                let slot_index = (self.key as usize) %self.index.pps;
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
