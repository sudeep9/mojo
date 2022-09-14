use anyhow::Error;
use mojokv::{Store};

pub fn cmd(kvpath: &std::path::Path, name: &str, ver: u32, additional: bool, keys: bool) -> Result<(), Error> {
    let st = Store::readonly(kvpath, ver)?;
    let ret = st.get_index(name)?; //Bucket::load_index(&kvpath, name, ver)?;

    if ret.is_none() {
        println!("Bucket {} does not exists", name);
    }

    let (uncomp_sz, comp_sz, i) = ret.unwrap();

    let h = i.header();

    let st = if additional {
        Some(Store::load_state(kvpath)?)
    }else{
        None
    };

    println!("Format version    : {}", h.format_ver);
    println!("Minimum version   : {}", h.min_ver);
    println!("Maximum version   : {}", h.max_ver);
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