#!/bin/bash
./update_rust.sh
./format.sh
cargo +nightly --color always clippy -- -W clippy::all -W clippy::style -W clippy::pedantic -W clippy::nursery -W clippy::perf -W clippy::suspicious -W clippy::cargo -W clippy::restriction -W clippy::exit -W clippy::dbg_macro -W clippy::unwrap_used -W clippy::complexity -W clippy::create_dir -W clippy::correctness -W clippy::expect_used -W clippy::too-many-lines -W clippy::must-use-candidate -W clippy::multiple-crate-versions -A clippy::print-stdout -A clippy::use-debug -A clippy::missing-docs-in-private-items -A clippy::implicit-return  -A clippy::default-numeric-fallback -A clippy::float-arithmetic -A clippy::integer-arithmetic -A clippy::arithmetic-side-effects -A clippy::integer-division -A clippy::get-unwrap && cargo +nightly --color always run
#MIRAI_FLAGS=--diag=paranoid cargo +nightly --color always mirai

