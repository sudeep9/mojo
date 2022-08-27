#ifndef __mojo_h
#define __mojo_h

#include <sqlite3.h>

typedef struct MojoFile {
    sqlite3_vfs base;
    void* custom_file;
} MojoFile;

sqlite3_vfs* mojo_create();

void mojofs_init_log();
#endif