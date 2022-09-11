project('mojo', 'c', default_options: ['c_std=c11'], version: '0.1.0')

compiler = meson.get_compiler('c')

mojokv_lib_path = meson.source_root() + '/target/debug'
if get_option('buildtype') == 'release'
    mojokv_lib_path = meson.source_root() + '/target/release'
endif

mojofs_lib = compiler.find_library('mojofs',
    dirs: [mojokv_lib_path])

mojofs_dep = declare_dependency(dependencies: [mojofs_lib])

shared_library('mojo',
    sources: ['sqlite-ext/mojo.c'],
    include_directories: ['sqlite-ext'],
    dependencies: [mojofs_dep])

