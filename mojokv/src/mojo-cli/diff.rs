use anyhow::Error;
use mojokv::KVStore;

pub fn cmd(kvpath: &std::path::Path, fver: u32, tver: u32) -> Result<(), Error> {
    if fver >= tver {
        return Err(Error::msg("'from' version cannot be greater than or equal 'to' version"));
    }

    let state = KVStore::load_state(kvpath)?;
    let f_index = KVStore::load_index(kvpath, fver)?;
    let t_index = KVStore::load_index(kvpath, tver)?;

    let mut key = 0u32;
    for (slot_index, t_slot) in t_index.kmap.slot_map.iter().enumerate() {
        let f_slot = &f_index.kmap.slot_map[slot_index];

        match (f_slot, t_slot) {
            (Some(fs), Some(ts)) => {
                if fs.len() != ts.len() {
                    return Err(Error::msg(format!("Slot length mismatch {} {}", fs.len(), ts.len()))); 
                }

                for (j,(fv, tv)) in std::iter::zip(fs, ts).enumerate() {
                    if fv.get_ver() != tv.get_ver() {
                        println!("M k={} fv={} tv={} fo={} to={}",
                            key+j as u32, 
                            fv.get_ver(),
                            tv.get_ver(),
                            fv.get_off(),
                            tv.get_off());
                    }
                }
            },
            (Some(_fs), None) => {
                println!("D {} -> {} deleted", key, key + state.pps);
            },
            (None, Some(_ts)) => {
                println!("A {} -> {} added", key, key + state.pps);
            },
            (None, None) => {},
        }

        key += state.pps;
    }

    Ok(())
}