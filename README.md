# MojoFS

MojoFS is a versioning, userspace filesystem for sqlite DB. It is tailor made for sqlite and not supposed to be used as a general purpose fs.

The main feature of the fs is versioning/snapshotting. Only one version is writable and all the old versions are immutable.

- [MojoFS](#mojofs)
- [License](#license)
- [Development status](#development-status)
- [Build](#build)
- [Usage](#usage)
  - [Create databse using mojo and insert few records](#create-databse-using-mojo-and-insert-few-records)
  - [Select data](#select-data)
  - [Commit the database](#commit-the-database)
  - [Write to active version=2](#write-to-active-version2)
  - [Read old version=1](#read-old-version1)
  - [Read active version=2:](#read-active-version2)
- [Limits](#limits)
- [Testing](#testing)
- [Performance](#performance)
- [Road to v1.0](#road-to-v10)

# License

At present there is no license and to my knowledge it is very restrictive.
This will change. I have not made up my mind on the exact license.

# Development status


|Item|Value|
|-------|------|
|Quality|pre-alpha|
|Maintainance|active|

# Build

At present only mac/linux is supported. Windows is not supported only because I do not have windows machine. This may change in future.

The build expects the following in the environment:

1. Meson Build + Ninja (see [here](https://mesonbuild.com/Getting-meson.html))
2. C compiler (gcc or clang)
3. Rust version v1.59+
4. sqlite headers + libraries
5. Optional: python3 for testing (most versions should be ok)

```
git clone github.com/sudeep9/mojo
cd mojo
./build.sh release
```

Following artifacts will be in `build` dir:

* `build/libmojo.dylib` (`.so` extension in linux) => The sqlite extension/vfs 
* `build/mojo-cli` => mojo cli tool to manage the file system

# Usage

All the examples below uses sqlite3 binary. However, you can use any bindings of sqlite.

## Create databse using mojo and insert few records

```
rm -fR a.db
sqlite3 <<EOF
pragma page_size = 4096;
.load ./build/libmojo
.open 'file:a.db?vfs=mojo&pagesz=4096'
create table if not exists test (n int);
insert into test values(1);
insert into test values(2);
EOF
```

The `.load` loads the extension and registers the mojo vfs.

The `.open` creates the database `a.db`. The mojofs creates a dir `a.db` instead of a file.

The `pagesz=4096` is the page size which the fs will use.
This needs to be same as the page size used by sqlite.
I am not aware of any way this can be detected automatically by VFS layer.
Unfortunately its upto the user to ensure that these values are consistent. 
If these values mismatch the database is bound to be corrupted.

## Select data

```
sqlite3 <<EOF
pragma page_size = 4096;
.load ./build/libmojo
.open 'file:a.db?vfs=mojo&pagesz=4096'
select * from test;
EOF
```

Output:

```
1
2
```

## Commit the database

```
./build/mojo-cli ./a.db commit
```

Output:

```
active version before commit: 1
active version after commit: 2
```

New databases always start with version=1. The commit advances the version number. Here the version=1 is immutable and version=2 is now writable. Lets write again

## Write to active version=2

```
sqlite3 <<EOF
pragma page_size = 4096;
.load ./build/libmojo
.open 'file:a.db?vfs=mojo&pagesz=4096'
insert into test values(3);
insert into test values(4);
EOF
```

## Read old version=1

```
sqlite3 <<EOF
pragma page_size = 4096;
.load ./build/libmojo
.open 'file:a.db?vfs=mojo&pagesz=4096&ver=1&mode=ro'
select * from test;
EOF
```

We specify the `ver=1` and `mode=ro` (i.e. readonly)

Output:

```
1
2
```
## Read active version=2:

```
sqlite3 <<EOF
pragma page_size = 4096;
.load ./build/libmojo
.open 'file:a.db?vfs=mojo&pagesz=4096'
select * from test;
EOF
```

Output:

```
1
2
3
4
```

# Limits

For a page size = 4KB (which is the default for sqlite for some years now) following are the limits:

* Max sqlite db file size (KB) = `pow(2,32) * 4` = 17179869184 KB or 16TB 

  16TB is logical size i.e. file size reported by stat like call in any version. Since there could be multiple versions of the file, the total size of all such versions taken together can exceed 16TB.

* Max version num = `pow(2,24)` = 16777216 or 16 million

  To put 16M versions in perspective, even if you create a version every 1 min, it will take ~31.92 years to reach the max version.

Note: All these limits are actually artificial to keep the memory usage reasonable. In future, these will be tunable and also have the ability to baseline the versions.

# Testing

I wanted to use sqlite [test harness](https://www.sqlite.org/th3.html) but it requires license. Quoting from the test harness link:

> SQLite itself is in the public domain and can be used for any purpose. But TH3 is proprietary and requires a license.

Instead I have `testdb.sql` for black box testing and `perftest.py` for perf tests.
At present the `testdb.sql` tests combinations of the following:

```     
page_sizes = [4096]
journal_modes = ["OFF", "WAL", "MEMORY", "DELETE", "TRUNCATE", "PERSIST"]
vacuum_modes = ["NONE", "FULL", "INCREMENTAL"]
```

For each of the combination, there are about ~11 subtests so in all 18 x 11 = 198 tests.
These are early days and off-course there is a long way to go.

To run the full suite:

```
python3 testdb.py build/libmojo full
```


# Performance

About `10_000_000` rows are inserted and then for reading we select the rows and get the row count.
Finally it updates half the rows.

To run the perf test:

```
MOJOKV_CLI=build/mojo-cli python3 perftest.py ./build/libmojo
```

Output on 2018/19 macbook:

```
Running perf for: insert
	vfs=std time elapsed (s): 20.524982929229736
	vfs=mojo time elapsed (s): 21.202611923217773
	Mojo takes 1.033 times than std vfs
------------------------
Running perf for: update rows
	vfs=std time elapsed (s): 2.871242046356201
	vfs=mojo time elapsed (s): 2.439574956893921
	Mojo takes 0.85 times than std vfs
------------------------
Running perf for: select
select iter count: 10000000
	vfs=std time elapsed (s): 8.659775018692017
select iter count: 10000000
	vfs=mojo time elapsed (s): 5.907814025878906
	Mojo takes 0.682 times than std vfs
------------------------
Running perf for: row count
row count: 0
	vfs=std time elapsed (s): 2.96425199508667
row count: 0
	vfs=mojo time elapsed (s): 1.5106308460235596
	Mojo takes 0.51 times than std vfs
------------------------
```

The writes being only `1.033` times worse is in line with my expectations. However, I am investigating why the reads are so better with mojo. 

My guess as of now is that in the standard default vfs at https://github.com/sqlite/sqlite/blob/master/src/os_unix.c does not use pread, whereas mojo uses pread. Lack of pread results into two system call i.e. seek + read. This ***might*** explain the perf difference. This need further confirmation though.

See the comment and code in the c file above:

```
** ... Since SQLite does not define USE_PREAD
** in any form by default, we will not attempt to define _XOPEN_SOURCE.
** See tickets #2741 and #2681.
```

In seekAndRead function:

```
#if defined(USE_PREAD)
    got = osPread(id->h, pBuf, cnt, offset);
    SimulateIOError( got = -1 );
#elif defined(USE_PREAD64)
    got = osPread64(id->h, pBuf, cnt, offset);
    SimulateIOError( got = -1 );
#else
    newOffset = lseek(id->h, offset, SEEK_SET);
    SimulateIOError( newOffset = -1 );
    if( newOffset<0 ){
      storeLastErrno((unixFile*)id, errno);
      return -1;
    }
    got = osRead(id->h, pBuf, cnt);
```

# Road to v1.0

It needs atleast the following:

- [ ] Top-notch unit & black box test coverage
- [ ] Ease of use e.g debugability, add more mojo-cli admin commands
- [ ] Ability to diff the versions
- [ ] Ability to delete versions
- [ ] Ability to merge versions (not like git merge)
- [ ] Ability to recover from corrupted fs.
- [ ] Stablize on-disk format
- [ ] User guide

A lot of the above needs to be clearly defined.
