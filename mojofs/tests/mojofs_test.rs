use std::path::Path;
use anyhow::Error;
use std::collections::HashMap;
use mojofs::{self, VFS, VFSFile};

fn remove_fs(rootpath: &Path) -> Result<(), Error> {
    if let Err(err) = std::fs::remove_dir_all(rootpath) {
        if err.kind() != std::io::ErrorKind::NotFound {
            return Err(err.into());
        }
    }
    Ok(())
}

fn setup() -> Result<String, Error> {
    let path = Path::new("./testfs");
    remove_fs(path)?;
    Ok(path.to_owned().to_str().unwrap().to_owned())
}

fn default_params(pagesz: u32) -> HashMap<String, String> {
    let mut h = HashMap::new();

    h.insert("ver".to_owned(), "1".to_owned());
    h.insert("pagesz".to_owned(), format!("{}", pagesz));
    h.insert("pps".to_owned(), "65536".to_owned());

    h
}

fn read_test(a: &mut VFSFile, nitems: usize, pagesz: u64, f: fn(usize) -> usize) -> Result<(), Error> {
    let mut buf = [0u8; 8];
    for i in 0usize..nitems {
        let off = i as u64 * pagesz;
        let n = f(i);
        a.pread(off, &mut buf)?;
        assert_eq!(n, usize::from_be_bytes(buf));
    }
    Ok(())
}

fn write_test(a: &mut VFSFile, nitems: usize, pagesz: u64, f: fn(usize) -> usize) -> Result<(), Error> {
    for i in 0usize..nitems {
        let off = i as u64 * pagesz;
        let n = f(i);
        a.pwrite(off, &n.to_be_bytes())?;
    }

    a.sync(1)?;
    Ok(())
}

fn write_read(a: &mut VFSFile, nitems: usize, pagesz: u64, f: fn(usize) -> usize) -> Result<(), Error> {
    write_test(a, nitems, pagesz, f)?;
    read_test(a, nitems, pagesz, f)?;

    Ok(())
}

#[test]
fn rw_same_version() -> Result<(), Error> {
    env_logger::init();
    let fspath = setup()?;    
    let mut fs_uri_opt = default_params(8);
    let mut opt = mojofs::OpenOptions::from_flags(326).unwrap();
    let nitems = 10;

    {
        let mut fs = VFS::default();
        fs.init(&fspath, &fs_uri_opt, opt.clone())?;
        let fsopt = fs.fs_options();

        let mut a = fs.open("a", opt.clone(), &mut opt)?;

        assert_eq!(fsopt.pagesz, 8);

        write_read(&mut a, nitems, fsopt.pagesz as u64, |n| n)?;
        assert_eq!(a.filesize()?, (fsopt.pagesz as u64) * nitems as u64);
        a.close()?;
        fs.commit()?;
        assert_eq!(fs.active_ver(), 2);

        let mut a = fs.open("a", opt.clone(), &mut opt)?;
        write_read(&mut a, nitems, fsopt.pagesz as u64, |n| n+10)?;
        assert_eq!(a.filesize()?, (fsopt.pagesz as u64) * nitems as u64);
        a.close()?;
        fs.commit()?;
        assert_eq!(fs.active_ver(), 3);
    }

    {
        let mut fs = VFS::default();
        opt.access = mojofs::OpenAccess::Read;
        fs_uri_opt.insert("ver".to_owned(), "1".to_owned());
        fs.init(&fspath, &fs_uri_opt, opt.clone())?;
        let fsopt = fs.fs_options();
        let mut a = fs.open("a", opt.clone(), &mut opt)?;

        read_test(&mut a, nitems, fsopt.pagesz as u64, |n| n)?;
    }
    
    {
        let mut fs = VFS::default();
        opt.access = mojofs::OpenAccess::Read;
        fs_uri_opt.insert("ver".to_owned(), "2".to_owned());
        fs.init(&fspath, &fs_uri_opt, opt.clone())?;
        let fsopt = fs.fs_options();
        let mut a = fs.open("a", opt.clone(), &mut opt)?;

        read_test(&mut a, nitems, fsopt.pagesz as u64, |n| n+10)?;
    }

    Ok(())
}