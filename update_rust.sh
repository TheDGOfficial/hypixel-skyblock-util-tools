#!/bin/bash
if [[ -z "$SKIP_RUST_UPDATES" ]]; then
 git fetch && git pull

 rustup self update
 rustup update

 cargo update
 cargo generate-lockfile

 cargo install cargo-binstall

 cargo binstall -y cargo-binstall
 cargo binstall -y cross

 cargo binstall -y cargo-update
 cargo install-update --git --all

 podman image ls --format "{{.Repository}}:{{.Tag}}" | while read -r container ; do
  podman pull "$container"
 done
 podman image prune -f
fi

