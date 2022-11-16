#!/bin/bash
if [ "$1" == "--offline" ] || [ "$2" == "--offline" ]; then
 echo "Running in offline mode"
else
 ./update_rust.sh
fi

./format.sh

# Disabled, slower than just building on disk because of copy overhead AND the fact that it slows down other programs and whole
# computer if theres not enough RAM freed, and if not, the Linux kernel has its own cache anyways, so it doesnt provide that great benefits.

# Plus theres a problem where cargo recompiles all crates everytime ignoring target/ even if its contents can be used to cache, no idea how to fix. This doesn't happen if we disable ramdisk. This makes it painfully slow because cargo clippy re-compiles everything, AND then cargo run recompiles everything again..
#./ramdisk_create.sh

MOLD_WRAPPER_CMD="mold -run"

if ! command -v mold &> /dev/null
then
    echo "Mold is not available, using (slower) default linker"
    MOLD_WRAPPER_CMD=""
fi

if [ "$1" == "--skip-extra-analyzers" ] || [ "$2" == "--skip-extra-analyzers" ]; then
 $MOLD_WRAPPER_CMD cargo run
else
 $MOLD_WRAPPER_CMD cargo clippy -- -W clippy::all -W clippy::style -W clippy::pedantic -W clippy::nursery -W clippy::perf -W clippy::suspicious -W clippy::cargo -W clippy::restriction -W clippy::exit -W clippy::dbg_macro -W clippy::unwrap_used -W clippy::complexity -W clippy::create_dir -W clippy::correctness -W clippy::expect_used -W clippy::too-many-lines -W clippy::must-use-candidate -W clippy::multiple-crate-versions -A clippy::print-stdout -A clippy::print-stderr -A clippy::use-debug -A clippy::missing-docs-in-private-items -A clippy::implicit-return -A clippy::default-numeric-fallback -A clippy::float-arithmetic -A clippy::integer-arithmetic -A clippy::arithmetic-side-effects -A clippy::integer-division -A clippy::get-unwrap -A clippy::redundant-pub-crate && $MOLD_WRAPPER_CMD cargo run
 #MIRAI_FLAGS=--diag=paranoid cargo mirai
fi

# Disabled for same reason as above.
#./ramdisk_flush_and_unmount.sh

