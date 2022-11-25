#!/bin/bash

. ./init_clippy_args.sh

if [[ -z "$CARGO_CMD_FULL_TEST" ]]; then
  echo "Running tests with default Cargo command line"
  cargo test
else
  $CARGO_CMD_FULL_TEST
fi
