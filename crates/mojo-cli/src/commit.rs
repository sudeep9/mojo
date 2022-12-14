use anyhow::Error;
use mojokv::Store;

pub fn cmd(kvpath: &std::path::Path) -> Result<(), Error> {
    let st = Store::writable(&kvpath, false, None, None)?;

    println!("active version before commit: {}", st.active_ver());
    let new_ver = st.commit()?;
    println!("active version after commit: {}", new_ver);
    Ok(())
}