#!/bin/bash
if [[ -z "$SKIP_RUST_UPDATES" ]]; then
 git fetch & git stash & git pull & git stash pop

 rustup self update
 rustup update

 cargo update
 cargo generate-lockfile

 cargo install cargo-binstall

 cargo binstall -y cargo-binstall
 cargo binstall -y cross

 cargo binstall -y --disable-strategies quick-install cargo-update
 cargo install-update --git --all

 podman image ls --format "{{.Repository}}:{{.Tag}}" | while read -r container ; do
  podman pull "$container"
 done
 podman image prune -f
fi

