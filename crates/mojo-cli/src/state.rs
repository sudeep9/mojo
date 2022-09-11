use anyhow::Error;
use mojokv::{self, KVStore};
use std::mem::size_of;

pub fn cmd(kvpath: &std::path::Path, additional: bool) -> Result<(), Error> {
    let st = KVStore::load_state(kvpath)?;

    println!("Format version  : {}", st.format_ver());
    println!("Minimum version : {}", st.min_ver());
    println!("Active version  : {}", st.active_ver());
    println!("Pages per slot  : {}", st.pps());
    println!("Page size       : {}", st.page_size());
    println!("File header len : {}", st.file_page_sz());

    if additional {
        println!("----------------------------");
        println!("Size of KVStore : {} bytes", size_of::<KVStore>());
        println!("Size of MemIndex   : {} bytes", size_of::<mojokv::index::mem::MemIndex>());
        println!("Size of KeyMap  : {} bytes", size_of::<mojokv::KeyMap>());
        println!("Size of Value   : {} bytes", size_of::<mojokv::Value>());
        println!("Size of Slot    : {} bytes", size_of::<mojokv::Slot>());
    }

    Ok(())
}