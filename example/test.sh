#!/bin/sh -vex
export RUST_BACKTRACE=full

CARGO_HFUZZ=${CARGO_HFUZZ:-"cargo hfuzz"} # by default, use regular subcommand. Override to force using another version (for tests) 

cargo clean
cargo update

# run commands from this directory to check that they correctly find the root crate directory
[ -d subdirectory ] && rmdir subdirectory
mkdir subdirectory

cd subdirectory
$CARGO_HFUZZ clean
cd ..

# build example with instrumentation
cd subdirectory
$CARGO_HFUZZ build --verbose
cd ..

# clean and prepare hfuzz_workspace
workspace="hfuzz_workspace/example"
rm -rf $workspace
mkdir -p $workspace/input

# fuzz example
cd subdirectory
HFUZZ_RUN_ARGS="-v -N 10000000 --run_time 120 -F3 --exit_upon_crash" $CARGO_HFUZZ run example
cd ..

# build example without instrumentation
cd subdirectory
$CARGO_HFUZZ build-no-instr --verbose
cd ..

# get crash file path
crash_path="$(ls $workspace/*.fuzz | head -n1)"

# verify that the fuzzing process found the crash
test $(cat "$crash_path") = "hey"

# build example in debug mode (and without sanitizers)
cd subdirectory
RUSTFLAGS="" $CARGO_HFUZZ build-debug --verbose
cd ..

# try to launch the debug executable without the crash file, it should fail with error code 1
set +e
hfuzz_target/*/debug/example
status=$?
set -e
test $status -eq 1

# try to launch the debug executable with the crash file, it should fail with error code 101 (rust panic's error code)
set +e
CARGO_HONGGFUZZ_CRASH_FILENAME="$crash_path" hfuzz_target/*/debug/example
status=$?
set -e
test $status -eq 101

# try to launch the debug executable with the an incorrect crash file, it should fail with error code 2
set +e
CARGO_HONGGFUZZ_CRASH_FILENAME="test.sh" hfuzz_target/*/debug/example
status=$?
set -e
test $status -eq 2

# run `hfuzz clean` from a subdirectory just to check that hfuzz subcommands are run at the crate root
cd subdirectory
$CARGO_HFUZZ clean
cd ..

rm -rf hfuzz_workspace

# verify that the hfuzz_target has been cleaned
test ! -e hfuzz_target

# verify that no target directory has been created
test ! -e target

# verify that we can build the target without instrumentation
RUSTFLAGS="" cargo build

# but when we run it, it should fail with a useful error message and status 17
set +e
RUSTFLAGS="" cargo run
status=$?
set -e
test $status -eq 17

cargo clean

# this directory should be empty
rmdir subdirectory

