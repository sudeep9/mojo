project('cmojo', 'c')

if get_option('buildtype') == 'release'
    mojo_lib_dir = meson.source_root() + '/../target/release'
else
    mojo_lib_dir = meson.source_root() + '/../target/debug'
endif

message(mojo_lib_dir)

mojo_lib = meson.get_compiler('c').find_library('mojofs',
    static: true,
    dirs: [mojo_lib_dir])

shared_library('cmojo',
    sources:['cmojo.c'],
    dependencies: [mojo_lib])
