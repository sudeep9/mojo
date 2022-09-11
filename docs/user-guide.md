
This document assumes that you have followed the build instructions and/or have the libmojo shared library.

- [Opening/Creating the database](#openingcreating-the-database)
- [Committing database](#committing-database)
- [Committing MojoFS vs Committing Database](#committing-mojofs-vs-committing-database)
- [Reading old version](#reading-old-version)


## Opening/Creating the database

Mojofs is compiled down to a shared library. 
It should be loaded as extension in sqlite before it can be used.

```shell
sqlite3 <<EOF
pragma page_size = 4096;
.load ./path/to/libmojo
.open 'file:a.db?vfs=mojo&pagesz=4096'
EOF
```

In python3 sqlite binding:

```python
import sqlite3

mojo_lib="<path/to/libmojo>"
db_path="a.db"

con = sqlite3.connect(":memory:")
con.enable_load_extension(True)
con.execute(f"select load_extension('{mojo_lib}')")
con.execute("pragma page_size=4096")
con.enable_load_extension(False)
con.close()

conn_str = f"file:{db_path}?vfs=mojo&pagesz=4096"
conn = sqlite3.Connection(conn_str, uri=True)
```

Let's decode the URI in the open `.open 'file:a.db?vfs=mojo&pagesz=4096'`:

* `a.db` => Name of the database
* `vfs=mojo` => Name of the MojoFS
* `pagesz=4096` => Page size used by the fs.
  This should same as the page size in the pragma `pragma page_size = 4096`

Once set, the page size cannot be changed.

When the database is created for the first time, it starts with version=1.
Version numbers are ever incrementing and the highest version number is writable 
and old versions are read-only. 

The fs has to be commited to make the current active version as read-only.

## Committing database

The `mojo-cli` is the tool for administration of the mojofs. To commit the database `a.db`:

```shell
mojo-cli ./a.db commit
```

Commit advances version number by 1. So if current active version=1 then after committing, a new version=2
will be created and version=1 will be read-only now.

Committing FS is really a cheap operation. It only manipulates the metadata of the FS and no data movement
is involved.

## Committing MojoFS vs Committing Database

Committing the fs is different than committing the database. You can continue to use the database
in the active version as long as you want. This means multiple transactions can be initiated with
all the DB commits and rollbacks, all in a single version. 

Committing the database makes sqlite issue `fsync()` call which makes the data written to disk durable.
It is recommended to commit the fs when databases is just committed/rolled-backed.
The FS is not aware any uncommitted transactions and committing the fs midway can cause undefined behaviour.


## Reading old version

Pass the `ver=<num>` and `mode=ro` to open it in readonly mode:

```
.open 'file:a.db?vfs=mojo&pagesz=4096&ver=2&mode=ro'
```