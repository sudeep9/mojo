use anyhow::Error;
use mojokv::{Store,BucketOpenMode};

pub fn cmd(kvpath: &std::path::Path, bucket: &str, ver: u32, key: u32) -> Result<(), Error> {
    let st = Store::readonly(&kvpath, ver)?;
    let b = st.open(bucket, BucketOpenMode::Read)?;

    println!("Max key: {}", b.max_key());
    match b.get_key(key)? {
        Some(val) => {
            println!("value: {:?}", val);
        },
        None => {
            println!("Key not found")
        }
    }
    Ok(())
}