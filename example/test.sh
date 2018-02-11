#!/bin/sh -ve
export RUST_BACKTRACE=full

cargo clean
cargo hfuzz clean

# build example with instrumentation
cargo hfuzz build --verbose

# clean and prepare workspace
rm -rf workspace
mkdir -p workspace/input

# fuzz exemple
cargo honggfuzz -W workspace -f workspace/input -P -v -N 1000000 --exit_upon_crash  -- fuzzing_target/x86_64-unknown-linux-gnu/release/example

# verify that the fuzzing process found the crash
test "$(cat workspace/*.fuzz)" = "qwertyuiop"

# clean
cargo hfuzz clean

# verify that the fuzzing-target has been cleaned
test ! -e fuzzing_target

# build example in debug mode
cargo hfuzz build-debug --verbose

# test that the debug executable exists
test -x fuzzing_target/x86_64-unknown-linux-gnu/debug/example

# clean
cargo hfuzz clean

# verify that the fuzzing-target has been cleaned
test ! -e fuzzing_target

# verify that no target directory has been created
test ! -e target

# verify that we can build the target without instrumentation
cargo build

# but when we run it, it should fail with a useful error message and status 17
set +e
cargo run
status=$?
set -e
test $status -eq 17

cargo clean

