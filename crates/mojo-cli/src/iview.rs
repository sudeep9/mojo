use anyhow::Error;
use mojokv::{KVStore, Bucket};

pub fn cmd(kvpath: &std::path::Path, name: &str, ver: u32, additional: bool, keys: bool) -> Result<(), Error> {
    let (uncomp_sz, comp_sz, i) = Bucket::load_index(&kvpath, name, ver)?;

    let h = i.header();

    let st = if additional {
        Some(KVStore::load_state(kvpath)?)
    }else{
        None
    };

    println!("Format version    : {}", h.format_ver);
    println!("Minimum version   : {}", h.min_ver);
    println!("Active version    : {}", h.active_ver);
    println!("Pages per slot    : {}", h.pps);
    println!("Maximum key       : {}", h.max_key);
    println!("Compressed size   : {}", comp_sz);
    println!("Uncompressed size : {}", uncomp_sz);

    if let Some(st) = st {
        println!("----------------------");
        println!("Logical size      : {}", st.page_size() * (h.max_key + 1) as u32);
    }

    if keys {
        println!("----------------------");
        println!("keys");
        for (key, val) in i.iter(0, 0) {
            println!("   {} {:?}", key, val);
        }
    }
    

    Ok(())
}