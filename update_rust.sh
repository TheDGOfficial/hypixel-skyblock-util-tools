#!/bin/bash
rustup self update
rustup update

cargo update
cargo generate-lockfile

cargo install install-update
cargo install-update -a

