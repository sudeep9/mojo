- [Design choices](#design-choices)
- [Design](#design)
  - [Layers](#layers)
  - [MojoKV](#mojokv)
    - [Index](#index)
  - [MojoFS](#mojofs)

# Design choices

Mojofs is a versioning file-system for sqlite database. It is a completely tailor made for sqlite and is not
to be used as a general purpose file-system. This allows the fs to make certain assumptions which in turn drives design choices. 

Following are the assumptions about any sqlite fs, which I think are reasonable:

* Small number of files
* Large files
* Very flat folder structure
* Read/write in terms of fixed page size (or there multiple) dominates compared to any random offset.

This influences the way we store namespace (i.e. folder/file names), the index (i.e. mapping between pages and offsets) and internal decoupling (e.g mojofs itself doesn't do much but instead uses mojokv as storage layer)

# Design

## Layers

The layers of mojofs are:

Sqlite -> Mojofs extension lib -> mojofs -> mojokv

The sqlite could be the cli (i.e. sqlite3 binary) or sqlite C API or any of its bindings. 
The fs is developed as both an sqlite extension and vfs, which is compiled down to a shared library.
This shared library is loaded as extension which then registers the VFS=mojo with the sqlite.
Mojofs implements the sqlite's [VFS interface](https://www.sqlite.org/vfs.html) which asks file system like apis to be implemented.

The fs uses mojokv (Mojo Key-Value store) to actually store the data. The KV has a notion of 'bucket' which the fs creates for each new file asked by sqlite. All the buckets i.e files taken together are versioned.

The MojoKV is tailor made for the needs of sqlite and as such is not a general purpose key-value store.

## MojoKV

MojoKV is the core storage layer which handles the index and actual data files.
The KV has a notion of bucket, on which read/write happens.

Each bucket has an index which is a mapping of (Page No) => (New Page No, version)

### Index

The write api at [File IO methods](https://www.sqlite.org/c3ref/io_methods.html) looks like below:

```
int (*xWrite)(sqlite3_file*, const void*, int iAmt, sqlite3_int64 iOfst);
```

The `sqlite3_file*` is the file handle, `const void*, int iAmt` are the pointer to data and its length. The `iOfst` is the offset at which data needs to be written. Mojofs has versions/snapshots and the sqlite API does not know anything about it. This means that when write is called, mojofs needs to know to which version the data should be written.

The sqlite divides the file into pages. This is configurable when db is created for the first time, but assume 4KB for this document. We can logically think of a file and its versions as below:

|      |1|2|3|4|
|------|-|-|-|-|
|Page 0|1|2|3|4|
|Page 1|1|2| |4|
|Page 2|1| | | |
|Page 3|1| |3|4|
|Page 4|1| |3| |

The file above has 5 pages of 4KB each and are depicted as rows. The columns are the version numbers.
The value for page 0 and version 2 is 2. This means that the page 0 was modified in version 2.
For page 2 and version 2 the value is empty and it means the page was not modified from previous version.
When the file is created new, naturally all the pages will be marked as version 1.

When page 2 and version=3 needs to be read, it actually needs to be fetch the page from version=1.

Essentially the page no should map to a certain location on disk.
For simple, unversioned file, the page number translates to an offset in file i.e. page no x page size.
But for versioned file, the mapping of between the tuple (Pg No, Version) => \<Location where page is stored\> is needed.

This nicely yields itself to be stored in a key-value store.

## MojoFS

Each sqlite database is created as a directory instead of a single file.
For each file name = F and version = V the mojokv stores the file with name F.V.
The filename (=F) is chosen by the fs. 

So for a given sqlite db 'a.db' being created for the first time the fs will create the following on disk:

```
sudeep@local-3 mojo-rs % tree ./a.db 
./a.db
├── a.db_d.1
├── a.db_i.1
├── mojo.bmap.1
├── mojo.init
└── mojo.state
...
```

So the fs creates the dir = `a.db`. The sqlite issues open call for the main db file i.e. test.db. 
The fs adds `_d` (d=data) to the name creates `a.db_d.1`. The `.1` is the version.

The `a.db_i.1` is the index file, which is internal to the fs.

The `mojo.*` files are files created/for the mojokv.

When a version 2 is created it will create a file: `a.db_d.2`. This file will contain only those pages which were modified in that version. 
As a result it will also create the index file `a.db_i.2`.
