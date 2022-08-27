
pub mod vfs;
pub mod error;
pub mod open_options;
pub mod vfsfile;
mod native_file;
mod kvfile;

use std::ffi::CStr;
use std::collections::HashMap;

use libsqlite3_sys::{
    self,
    sqlite3_file,
    sqlite3_vfs};

use std::ffi::c_void;
use std::os::raw::{c_int, c_char};
pub use error::*;
pub use vfs::{VFS,AccessCheck};
pub use vfsfile::VFSFile;
pub use open_options::*;


#[repr(C)]
pub struct MojoFile {
    base: sqlite3_file,
    custom_file: *mut c_void,
    vfs: *mut sqlite3_vfs,
}

#[no_mangle]
pub extern "C" fn mojo_create() -> *mut sqlite3_vfs {
    let mut name_buf = Vec::from("mojo".as_bytes());
    name_buf.push(0);

    let fs_name_c  = name_buf.as_ptr();
    std::mem::forget(name_buf);

    let fs = Box::new(vfs::VFS::default());
    let p_app_data = Box::into_raw(fs) as *mut c_void;

    //println!("p_app_data={:?}", p_app_data);

    //let sql_vfs = unsafe {
    //    let raw_ptr = sqlite3_malloc64(std::mem::size_of::<sqlite3_vfs>() as u64);
    //    raw_ptr as *mut sqlite3_vfs
    //};
    let vfs = Box::into_raw(Box::new(sqlite3_vfs{
        iVersion: 1,
        szOsFile: (std::mem::size_of::<MojoFile>()) as i32,
        mxPathname: 512,
        pNext: std::ptr::null_mut(),
        zName: fs_name_c as *const i8,
        pAppData: p_app_data,
        xOpen: Some(mojo_open),
        xDelete: Some(mojo_delete),
        xAccess: Some(mojo_access),
        xFullPathname: Some(mojo_fullname),
        xDlOpen: Some(mojo_dlopen),
        xDlError: Some(mojo_dlerror),
        xDlSym: Some(mojo_dlsym),
        xDlClose: Some(mojo_dlclose),
        xRandomness: Some(mojo_randomness),
        xSleep: Some(mojo_sleep),
        xCurrentTime: Some(mojo_current_time),
        xCurrentTimeInt64: Some(mojo_current_time64),
        xGetLastError: Some(mojo_getlasterr),
        xSetSystemCall: None,
        xGetSystemCall: None,
        xNextSystemCall: None,
    }));

    vfs
}

#[no_mangle]
extern "C" fn mojo_open(vfs: *mut sqlite3_vfs, zname: *const c_char, file: *mut sqlite3_file, flags: c_int, out_flags: *mut c_int) -> c_int {
    let fs = getfs(vfs);

    let opt = open_options::OpenOptions::from_flags(flags as i32).unwrap();
    let mut out_opt = opt.clone();
    
    let file_str = if zname.is_null() {
        ""
    }else{
        let file_rs = unsafe{std::ffi::CStr::from_ptr(zname)};
        match file_rs.to_str() {
            Ok(file_str) => file_str,
            Err(_err) => {
                println!("mojo_open error in filepath");
                return libsqlite3_sys::SQLITE_CANTOPEN
            },
        }
    };

    if opt.kind.is_main() {
        let query_map = match extract_query_params(zname) {
            Ok(map) => map,
            Err(_err) => {
            return libsqlite3_sys::SQLITE_CANTOPEN
            }
        };

        if let Err(err) = fs.init(file_str, &query_map, opt.clone()) {
            log::error!("mojo_open init path={} err = {:?}", file_str, err);
            return libsqlite3_sys::SQLITE_CANTOPEN
        }
    }


    match fs.open(file_str, opt, &mut out_opt) {
        Ok(vfs_file) => {
            let mojo_file = unsafe {(file as *mut MojoFile).as_mut().unwrap()};
            let io_methods = Box::into_raw(Box::new(libsqlite3_sys::sqlite3_io_methods{
                iVersion: 1,
                xClose: Some(mojo_close),
                xRead: Some(mojo_read),
                xWrite: Some(mojo_write),
                xTruncate: Some(mojo_truncate),
                xSync: Some(mojo_sync),
                xFileSize: Some(mojo_filesize),
                xLock: Some(mojo_lock),
                xUnlock: Some(mojo_unlock),
                xCheckReservedLock: Some(mojo_check_reserved_lock),
                xFileControl: Some(mojo_file_control),
                xSectorSize: Some(mojo_sector_size),
                xDeviceCharacteristics: Some(mojo_device_char),
                xShmMap: None,
                xShmLock: None,
                xShmBarrier: None,
                xShmUnmap: None,
                xFetch: None,
                xUnfetch: None,
            }));
            mojo_file.base.pMethods = io_methods as *const libsqlite3_sys::sqlite3_io_methods;
            mojo_file.custom_file = Box::into_raw(vfs_file) as *mut c_void;
            mojo_file.vfs = vfs;
        },
        Err(err) => {
            log::error!("mojo_open path={} err = {:?}", file_str, err);
            return libsqlite3_sys::SQLITE_CANTOPEN;
        }
    }

    unsafe {
        if !out_flags.is_null() {
            *out_flags = out_opt.flags;
        }
    };

    libsqlite3_sys::SQLITE_OK
}


