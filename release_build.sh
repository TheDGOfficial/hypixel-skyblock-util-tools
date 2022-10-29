#!/bin/bash
./update_rust.sh
./test.sh
RUSTFLAGS="--remap-path-prefix=/home/$USER/.cargo/=/.cargo/ -C opt-level=3 -C target-cpu=native -Z tune-cpu=native -C lto -Z mir-opt-level=4" cargo build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target=x86_64-unknown-linux-gnu --release
strip target/x86_64-unknown-linux-gnu/release/hypixel-skyblock-util-tools
#upx --best target/x86_64-unknown-linux-gnu/release/hypixel-skyblock-util-tools

