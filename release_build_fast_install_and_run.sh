#!/bin/bash
export SKIP_TESTS=true
export SKIP_RUST_UPDATES=true
export SKIP_RUST_INSTALLS=true

EXECUTABLE_LOCATION="./target/x86_64-unknown-linux-gnu/release/hypixel-skyblock-util-tools"

. ./release_build.sh --no-pgo && "$EXECUTABLE_LOCATION" install-minecraft-launcher-launcher && "$EXECUTABLE_LOCATION"

