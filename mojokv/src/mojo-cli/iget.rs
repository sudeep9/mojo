use anyhow::Error;
use mojokv::{KVStore,BucketOpenMode};

pub fn cmd(kvpath: &std::path::Path, bucket: &str, ver: u32, key: u32) -> Result<(), Error> {
    let mut st = KVStore::readonly(&kvpath, ver)?;
    let b = st.open_bucket(bucket, BucketOpenMode::Read)?;

    println!("Max key: {}", b.max_key());
    match b.get_key(key) {
        Some(val) => {
            println!("value: {:?}", val);
        },
        None => {
            println!("Key not found")
        }
    }
    Ok(())
}