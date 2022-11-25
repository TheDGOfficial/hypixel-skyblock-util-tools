#!/bin/bash
if [[ -z "$SKIP_RUST_UPDATES" ]]; then
 rustup self update
 rustup update

 cargo update
 cargo generate-lockfile

 cargo install cargo-update
 cargo install-update --git --all
fi

