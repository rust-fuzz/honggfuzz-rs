#!/bin/sh -ve
export RUST_BACKTRACE=full

cargo clean
cargo hfuzz clean
cargo update

# build example with instrumentation
cargo hfuzz build --verbose

# clean and prepare hfuzz_workspace
workspace="hfuzz_workspace/example"
rm -rf $workspace
mkdir -p $workspace/input

# fuzz exemple
HFUZZ_RUN_ARGS="-v -N 1000000 --exit_upon_crash" cargo hfuzz run example

# verify that the fuzzing process found the crash
test "$(cat $workspace/*.fuzz)" = "qwertyuiop"

# clean
cargo hfuzz clean

# verify that the hfuzz_target has been cleaned
test ! -e hfuzz_target

# build example in debug mode
cargo hfuzz build-debug --verbose

# test that the debug executable exists
test -x hfuzz_target/x86_64-unknown-linux-gnu/debug/example

# clean
cargo hfuzz clean
rm -rf hfuzz_workspace

# verify that the hfuzz_target has been cleaned
test ! -e hfuzz_target

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