#[no_mangle]
extern "C" fn mojo_close(sfile: *mut sqlite3_file) -> c_int {
    unsafe {
        let mojo_file = (sfile as *mut MojoFile).as_ref().unwrap();
        let vfs_file = Box::from_raw(mojo_file.custom_file as *mut VFSFile);
        let fs = getfs(mojo_file.vfs);
        match fs.close(*vfs_file) {
            Ok(_) => {},
            Err(_err) => {
                return libsqlite3_sys::SQLITE_IOERR_CLOSE
            }
        }
    }
    libsqlite3_sys::SQLITE_OK
}

#[no_mangle]
extern "C" fn mojo_read(sfile: *mut sqlite3_file, ptr: *mut c_void, n: i32, off: i64) -> c_int {
    let file = get_file(sfile);
    let buf = unsafe{ std::slice::from_raw_parts_mut(ptr as *mut u8, n as usize)};

    let rc = match file.pread(off as u64, buf) {
        Ok(n) => {
            //let m = n.min(20);
            //log::debug!("after read n={} {:?}", n, &buf[..m]);

            if n == buf.len() {
                libsqlite3_sys::SQLITE_OK
            }else{
                //println!("short read");
                libsqlite3_sys::SQLITE_IOERR_SHORT_READ
            }
        }
        Err(err) => {
            log::error!("mojo_read id={} off={} blen={} err={:?}", file.id(), off, buf.len(), err);
            libsqlite3_sys::SQLITE_IOERR_READ
        },
    };

    rc
}

#[no_mangle]
extern "C" fn mojo_write(sfile: *mut sqlite3_file, ptr: *const c_void, n: i32, off: i64) -> c_int {
    let file = get_file_mut(sfile);
    let buf = unsafe{ std::slice::from_raw_parts(ptr as *const u8, n as usize)};

    let rc = match file.pwrite(off as u64, buf) {
        Ok(_) => libsqlite3_sys::SQLITE_OK,
        Err(err) => {
            log::error!("mojo_write id={} off={} blen={} err={:?}", file.id(), off, buf.len(), err);
            libsqlite3_sys::SQLITE_IOERR_WRITE
        },
    };

    rc
}

#[no_mangle]
extern "C" fn mojo_truncate(sfile: *mut sqlite3_file, new_sz: i64) -> c_int {
    let file = get_file_mut(sfile);

    let rc = match file.truncate(new_sz as u64) {
        Ok(_) => libsqlite3_sys::SQLITE_OK,
        Err(err) => {
            log::error!("mojo_truncate id={} new_sz={} err={:?}", file.id(), new_sz, err);
            libsqlite3_sys::SQLITE_IOERR_WRITE
        },
    };

    rc
}

#[no_mangle]
extern "C" fn mojo_sync(sfile: *mut sqlite3_file, flags: i32) -> c_int {
    let file = get_file_mut(sfile);

    let rc = match file.sync(flags) {
        Ok(_) => libsqlite3_sys::SQLITE_OK,
        Err(err) => {
            log::error!("mojo_sync id={} err={:?}", file.id(), err);
            libsqlite3_sys::SQLITE_IOERR_WRITE
        },
    };

    rc
}

