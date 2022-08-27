use anyhow::Error;
use mojokv::KVStore;

pub fn cmd(kvpath: &std::path::Path, sz: usize) -> Result<(), Error> {
    let st = KVStore::load_state(&kvpath)?;
    if sz % (st.page_size() as usize) != 0 {
       return Err(Error::msg(format!("Error: truncate size is not multiple of page sz ({})", st.page_size())));
    }

    let mut store = KVStore::writable(kvpath, st.page_size(), Some(st.pps()))?;

    store.truncate(sz)?;
    store.sync()?;

    Ok(())
}