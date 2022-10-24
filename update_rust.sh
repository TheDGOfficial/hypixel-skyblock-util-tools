#!/bin/bash
rustup self update
rustup update

cargo +nightly update
cargo +nightly generate-lockfile

cargo install-update -a