#[no_mangle]
extern "C" fn mojo_filesize(sfile: *mut sqlite3_file, out_sz: *mut i64) -> c_int {
    let file = get_file(sfile);

    let rc = match file.filesize() {
        Ok(sz) => {
            unsafe{*out_sz = sz as i64;}
            libsqlite3_sys::SQLITE_OK 
        }
        Err(_) => libsqlite3_sys::SQLITE_IOERR_WRITE,
    };

    rc
}

#[no_mangle]
extern "C" fn mojo_access(vfs: *mut sqlite3_vfs, zname: *const c_char, flags: c_int, resout: *mut c_int) -> c_int {
    let path = match c_to_path(zname) {
        Ok(path) => path,
        Err(_) => {
            return libsqlite3_sys::SQLITE_IOERR_CONVPATH;
        }
    };

    let access_req = if flags == libsqlite3_sys::SQLITE_ACCESS_EXISTS {
        AccessCheck::Exists
    }else{
        AccessCheck::ReadWrite
    };

    let fs = getfs(vfs);
    match fs.access(&path, access_req) {
        Ok(status) => {
            unsafe{*resout = if status {1}else{0}}
        },
        Err(_) => {
            return libsqlite3_sys::SQLITE_IOERR_ACCESS;
        }
    }

    libsqlite3_sys::SQLITE_OK
}

#[no_mangle]
extern "C" fn mojo_delete(vfs: *mut sqlite3_vfs, zname: *const c_char, _syncdir: c_int) -> c_int {
    let path = match c_to_path(zname) {
        Ok(path) => path,
        Err(_) => {
            return libsqlite3_sys::SQLITE_IOERR_DELETE;
        }
    };

    let fs = getfs(vfs);
    match fs.delete(&path) {
        Ok(_) => {
            libsqlite3_sys::SQLITE_OK
        }
        Err(err) => {
            log::error!("mojo_delete path={:?} err={:?}", path, err);
            libsqlite3_sys::SQLITE_IOERR_DELETE
        }
    }
}

#[no_mangle]
extern "C" fn mojo_fullname(vfs: *mut sqlite3_vfs, zname: *const c_char, _nout: c_int, _zout: *mut c_char) -> c_int {
    let fs = getfs(vfs);

    let file_rs = unsafe{std::ffi::CStr::from_ptr(zname)};
    let file_str = match file_rs.to_str() {
        Ok(file_str) => file_str,
        Err(_err) => {
            log::error!("mojo_fullname error in filepath");
            return libsqlite3_sys::SQLITE_CANTOPEN;
        }
    };

    let _ = fs.fullpath(file_str);

    libsqlite3_sys::SQLITE_OK
}

#[no_mangle]
extern "C" fn mojo_dlopen(_arg1: *mut sqlite3_vfs, _zfilename: *const c_char) -> *mut c_void {
    std::ptr::null_mut()
}

#[no_mangle]
extern "C" fn mojo_dlerror(_arg1: *mut sqlite3_vfs, _nbyte: c_int, _zerrmsg: *mut c_char) {
}

//extern "C" fn           (_arg1: *mut sqlite3_vfs, _arg2: *mut c_void, _zSymbol: *const c_char)
#[no_mangle]
extern "C" fn mojo_dlsym(_arg1: *mut sqlite3_vfs, _arg2: *mut c_void, _zsymbol: *const c_char) 
-> Option<unsafe extern "C" fn(*mut sqlite3_vfs, *mut c_void, *const i8)> {
    None
}

#[no_mangle]
extern "C" fn mojo_dlclose(_arg1: *mut sqlite3_vfs, _arg2: *mut c_void) {
}

#[no_mangle]
extern "C" fn mojo_randomness(_arg1: *mut sqlite3_vfs, _nbyte: c_int, _zout: *mut c_char) -> c_int {
    libsqlite3_sys::SQLITE_OK
}

#[no_mangle]
extern "C" fn mojo_sleep(_arg1: *mut sqlite3_vfs, microseconds: c_int) -> c_int {
    std::thread::sleep(std::time::Duration::from_micros(microseconds as u64));
    libsqlite3_sys::SQLITE_OK
}

