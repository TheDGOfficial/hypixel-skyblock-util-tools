#!/bin/bash
get_cargo_cmd() {
  if [ "$1" == "test" ]; then
    echo "cargo $1 --target=$2"
  else
    echo "cargo $1 -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target=$2 --release"
  fi
}

export ARCH="x86_64"

if [ "$1" == "--i686" ] || [ "$2" == "--i686" ] || [ "$3" == "--i686" ] || [ "$4" == "--i686" ]; then
  ARCH="i686"
elif [ "$1" == "--aarch64" ] || [ "$2" == "--aarch64" ] || [ "$3" == "--aarch64" ] || [ "$4" == "--aarch64" ]; then
  ARCH="aarch64"
fi

export TARGET="$ARCH-unknown-linux-gnu"

if [ "$1" == "--windows" ] || [ "$2" == "--windows" ] || [ "$3" == "--windows" ] || [ "$4" == "--windows" ]; then
  TARGET="$ARCH-pc-windows-gnu"
elif [ "$1" == "--macos" ] || [ "$2" == "--macos" ] || [ "$3" == "--macos" ] || [ "$4" == "--macos" ]; then
  TARGET="$ARCH-apple-darwin"
fi

echo "Target is $TARGET"

if [[ -z "$SKIP_RUST_INSTALLS" ]]; then
 rustup target add $TARGET
fi

. ./update_rust.sh

rm -rf /tmp/pgo-data

export TARGET_CPU="native"
export TUNE_CPU="native"

if [ "$1" == "--portable" ] || [ "$2" == "--portable" ] || [ "$3" == "--portable" ] || [ "$4" == "--portable" ]; then
  TARGET_CPU="x86-64-v2"
  TUNE_CPU="generic"
fi

echo "Target CPU is $TARGET_CPU. Tuning for $TUNE_CPU CPUs."

export PGO_FLAG=" -Cprofile-generate=/tmp/pgo-data"

if [ "$1" == "--no-pgo" ] || [ "$2" == "--no-pgo" ] || [ "$3" == "--no-pgo" ] || [ "$4" == "--no-pgo" ]; then
  echo "Will not do Profile Guided Optimization (PGO) as requested by the --no-pgo flag."

  PGO_FLAG=""
else
  # TODO remove the following lines and only print the enabled message when its fixed
  echo "Profile Guided Optimization (PGO) is disabled at the moment because of an incompatibility. See https://github.com/rust-lang/wg-cargo-std-aware/issues/68"

  PGO_FLAG=""

  #echo "Profile Guided Optmization (PGO) will be done using default inputs."
fi

LLVM_VERSION="15"
LLVM_PROFDATA_CMD="llvm-profdata-$LLVM_VERSION"

if ! command -v $LLVM_PROFDATA_CMD &>/dev/null; then
  echo "$LLVM_PROFDATA_CMD not available, PGO will not be used."
  PGO_FLAG=""
fi

export NORMAL_FLAGS="--remap-path-prefix=/home/$USER/.cargo/=/.cargo/ -C opt-level=3 -C target-cpu=$TARGET_CPU -Z tune-cpu=$TUNE_CPU -C lto -Z mir-opt-level=4"

CARGO_CMD="$(get_cargo_cmd build $TARGET)"
CARGO_CMD_TEST="$(get_cargo_cmd test $TARGET)"

export CARGO_CMD
export CARGO_CMD_TEST

export CARGO_CMD_FULL=$CARGO_CMD
export CARGO_CMD_FULL_TEST=$CARGO_CMD_TEST

if [[ -z "$SKIP_TESTS" ]]; then
 . ./test.sh
fi

export RUSTFLAGS="$NORMAL_FLAGS$PGO_FLAG"

$CARGO_CMD_FULL

if [ "$PGO_FLAG" != "" ]; then
  echo -e "1\n1\n7\n" | ./target/$TARGET/release/hypixel-skyblock-util-tools
  echo -e "2\n0\n0\n0\n50\n5\n5\n" | ./target/$TARGET/release/hypixel-skyblock-util-tools
  echo -e "3\n1\n0\n5\n99999\n" | ./target/$TARGET/release/hypixel-skyblock-util-tools
  echo -e "4\n1\n9\n15000\n5000\n4000\n" | ./target/$TARGET/release/hypixel-skyblock-util-tools
  echo -e "5\n1\n" | ./target/$TARGET/release/hypixel-skyblock-util-tools

  $LLVM_PROFDATA_CMD merge -o /tmp/pgo-data/merged.profdata /tmp/pgo-data

  RUSTFLAGS="$NORMAL_FLAGS -Cprofile-use=/tmp/pgo-data/merged.profdata -Cllvm-args=-pgo-warn-missing-function" $CARGO_CMD
fi

strip target/$TARGET/release/hypixel-skyblock-util-tools
#upx --best target/$TARGET/release/hypixel-skyblock-util-tools
