#!/bin/bash
./shellcheck.sh
./format.sh
export RUSTFLAGS='--cfg reqwest_unstable'
./test.sh
cargo clippy $CLIPPY_ARGS
