#!/bin/bash
if [[ -z "$SKIP_RUST_UPDATES" ]]; then
 git fetch
 git stash
 git pull
 git stash pop

 rustup self update
 rustup update

 cargo update
 cargo generate-lockfile

 cargo install cargo-binstall

 cargo binstall -y cargo-binstall
 cargo binstall -y cross

 cargo binstall -y --disable-strategies quick-install cargo-update
 cargo install-update --git --all
 
 curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin

 podman image ls --format "{{.Repository}}:{{.Tag}}" | while read -r container ; do
  podman pull "$container"
 done
 podman image prune -f
 
 cargo binstall -y cargo-sweep
 
 # Prevent target folder from growing to a gigantic size
 cargo sweep --toolchains nightly-x86_64-unknown-linux-gnu

 # Remove old versions of crates installed by cargo-install
 rm -f ~/.cargo/bin/*-v*
fi

