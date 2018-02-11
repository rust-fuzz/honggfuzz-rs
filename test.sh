#!/bin/sh -ve
export RUST_BACKTRACE=full

cargo uninstall honggfuzz 2>/dev/null || true
cargo clean

# install cargo subcommands
cargo install --force --verbose

cd example

# run test.sh without sanitizers
RUSTFLAGS="" ./test.sh

# run test.sh with sanitizers only on nightly
version=`rustc --version`
if [ -z "${version##*nightly*}" ] ;then
	RUSTFLAGS="-Z sanitizer=address" ./test.sh
	RUSTFLAGS="-Z sanitizer=leak" ./test.sh
	# RUSTFLAGS="-Z sanitizer=memory" ./test.sh # not working because of some use of uninitialized value
	RUSTFLAGS="-Z sanitizer=thread" ./test.sh
fi

# go back to root crate
cd ..

# try to generate doc
cargo doc

cargo clean
