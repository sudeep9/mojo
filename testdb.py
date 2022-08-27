"""" Tests for mojo filesystem """

import os
import sys
import unittest
import sqlite3
import shutil
import subprocess

MOJOKV_CLI=None

class TestConfig:
    '''Config for test db'''

    def __init__(self, page_sz=4096, journal_mode="WAL", vac_mode="NONE"):
        self.page_sz = page_sz
        self.journal_mode = journal_mode
        self.vac_mode = vac_mode

    def __repr__(self):
        return f"page_sz={self.page_sz} journal={self.journal_mode}"

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

def commit_version(dbpath):
    '''commit_version commits version for given dbpath'''
    subprocess.run([MOJOKV_CLI, dbpath, "commit"], check=True, capture_output=True)

def create_table(conn, table):
    """create table"""
    conn.execute(f"create table if not exists {table}")

def get_row_count(conn, table, condition=""):
    """Get the number of rows for a table"""
    cur = conn.cursor()
    if condition == "":
        row = cur.execute(f"select count(*) from {table}").fetchone()
    else:
        row = cur.execute(f"select count(*) from {table} where {condition}").fetchone()
    return int(row[0])

def delete_rows(conn, table, condition=""):
    """Delete rows"""
    cur = conn.cursor()
    if condition == "":
        cur.execute(f"delete from {table}")
    else:
        cur.execute(f"delete from {table} where {condition}")

    conn.commit()

def drop_table(conn, table):
    """Delete rows"""
    cur = conn.cursor()
    cur.execute(f"drop table {table}")

def insert_text_rows(conn, table, count, suffix=""):
    """Insert rows"""
    for i in range(count):
        tmp = "odd"
        if i%2 == 0:
            tmp ="even"
        key = f"{tmp}-text-{i}{suffix}"
        conn.execute(f"insert into {table} values(?)", (key,))
    conn.commit()

def iter_rows_count(conn, table, suffix=""):
    """Iterate table and count"""
    count = 0
    cur = conn.cursor()
    for row in cur.execute(f"select * from {table}"):
        if row[0].endswith(suffix):
            count += 1
    return count

