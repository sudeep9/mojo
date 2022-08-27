"""" Perf test for mojo filesystem """

import sqlite3
import os
import shutil
import time
import sys

def rm_dir(path):
    '''Remove dir. Ignore file not found error'''
    try:
        if os.path.isdir(path):
            rm_fr(path)
        else:
            os.unlink(path)
    except FileNotFoundError:
        pass

def rm_fr(path):
    '''Equivalent of rm -fr'''

    journal_path = path + "-journal"
    wal_path = path + "-wal"

    rm_dir(journal_path)
    rm_dir(wal_path)

    if not os.path.exists(path):
        return
    if os.path.isfile(path) or os.path.islink(path):
        os.unlink(path)
    else:
        shutil.rmtree(path)

def _mojo_conn_str(db_path, ver="1", mode=""):
    """open database"""
    if mode == "":
        conn_str = f"file:{db_path}?vfs=mojo&ver={ver}&pagesz=4096&pps=65536"
    else:
        conn_str = f"file:{db_path}?vfs=mojo&mode=ro&ver={ver}&pagesz=4096&pps=65536"

    return conn_str

def _std_conn_str(db_path, mode=""):
    if mode == "":
        conn_str = f"file:{db_path}"
    else:
        conn_str = f"file:{db_path}?mode=ro"
    return conn_str

def open_db(db_path, vfs="mojo", ver="1", mode=""):
    """Open database"""
    if vfs == "mojo":
        conn_str=_mojo_conn_str(db_path, ver=ver, mode=mode)
    else:
        conn_str=_std_conn_str(db_path, mode=mode)

    conn = sqlite3.Connection(conn_str, uri=True)

    if mode != "ro":
        conn.execute("PRAGMA page_size=4096")
    return conn


def create_table(conn):
    """create table"""
    conn.execute("create table test(s text primary key)")

def insert_rows(conn, row_count, ver, suffix=""):
    """create table"""

    cur = conn.cursor()
    for i in range(row_count):
        val = f"{ver}-text-{i}{suffix}"
        cur.execute("insert into test values(?)", (val,))

def count_rows(conn, condition=""):
    """count rows"""

    cur = conn.cursor()
    if condition == "":
        row = cur.execute("select count(*) from test").fetchone()
    else:
        row = cur.execute(f"select count(*) from test where {condition}").fetchone()
    
    return int(row[0])

def select_rows(conn, condition=""):
    """select rows"""

    cur = conn.cursor()
    if condition == "":
        rows = cur.execute("select * from test")
    else:
        rows = cur.execute(f"select * from test where {condition}")
    
    count = 0
    for _r in rows:
        count += 1
    return count

def update_text_rows(conn):
    """Update rows"""
    key = "odd-update-text"
    conn.execute("update test set s = ? where s like 'odd%'",(key,))
    conn.commit()


def load_extension(mojo_lib):
    """load_ext"""
    print("using libpath =", mojo_lib)

    con = sqlite3.connect(":memory:")
    # enable extension loading
    con.enable_load_extension(True)
    con.execute(f"select load_extension('{mojo_lib}')")
    con.execute("pragma page_size=4096")
    con.enable_load_extension(False)
    con.close()

ROW_COUNT=10000000

def perf_insert(conn):
    """"Perf insert"""

    start = time.time()
    create_table(conn)
    insert_rows(conn, ROW_COUNT, "1")
    conn.commit()
    end = time.time()
    return end-start

def perf_select(conn):
    """"Perf select"""

    start = time.time()
    count = select_rows(conn)
    end = time.time()
    print("select iter count:", count)
    return end-start

def perf_count_rows(conn):
    """"Perf select"""

    start = time.time()
    count = count_rows(conn, "s like '%abc%'")
    print("row count:", count)
    end = time.time()
    return end-start

def perf_update_rows(conn):
    """"Perf update rows"""

    start = time.time()
    update_text_rows(conn)
    end = time.time()
    return end-start

if __name__ == '__main__':
    if len(sys.argv[1:]) < 1:
        print("Error: missing extension library path")
        sys.exit(1)

    ext_path = sys.argv[1]
    load_extension(ext_path)

    STD_DBPATH="./perf-std.db"
    MOJO_DBPATH="./perf-mojo.db"

    try:
        rm_fr(STD_DBPATH)
        rm_fr(MOJO_DBPATH)

        perf_list=[
            ("insert", perf_insert),
            ("update rows", perf_update_rows),
            ("select", perf_select),
            ("row count", perf_count_rows),
        ]

        for desc, perf_fn in perf_list:
            print("Running perf for:", desc)
            e = []
            for dbpath, vfs in [(STD_DBPATH, "std"), (MOJO_DBPATH, "mojo")]:
                dbconn = open_db(dbpath, vfs=vfs)
                elapsed = perf_fn(dbconn)
                e.append(elapsed)
                print(f"\tvfs={vfs} time elapsed (s):", elapsed)
                dbconn.close()
            ratio = round(e[1]/e[0], 3)
            print(f"\tMojo takes {ratio} times than std vfs")
            print("------------------------")

    except Exception as e:
        raise e
    finally:
        rm_fr(STD_DBPATH)
        rm_fr(MOJO_DBPATH)
