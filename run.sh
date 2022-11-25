#!/bin/bash
if [ "$1" == "--offline" ] || [ "$2" == "--offline" ]; then
  echo "Running in offline mode"
else
  . ./update_rust.sh
fi

. ./format.sh

# Disabled, slower than just building on disk because of copy overhead AND the fact that it slows down other programs and whole
# computer if theres not enough RAM freed, and if not, the Linux kernel has its own cache anyways, so it doesnt provide that great benefits.

# Plus theres a problem where cargo recompiles all crates everytime ignoring target/ even if its contents can be used to cache, no idea how to fix. This doesn't happen if we disable ramdisk. This makes it painfully slow because cargo clippy re-compiles everything, AND then cargo run recompiles everything again..
#./ramdisk_create.sh

MOLD_WRAPPER_CMD="mold -run"

if ! command -v mold &>/dev/null; then
  echo "Mold is not available, using (slower) default linker"
  MOLD_WRAPPER_CMD=""
fi

if [ "$1" == "--skip-extra-analyzers" ] || [ "$2" == "--skip-extra-analyzers" ]; then
  $MOLD_WRAPPER_CMD cargo run
else
  . ./init_clippy_args.sh
  $MOLD_WRAPPER_CMD cargo clippy "$CLIPPY_ARGS" && $MOLD_WRAPPER_CMD cargo run
  #MIRAI_FLAGS=--diag=paranoid cargo mirai
fi

# Disabled for same reason as above.
#. ./ramdisk_flush_and_unmount.sh
