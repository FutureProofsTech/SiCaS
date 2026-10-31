#!/bin/sh
# Run workspace library tests for 32-bit x86 Linux.
set -eu

CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=gcc \
RUSTFLAGS="-C link-arg=-m32" \
cargo test --workspace --all-targets --target i686-unknown-linux-gnu --locked
