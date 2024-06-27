#include <stdio.h>

#include <sqlite3ext.h>
SQLITE_EXTENSION_INIT1

#include <assert.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/file.h>
#include <sys/param.h>
#include <unistd.h>
#include <time.h>
#include <errno.h>
#include <fcntl.h>
#include <mojofs.h>

/*
** The maximum pathname length supported by this VFS.
*/
#define MAXPATHNAME 512

/*
** Argument zPath points to a nul-terminated string containing a file path.
** If zPath is an absolute path, then it is copied as is into the output 
** buffer. Otherwise, if it is a relative path, then the equivalent full
** path is written to the output buffer.
**
** This function assumes that paths are UNIX style. Specifically, that:
**
**   1. Path components are separated by a '/'. and 
**   2. Full paths begin with a '/' character.
*/
static int mojo_fullpath_name(
  sqlite3_vfs *pVfs,              /* VFS */
  const char *zPath,              /* Input path (possibly a relative path) */
  int nPathOut,                   /* Size of output buffer in bytes */
  char *zPathOut                  /* Pointer to output buffer */
){
  char zDir[MAXPATHNAME+1];
  if( zPath[0]=='/' ){
    zDir[0] = '\0';
  }else{
    if( getcwd(zDir, sizeof(zDir))==0 ) return SQLITE_IOERR;
  }
  zDir[MAXPATHNAME] = '\0';

  sqlite3_snprintf(nPathOut, zPathOut, "%s/%s", zDir, zPath);
  zPathOut[nPathOut-1] = '\0';

  return SQLITE_OK;
}

int sqlite3_mojo_init(sqlite3 *db, char **pzErrMsg, const sqlite3_api_routines *pApi){
  int rc = SQLITE_OK;
  SQLITE_EXTENSION_INIT2(pApi);

  mojofs_init_log();

  sqlite3_vfs *vfs = mojo_create();
  vfs->xFullPathname = mojo_fullpath_name;
  rc = sqlite3_vfs_register(vfs, 0);
  //printf("extension mojo called with rc=%d\n", rc);
  fflush(stdout);
  return rc;
}