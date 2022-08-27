use anyhow::Error;
use mojokv::{KVStore, Bucket};

pub fn cmd(kvpath: &std::path::Path, name: &str, ver: u32, additional: bool, keys: bool) -> Result<(), Error> {
    let (comp_sz, uncomp_sz, i) = Bucket::load_index_header(&kvpath, name, ver)?;

    let st = if additional {
        Some(KVStore::load_state(kvpath)?)
    }else{
        None
    };

    println!("Format version    : {}", i.format_ver);
    println!("Minimum version   : {}", i.min_ver);
    println!("Active version    : {}", i.active_ver);
    println!("Pages per slot    : {}", i.pps);
    println!("Maximum key       : {}", i.max_key);
    println!("Compressed size   : {}", comp_sz);
    println!("Uncompressed size : {}", uncomp_sz);

    if let Some(st) = st {
        println!("----------------------");
        println!("Logical size      : {}", st.page_size() * (i.max_key() + 1) as u32);
    }

    if keys {
        let i = Bucket::load_index(kvpath, name, ver)?;
        println!("----------------------");
        println!("keys");
        for (key, val) in i.iter(0, 0) {
            println!("   {} {:?}", key, val);
        }
    }
    

    Ok(())
}