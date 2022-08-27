#!/usr/bin/env bash

export BUILD_PROFILE=debug
export CARGO_OPT=""
export MESON_BUILD_PROFILE="debug"
export MESON_BUILD_DIR="build"

if [ $# -gt 0 ]; then
    if [ $1 = "release" ]; then
        export BUILD_PROFILE=release
        export CARGO_OPT="--release"
        export MESON_BUILD_PROFILE="release"
    fi
fi

echo "BUILD_PROFILE=$BUILD_PROFILE"


echo "-------------------------------"
echo "  [Rust build starting]"
echo "-------------------------------"

cargo build $CARGO_OPT
if [ $? -ne 0 ]; then
    echo "Error: Cargo build failed with $?"
    exit $?
fi

mkdir -p $MESON_BUILD_DIR

rm -fR $MESON_BUILD_DIR/*

cp target/$BUILD_PROFILE/mojo-cli $MESON_BUILD_DIR/.
if [ $? -ne 0 ]; then
    echo "Copying failed"
    exit $?
fi

echo "-------------------------------"
echo "  [C build starting]"
echo "-------------------------------"

meson setup --buildtype=$MESON_BUILD_PROFILE $MESON_BUILD_DIR
if [ $? -ne 0 ]; then
    echo "Error: meson reconfigure failed with $?"
    exit $?
fi

meson compile -C $MESON_BUILD_DIR
if [ $? -ne 0 ]; then
    echo "Error: meson compile failed with $?"
    exit $?
fi

echo ">> The build artifacts are at the dir: $MESON_BUILD_DIR"

if [ "$MOJO_TEST" != "" ]; then
    echo "-------------------------------"
    echo "  [Mojo test starting]"
    echo "-------------------------------"

    export MOJOKV_CLI=./target/$BUILD_PROFILE/mojo-cli

    python3 testdb.py $MESON_BUILD_DIR/libmojo
    if [ $? -ne 0 ]; then
        echo "Error: mojo test failed with $?"
        exit 1
    fi
fi