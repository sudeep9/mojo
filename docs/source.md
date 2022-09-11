
Whether you want to just read or contribute to mojo, this document describes the code to get you started.

*__Note__*: This is not a design document.

## Bird's eye view

* All the rust code is under `crates` folder
* All the docs are under `docs` folder
* The `sqlite-ext` folder has C code which is compiled down to shared lib
* The `test-scripts` has various assorted test scripts which includes perf & black-box test


## Crates

### mojokv

This is the KV which powers the mojofs.

* `store.rs` has the main store object. Buckets are "opened" using a store object
* `bucket.rs` has the bucket object. A bucket has get & put methods. Each bucket has an index.
* `index/mem.rs` has the `MemIndex` which is in-memory index which has the mapping `offset -> (physical offset, version)`. Each index has KeyMap.
* `keymap.rs` The index is split into slots and a vector such slots are wrapped in KeyMap. 
* `state.rs` has the state object which reflects the current state of the kv

### mojofile

Abstracts out the notion of file. This is the code which does the actual IO. It will have different implementations including remote KV store.

* `nix.rs` implements unix based file

### mojofs

Mojofs is the filesystem which is powered by mojokv. Each user file in fs maps to a bucket in mojokv.

* `vfs.rs` has FS like operations like `open`, `delete`, `access`, etc
* `kvfile.rs` has file like object which is implemented using mojokv, hence the name.
* `native_file.rs` is the regular passthrough file object (uses std read/write)
* `vfsfile.rs` has the object VFSFile which either is a kvfile or nativefile. At present everything is kvfile. The native file will be used for transient/temp files which does not need versioning. This is an optimization.
* `lib.rs` has vfs functions needed by sqlite e.g. `fn mojo_read(sfile: *mut sqlite3_file, ptr: *mut c_void, n: i32, off: i64)`


### mojo-cli

This a CLI utility to control the mojokv. Each command in mojo-cli maps to a `*.rs` file. Example: `commit` command can be found in `commit.rs`