class MojoWritableTest(unittest.TestCase):
    '''MojoWritableTest'''

    def __init__(self, cfg, *args, **kargs):
        self.cfg = cfg
        self.db_conn = None
        self.db_path = "a.db"
        super(MojoWritableTest, self).__init__(*args, **kargs)

    def get_count(self, conn, table_name):
        '''Get the number of rows for a table'''
        cur = conn.cursor()
        row = cur.execute(f"select count(*) from {table_name}").fetchone()
        return int(row[0])

    def _open_db(self, db_path, ver="1", mode=""):
        if mode == "":
            conn_str = f"file:{db_path}?vfs=mojo&ver={ver}&pagesz={self.cfg.page_sz}&pps=65536"
        else:
            conn_str = f"file:{db_path}?vfs=mojo&mode=ro&ver={ver}&pagesz={self.cfg.page_sz}&pps=65536"

        conn = sqlite3.Connection(conn_str, uri=True)

        if mode != "ro":
            conn.execute(f"PRAGMA page_size={self.cfg.page_sz}")
            conn.execute(f"PRAGMA journal_mode={self.cfg.journal_mode}")
            conn.execute(f"PRAGMA auto_vacuum={self.cfg.vac_mode}")
        return conn

    def setUp(self):
        rm_fr(self.db_path)
    
    def tearDown(self):
        if self.db_conn:
            self.db_conn.close()

    def _subtest_name(self, name):
        return f"{name}: {self.cfg}"

    def test_db_use(self):
        '''Tests the general usage of the database '''

        row_count = 100
        table = "test"
        table_desc = "test (s text primary key)"
        self.db_conn = self._open_db(self.db_path)

        with self.subTest(self._subtest_name("create table and insert")):
            create_table(self.db_conn, table_desc)
            insert_text_rows(self.db_conn, table, row_count, "-1")
            self.assertEqual(row_count, get_row_count(self.db_conn, table))

        with self.subTest(self._subtest_name("select")):
            count = iter_rows_count(self.db_conn, table, "-1")
            self.assertEqual(count, row_count, 'select row count')
            self.assertEqual(row_count, get_row_count(self.db_conn, table, "s like '%-1'"))

        with self.subTest(self._subtest_name("delete")):
            delete_rows(self.db_conn, table, "s like 'even%'")
            self.assertEqual(row_count//2, get_row_count(self.db_conn, table))

        with self.subTest(self._subtest_name("drop and create")):
            drop_table(self.db_conn, table)
            create_table(self.db_conn, table_desc)
            insert_text_rows(self.db_conn, table, row_count, "-1")
            self.assertEqual(row_count, get_row_count(self.db_conn, table))

        with self.subTest(self._subtest_name("vacuum")):
            self.db_conn.execute("vacuum")

        with self.subTest(self._subtest_name("commit version v=1")):
            commit_version(self.db_path)
            self.db_conn.close()
            self.db_conn = None

        with self.subTest(self._subtest_name("open ver=2")):
            self.db_conn = self._open_db(self.db_path)

        with self.subTest(self._subtest_name("delete ver=2")):
            delete_rows(self.db_conn, table)
            count = iter_rows_count(self.db_conn, table, "-1")
            self.assertEqual(0, count, 'select row count')
            self.assertEqual(0, get_row_count(self.db_conn, table))

        with self.subTest(self._subtest_name("db ro ver=1")):
            conn_v1 = self._open_db(self.db_path, ver="1", mode="ro")
            count = iter_rows_count(conn_v1, table, "-1")
            self.assertEqual(count, row_count, 'select row count')
            self.assertEqual(row_count, get_row_count(conn_v1, table))

        with self.subTest(self._subtest_name("commit version v=2")):
            commit_version(self.db_path)
            self.db_conn.close()
            self.db_conn = None
        
        with self.subTest(self._subtest_name("open ver=3")):
            self.db_conn = self._open_db(self.db_path)
            conn_v2 = self._open_db(self.db_path, ver="2", mode="ro")

            count = iter_rows_count(conn_v2, table, "-1")
            self.assertEqual(0, count, 'select row count')
            self.assertEqual(0, get_row_count(conn_v2, table))

            count = iter_rows_count(conn_v1, table)
            self.assertEqual(count, row_count, 'select row count')
            self.assertEqual(row_count, get_row_count(conn_v1, table))


def load_extension(mojo_lib):
    """load_extension"""
    print("using libpath =", mojo_lib)

    con = sqlite3.connect(":memory:")
    # enable extension loading
    con.enable_load_extension(True)
    con.execute(f"select load_extension('{mojo_lib}')")
    con.execute("pragma page_size=4096")
    con.enable_load_extension(False)
    con.close()

def create_suite(full_mode):
    ''' Test suite for mojo '''

    suite = unittest.TestSuite()

    if not full_mode:
        page_sizes = [4096]
        journal_modes = ["WAL"]
        vacuum_modes = ["INCREMENTAL"]
    else:
        page_sizes = [4096]
        journal_modes = ["OFF", "WAL", "MEMORY", "DELETE", "TRUNCATE", "PERSIST"]
        vacuum_modes = ["NONE", "FULL", "INCREMENTAL"]

    for page_sz in page_sizes:
        for journal_mode in journal_modes:
            for vac_mode in vacuum_modes:
                cfg = TestConfig(page_sz=page_sz,
                    journal_mode=journal_mode,
                    vac_mode=vac_mode)
                    
                suite.addTest(MojoWritableTest(cfg, 'test_db_use'))

    return suite

if __name__ == '__main__':
    if len(sys.argv[1:]) < 1:
        print("Error: missing extension library path")
        sys.exit(1)

    if len(sys.argv[2:]) >= 1 and sys.argv[2] == "full":
        FULL = True
    else:
        FULL = False

    MOJOKV_CLI=os.getenv("MOJOKV_CLI")
    if not MOJOKV_CLI:
        MOJOKV_CLI="./build/mojo-cli"

    ext_path = sys.argv[1]
    load_extension(ext_path)

    runner = unittest.TextTestRunner()
    runner.run(create_suite(FULL))


#conn = sqlite3.connect("file:a.db?vfs=mojo&ver=1&pagesz=4096", uri=True)
#conn = sqlite3.connect("a.db")
