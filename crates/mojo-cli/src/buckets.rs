use anyhow::Error;
use mojokv::BucketMap;

pub fn cmd(kvpath: &std::path::Path, ver: u32) -> Result<(), Error> {
    let bmap = BucketMap::load(kvpath, ver)?;    

    for (bucket_name, ver) in bmap.iter() {
        println!("{} -> {}", bucket_name, ver);
    }
    Ok(())
}