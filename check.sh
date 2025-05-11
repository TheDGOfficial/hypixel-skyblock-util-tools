#!/bin/bash
./shellcheck.sh
./format.sh
export RUSTFLAGS='--cfg reqwest_unstable'
./test.sh
# shellcheck disable=SC2086
cargo clippy --fix --allow-dirty $CLIPPY_ARGS
