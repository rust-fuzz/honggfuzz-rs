# honggfuzz-rs [![Build Status][travis-img]][travis] [![Crates.io][crates-img]][crates] [![Documentation][docs-img]][docs]

[travis-img]:   https://travis-ci.org/rust-fuzz/honggfuzz-rs.svg?branch=master
[travis]:       https://travis-ci.org/rust-fuzz/honggfuzz-rs
[crates-img]:   https://img.shields.io/crates/v/honggfuzz.svg
[crates]:       https://crates.io/crates/honggfuzz
[docs-img]:     https://docs.rs/honggfuzz/badge.svg
[docs]:         https://docs.rs/honggfuzz

Fuzz your Rust code with Google-developped Honggfuzz !

## [Documentation](https://docs.rs/honggfuzz)

[![asciicast](https://asciinema.org/a/43MLo5Xl8ukHxgwDLArKqS9xc.png)](https://asciinema.org/a/43MLo5Xl8ukHxgwDLArKqS9xc)

## About Honggfuzz

Honggfuzz is a security oriented fuzzer with powerful analysis options. Supports evolutionary, feedback-driven fuzzing based on code coverage (software- and hardware-based).

* project homepage [honggfuzz.com](http://honggfuzz.com/)
* project repository [github.com/google/honggfuzz](https://github.com/google/honggfuzz)
* this upstream project is maintained by Google, but ...
* this is NOT an official Google product

## Compatibility

* __Rust__: stable, beta, nightly
* __OS__: GNU/Linux, macOS, FreeBSD, Android, WSL (Windows Subsystem for Linux)
* __Arch__: x86_64, x86, arm64-v8a, armeabi-v7a, armeabi
* __Sanitizer__: none, address, thread, leak 

## How to use this crate

Install honggfuzz commands to build with instrumentation and fuzz

```sh
# installs hfuzz and honggfuzz subcommands in cargo
cargo install honggfuzz
```

Add to your dependencies

```toml
[dependencies]
honggfuzz = "0.5"
```

Create a target to fuzz

```rust,should_panic
#[macro_use] extern crate honggfuzz;

fn main() {
    // Here you can parse `std::env::args and 
    // setup / initialize your project

    // You have full control over the loop but
    // you're supposed to call `fuzz` ad vitam aeternam
    loop {
        // The fuzz macro gives an arbitrary object (see `arbitrary crate`)
        // to a closure-like block of code.
        // For performance reasons, it is recommended that you use the native type
        // `&[u8]` when possible.
        // Here, this slice will contain a "random" quantity of "random" data.
        fuzz!(|data: &[u8]| {
            if data.len() != 6 {return}
            if data[0] != b'q' {return}
            if data[1] != b'w' {return}
            if data[2] != b'e' {return}
            if data[3] != b'r' {return}
            if data[4] != b't' {return}
            if data[5] != b'y' {return}
            panic!("BOOM")
        });
    }
}

```

Fuzz for fun and profit !

```sh
# builds with fuzzing instrumentation and then runs the "example" target
cargo hfuzz run example
```

Once you got a crash, replay it easily in a debug environment

```sh
# builds the target in debug mode and replays automatically the crash in rust-lldb
cargo hfuzz run-debug example fuzzing_workspace/*.fuzz
```

Clean

```sh
# a wrapper on "cargo clean" which cleans the fuzzing_target directory
cargo hfuzz clean 
```

Version

```sh
cargo hfuzz version
```

### Environment variables

#### `RUSTFLAGS`

You can use `RUSTFLAGS` to send additional arguments to `rustc`.

For instance, you can enable the use of LLVM's [sanitizers](https://github.com/japaric/rust-san).
This is a recommended option if you want to test your `unsafe` rust code but it will have an impact on performance.

```sh
RUSTFLAGS="-Z sanitizer=address" cargo hfuzz run example
```

#### `HFUZZ_BUILD_ARGS`

You can use `HFUZZ_BUILD_ARGS` to send additional arguments to `cargo build`.

#### `HFUZZ_RUN_ARGS`

You can use `HFUZZ_RUN_ARGS` to send additional arguments to `honggfuzz`.
See [USAGE](https://github.com/google/honggfuzz/blob/master/docs/USAGE.md) for the list of those.

For example:

```sh
# 1 second of timeout
# use 12 fuzzing thread
# be verbose
# stop after 1000000 fuzzing iteration
# exit upon crash
HFUZZ_RUN_ARGS="-t 1 -n 12 -v -N 1000000 --exit_upon_crash" cargo hfuzz run example
```

#### `HFUZZ_DEBUGGER`

By default we use `rust-lldb` but you can change it to `rust-gdb`, `gdb`, `/usr/bin/lldb-7` ...

#### `CARGO_TARGET_DIR`

Target compilation directory, defaults to `hfuzz_target` to not clash with `cargo build`'s default `target` directory.

#### `HFUZZ_WORKSPACE`

Honggfuzz working directory, defaults to `hfuzz_workspace`.

#### `HFUZZ_INPUT`

Honggfuzz input files (also called "corpus"), defaults to `$HFUZZ_WORKSPACE/{TARGET}/input`.

## Conditionnal compilation

Sometimes, it is necessary to make some specific adaptation to your code to yield a better fuzzing efficiency.

For instance:
- Make you software behavior as much as possible deterministic on the fuzzing input
  - [PRNG](https://en.wikipedia.org/wiki/Pseudorandom_number_generator)s must be seeded with a constant or the fuzzer input
  - Behavior shouldn't change based on the computer's clock.
  - Avoid potential undeterministic behavior from racing threads.
  - ...
- Never ever call `std::process::exit()`.
- Disable logging and other unnecessary functionnalities.
- Try to avoid modifying global state when possible.


When building with `cargo hfuzz`, the argument `--cfg fuzzing` is passed to `rustc` to allow you to condition the compilation of thoses adaptations thanks to the `cfg` macro like so:

```rust
#[cfg(fuzzing)]
let mut rng = rand::StdRng::from_seed(&[0]);
#[cfg(not(fuzzing))]
let mut rng = rand::thread_rng();
```

Also, when building in debug mode, the `fuzzing_debug` argument is added in addition to `fuzzing`.

For more information about conditional compilation, please see the [reference](https://doc.rust-lang.org/reference/attributes.html#conditional-compilation).

## Relevant documentation about honggfuzz
* [USAGE](https://github.com/google/honggfuzz/blob/master/docs/USAGE.md)
* [FeedbackDrivenFuzzing](https://github.com/google/honggfuzz/blob/master/docs/FeedbackDrivenFuzzing.md)
* [PersistentFuzzing](https://github.com/google/honggfuzz/blob/master/docs/PersistentFuzzing.md)

## About Rust fuzzing
 
There is other projects providing Rust fuzzing support at [github.com/rust-fuzz](https://github.com/rust-fuzz). 
 
You'll find support for [AFL](https://github.com/rust-fuzz/afl.rs) and LLVM's [LibFuzzer](https://github.com/rust-fuzz/cargo-fuzz) and there is also a [trophy case](https://github.com/rust-fuzz/trophy-case) ;-) .

This crate was inspired by those projects!
