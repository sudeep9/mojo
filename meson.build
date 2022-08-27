project('mojo', 'c', default_options: ['c_std=c11'], version: '0.1.0')

compiler = meson.get_compiler('c')

mojokv_lib_path = meson.source_root() + '/target/debug'
if get_option('buildtype') == 'release'
    mojokv_lib_path = meson.source_root() + '/target/release'
endif

mojofs_lib = compiler.find_library('mojofs',
    dirs: [mojokv_lib_path])

mojofs_inc_dir = include_directories('mojofs')
mojofs_dep = declare_dependency(dependencies: [mojofs_lib],
    include_directories: [mojofs_inc_dir])

shared_library('mojo',
    sources: ['mojo.c'],
    dependencies: [mojofs_dep])

