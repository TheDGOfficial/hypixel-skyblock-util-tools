#!/bin/bash
./shellcheck.sh
./format.sh
./test.sh
cargo --cfg reqwest_unstable clippy $CLIPPY_ARGS
