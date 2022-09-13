pub mod mem;
use std::{collections::HashSet, hash::Hash};

use crate::Error;
use crate::value::Value;
use serde::{Serialize, Deserialize};

pub const MOJO_INDEX_MAGIC: &'static str = "mojo_index";

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexHeader {
    pub magic: String, 
    pub format_ver: u32,
    pub min_ver: u32,
    pub max_ver: u32,
    pub vset: HashSet<u32>,
    pub active_ver: u32,
    pub max_key: isize,
    pub pps: usize,
}

impl IndexHeader {
    pub fn new(pps: usize) -> Self {
        let mut vset = HashSet::new();
        vset.insert(1);

        IndexHeader {
            magic: MOJO_INDEX_MAGIC.to_owned(),
            format_ver: 1,
            min_ver: 1,
            max_ver: 1,
            vset,
            active_ver: 1,
            pps,
            max_key: -1,
        }
    }
}

pub trait Index {
    fn put(&mut self, key: u32, off: u32) -> Result<(), Error>;
    fn get(&self, key: u32) -> Result<Option<&Value>, Error>;
    fn truncate(&mut self, key: u32) -> Result<(), Error>;
}

pub trait IndexSerde {
    fn serialize<I: Index, W: std::io::Write>(idx: &I, w: &mut W) -> Result<(), Error>;
    fn deserialize<I: Index, R: std::io::Read>(idx: &I, r: &mut R) -> Result<I, Error>;
}