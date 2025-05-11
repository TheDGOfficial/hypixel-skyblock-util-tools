#!/bin/bash

. ./init_clippy_args.sh

EXIT_CODE=0

if [[ -z "$CARGO_CMD_FULL_TEST" ]]; then
  echo "Running tests with default Cargo command line"
  cargo nextest run || EXIT_CODE=1
else
  $CARGO_CMD_FULL_TEST || EXIT_CODE=1
fi

(return 0 2>/dev/null) && sourced=1 || sourced=0

if (( sourced )); then
    return "$EXIT_CODE"
else
    exit "$EXIT_CODE"
fi
