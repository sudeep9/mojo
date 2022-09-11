"""Multiple commands for testing"""

import subprocess
import sqlite3

MOJOKV_CLI=None

class TestConfig:
    """Config for test db"""

    def __init__(self, page_sz=4096, journal_mode="WAL", vac_mode="NONE"):
        self.page_sz = page_sz
        self.journal_mode = journal_mode
        self.vac_mode = vac_mode

    def __repr__(self):
        return f"page_sz={self.page_sz} journal={self.journal_mode}"

def vacuum(cur):
    """Vacuum"""
    cur.execute("vacuum")

def opendb(cfg, db_path, ver="1", mode=""):
    """Open sqlite db using cfg"""

    if mode == "":
        conn_str = f"file:{db_path}?vfs=mojo&ver={ver}&pagesz={cfg.page_sz}&pps=65536"
    else:
        conn_str = f"file:{db_path}?vfs=mojo&mode=ro&ver={ver}&pagesz={cfg.page_sz}&pps=65536"

    conn = sqlite3.Connection(conn_str, uri=True)

    if mode != "ro":
        conn.execute(f"PRAGMA page_size={cfg.page_sz}")
        conn.execute(f"PRAGMA journal_mode={cfg.journal_mode}")
        conn.execute(f"PRAGMA auto_vacuum={cfg.vac_mode}")

    return conn

def mkdir(dir_path):
    """"Make dir"""
    subprocess.run(["mkdir", "-p", dir_path], check=True, capture_output=True)

def commit_version(dbpath):
    '''commit_version commits version for given dbpath'''
    subprocess.run([MOJOKV_CLI, dbpath, "commit"], check=True, capture_output=True)

def create_table_person(cur):
    """Create table Person"""
    cur.execute("""create table if not exists person(
        name text primary key,
        age integer,
        id integer
    )""")

    cur.execute("create index if not exists person_idx_1 on person(id)")

def get_row_count(cur, table):
    """Get row count of table"""
    row = cur.execute(f"select count(*) from {table}").fetchone()
    if row is None or row[0] is None:
        return 0
    return int(row[0])

def table_person_count(conn):
    """Get row count of Person"""
    cur = conn.cursor()
    row = cur.execute("select count(*) from person").fetchone()
    return int(row[0])

def get_max_id_person(cur):
    """get max id"""
    row = cur.execute("select max(id) from person").fetchone()
    if row is None or row[0] is None:
        return 0
    return int(row[0])

def insert_table_person(cur, count):
    """Insert into person table"""

    max_id = get_max_id_person(cur)

    for n in range(max_id+1, max_id+count+1):
        name = f"name-{n}"
        cur.execute("insert into person(name,age,id) values(?,?,?)", [name,n,n])

def delete_table_person(cur, from_id, to_id):
    """Delete from person table"""

    cur.execute(f"delete from person where id>={from_id} and id <= {to_id}")

def drop_table_person(cur):
    """"Drop table person"""

    cur.execute("drop table person")

def copy_table_person(cur, new_table):
    """"Copy table from person table"""
    cur.execute(f"create table if not exists {new_table} as select name,age,id from person")
    cur.execute(f"create index if not exists {new_table}_idx_1 on {new_table}(id)")