#[no_mangle]
extern "C" fn mojo_current_time(_arg1: *mut sqlite3_vfs, p: *mut f64) -> c_int {
    let now = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap()).as_secs() as f64;
    unsafe {
        *p = (2440587.5 + now / 864.0e5) * 864.0e5;
    }

    libsqlite3_sys::SQLITE_OK
}

#[no_mangle]
extern "C" fn mojo_current_time64(_arg1: *mut sqlite3_vfs, p: *mut i64) -> c_int {
    let now = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap()).as_secs() as f64;
    unsafe {
        *p = ((2440587.5 + now / 864.0e5) * 864.0e5) as i64;
    }

    libsqlite3_sys::SQLITE_OK
}

#[no_mangle]
extern "C" fn mojo_getlasterr(_arg1: *mut sqlite3_vfs, _arg2: c_int, _arg3: *mut c_char) -> c_int {
    libsqlite3_sys::SQLITE_OK
}

#[no_mangle]
extern "C" fn mojo_lock(_sfile: *mut sqlite3_file, _flags: c_int) -> c_int {
    libsqlite3_sys::SQLITE_OK
}

#[no_mangle]
extern "C" fn mojo_unlock(_sfile: *mut sqlite3_file, _flags: c_int) -> c_int {
    libsqlite3_sys::SQLITE_OK
}

#[no_mangle]
extern "C" fn mojo_check_reserved_lock(_sfile: *mut sqlite3_file, res_out: *mut c_int) -> c_int {
    unsafe{*res_out = 0;}
    libsqlite3_sys::SQLITE_OK
}

#[no_mangle]
extern "C" fn mojo_file_control(_sfile: *mut sqlite3_file, _op: c_int, _arg: *mut c_void) -> c_int {
    libsqlite3_sys::SQLITE_NOTFOUND
}

#[no_mangle]
extern "C" fn mojo_sector_size(_sfile: *mut sqlite3_file) -> c_int {
    0
}

#[no_mangle]
extern "C" fn mojo_device_char(_sfile: *mut sqlite3_file) -> c_int {
    libsqlite3_sys::SQLITE_OK
}

fn getfs(vfs: *mut sqlite3_vfs) -> &'static mut VFS {
    let fs = unsafe{
        ((*vfs).pAppData as *mut VFS).as_mut().unwrap()
    };

    fs
}

fn get_file_mut(sfile: *mut sqlite3_file) -> &'static mut VFSFile {
    let file = unsafe{
        let mojo_file = (sfile as *mut MojoFile).as_mut().unwrap();
        (mojo_file.custom_file as *mut VFSFile).as_mut().unwrap()
    };

    file
}

fn get_file(sfile: *mut sqlite3_file) -> &'static VFSFile {
    let file = unsafe{
        let mojo_file = (sfile as *mut MojoFile).as_ref().unwrap();
        (mojo_file.custom_file as *mut VFSFile).as_ref().unwrap()
    };

    file
}

fn c_to_path(cpath: *const c_char) -> Result<std::path::PathBuf, Error> {
    let file_rs = unsafe{std::ffi::CStr::from_ptr(cpath)};
    let file_str = file_rs.to_str()?;

    Ok(std::path::PathBuf::from(file_str))
}

#[no_mangle]
pub extern "C" fn mojofs_init_log() {
    env_logger::init();
}


fn extract_query_params(filepath: *const c_char) -> Result<HashMap<String, String>, Error> {
    let mut map = HashMap::new();
    if filepath.is_null() {
        return Ok(map);
    }

    let mut itr: *const c_char = filepath;
    let mut parse_key = true;
    let mut key: &str = "dummy";
    let mut value: &str;

    unsafe {
        while key.len() > 0 {
            while *itr != 0 {
                itr = itr.add(1);
            }
            itr = itr.add(1);

            let s = CStr::from_ptr(itr).to_str()?;
            if parse_key {
                key = s;
                parse_key = false;
            }else{
                value = s;
                parse_key = true;
                map.insert(key.to_owned(), value.to_owned());
            }
        } 
    }

    Ok(map)
}