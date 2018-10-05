#!/bin/sh -ve
export RUST_BACKTRACE=full

git submodule update --init

cargo uninstall honggfuzz 2>/dev/null || true
cargo clean
cargo update

# install cargo subcommands
cargo install --path . --force --verbose
cargo hfuzz version

cd example

# run test.sh without sanitizers
RUSTFLAGS="" ./test.sh

# run test.sh with sanitizers only on nightly
version=`rustc --version`
if [ -z "${version##*nightly*}" ] ;then
	RUSTFLAGS="-Z sanitizer=address" ./test.sh
	RUSTFLAGS="-Z sanitizer=thread" ./test.sh
	if [ "`uname`" = "Linux" ] ;then # the leak sanitizer is only available on Linux
		RUSTFLAGS="-Z sanitizer=leak" ./test.sh
	fi
	# RUSTFLAGS="-Z sanitizer=memory" ./test.sh # not working, see: https://github.com/rust-lang/rust/issues/39610
fi

# go back to root crate
cd ..

# try to generate doc
cargo doc

# run unit tests
cargo test

cargo clean
