#!/bin/bash
./shellcheck.sh
./format.sh
./test.sh
cargo clippy $CLIPPY_ARGS
