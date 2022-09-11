"""" Tests for mojo filesystem """

import os
import sys
import unittest
import sqlite3
import shutil
import commands as c

MOJOKV_CLI=None

class TestConfig:
    '''Config for test db'''

    def __init__(self, page_sz=4096, journal_mode="WAL", vac_mode="NONE", use_tx=True):
        self.page_sz = page_sz
        self.journal_mode = journal_mode
        self.vac_mode = vac_mode
        self.use_tx = use_tx

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

class MojoWritableTest(unittest.TestCase):
    '''MojoWritableTest'''

    def __init__(self, cfg, dbpath, *args, **kargs):
        self.cfg = cfg
        self.db_conn = None
        self.db_path = dbpath
        super(MojoWritableTest, self).__init__(*args, **kargs)

    def setUp(self):
        rm_fr(self.db_path)
    
    def tearDown(self):
        if self.db_conn:
            self.db_conn.close()

    def _subtest_name(self, name):
        return f"{name}: {self.cfg} {self.db_path}"

    def begin(self, cur):
        """begin"""
        if self.cfg.use_tx:
            cur.execute("begin")
    
    def commit(self, cur):
        """commit"""
        if self.cfg.use_tx:
            cur.execute("commit")

    def rollback(self, cur):
        """rollback"""
        if self.cfg.use_tx:
            cur.execute("rollback")

    def test_db_use(self):
        '''Tests the general usage of the database '''

        db_conn = c.opendb(self.cfg, self.db_path)

        with self.subTest(self._subtest_name("create table")):
            c.create_table_person(db_conn)

        with self.subTest(self._subtest_name("insert rows v1")):
            c.insert_table_person(db_conn, 100)
            db_conn.commit()

            count = c.table_person_count(db_conn)
            self.assertEqual(100, count)

        db_conn.close()


        ### Commit ver=1
        c.commit_version(self.db_path)
        db_conn = c.opendb(self.cfg, self.db_path)

        with self.subTest(self._subtest_name("insert rows v2")):
            self.assertEqual(100, c.table_person_count(db_conn))
            c.insert_table_person(db_conn, 100)
            db_conn.commit()

            self.assertEqual(200, c.table_person_count(db_conn))
            db_conn.close()

        ### Commit ver=2
        c.commit_version(self.db_path)
        db_conn = c.opendb(self.cfg, self.db_path)

        with self.subTest(self._subtest_name("copy table")):
            self.assertEqual(200, c.table_person_count(db_conn))

            c.copy_table_person(db_conn, "person_2")
            db_conn.commit()

            self.assertEqual(200, c.get_row_count(db_conn, "person_2"))

        with self.subTest(self._subtest_name("read v1")):
            db_v1 =  c.opendb(self.cfg, self.db_path, mode="ro", ver="1")
            self.assertEqual(100, c.table_person_count(db_v1))


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
        use_tx = [False]
    else:
        page_sizes = [4096]
        journal_modes = ["OFF", "WAL", "MEMORY", "DELETE", "TRUNCATE", "PERSIST"]
        vacuum_modes = ["NONE", "FULL", "INCREMENTAL"]
        use_tx = [False]


    dbid = 0
    for page_sz in page_sizes:
        for journal_mode in journal_modes:
            for vac_mode in vacuum_modes:
                for tx in use_tx:
                    cfg = TestConfig(page_sz=page_sz,
                        journal_mode=journal_mode,
                        vac_mode=vac_mode,
                        use_tx=tx)

                    dbid += 1
                    dbpath = f"./testdbs/a_{dbid}.db"
                    suite.addTest(MojoWritableTest(cfg, dbpath, 'test_db_use'))

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

    c.MOJOKV_CLI = MOJOKV_CLI

    ext_path = sys.argv[1]
    load_extension(ext_path)

    rm_fr("./testdbs/*")
    c.mkdir("./testdbs")

    runner = unittest.TextTestRunner(failfast=True)
    runner.run(create_suite(FULL))


#conn = sqlite3.connect("file:a.db?vfs=mojo&ver=1&pagesz=4096", uri=True)
#conn = sqlite3.connect("a.db")
