#!/bin/sh -ve
export RUST_BACKTRACE=full

cargo uninstall honggfuzz 2>/dev/null || true
cargo clean

# install cargo subcommands
cargo install --force --verbose

# run test.sh in example directory
cd example
./test.sh

# go back to root crate
cd ..

# try to generate doc
cargo doc

cargo clean
