#!/bin/sh -ve
export RUST_BACKTRACE=full

# install cargo subcommands
cargo install --force --verbose

cd example
cargo hfuzz-clean

# build example with instrumentation
cargo hfuzz-build --verbose

# fuzz exemple
mkdir -p in
cargo honggfuzz -f in -P -v -N 1000000 --exit_upon_crash  -- fuzzing_target/x86_64-unknown-linux-gnu/debug/example

# verify that the fuzzing process found the crash
test "$(cat *.fuzz)" = "qwertyuiop"

# clean
cargo hfuzz-clean

# verify that the fuzzing-target has been cleaned
test ! -e fuzzing_target
