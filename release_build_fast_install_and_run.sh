#!/bin/bash
export SKIP_TESTS=true
export SKIP_RUST_UPDATES=true
export SKIP_RUST_INSTALLS=true

get_executable_location() {
    echo "./target/x86_64-unknown-linux-gnu/$1/hypixel-skyblock-util-tools"
}

EXECUTABLE_LOCATION="$(get_executable_location release)"
BUILD_AND_INSTALL_CMD=". ./release_build.sh --no-pgo && $EXECUTABLE_LOCATION install-minecraft-launcher-launcher"

if [[ "$1" == "--profile" ]]; then
 sudo sh -c 'echo 1 > /proc/sys/kernel/perf_event_paranoid'
 sudo sh -c 'echo 0 > /proc/sys/kernel/kptr_restrict'

 EXECUTABLE_LOCATION="$(get_executable_location profiling)"

 PROFILING_PROFILE=true $BUILD_AND_INSTALL_CMD && perf record --freq=1000 --call-graph dwarf -q -o perf.data "$EXECUTABLE_LOCATION"

 perf report

 [ ! -e perf.data ] || rm perf.data
 [ ! -e perf.data.old ] || rm perf.data.old
else
 $BUILD_AND_INSTALL_CMD && "$EXECUTABLE_LOCATION"
fi

