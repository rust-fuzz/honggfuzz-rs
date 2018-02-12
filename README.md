# honggfuzz-rs [![Build Status][travis-img]][travis] [![Crates.io][crates-img]][crates] [![Documentation][docs-img]][docs]

[travis-img]:   https://travis-ci.org/PaulGrandperrin/honggfuzz-rs.svg?branch=master
[travis]:       https://travis-ci.org/PaulGrandperrin/honggfuzz-rs
[crates-img]:   https://img.shields.io/crates/v/honggfuzz.svg
[crates]:       https://crates.io/crates/honggfuzz
[docs-img]:     https://docs.rs/honggfuzz/badge.svg
[docs]:         https://docs.rs/honggfuzz

Fuzz your Rust code with Honggfuzz ! 

## [Documentation](https://docs.rs/honggfuzz)

[![asciicast](https://asciinema.org/a/162128.png)](https://asciinema.org/a/162128)

## About Honggfuzz

Honggfuzz is a security oriented fuzzer with powerful analysis options. Supports evolutionary, feedback-driven fuzzing based on code coverage (software- and hardware-based)

* project homepage [honggfuzz.com](http://honggfuzz.com/)
* project repository [github.com/google/honggfuzz](https://github.com/google/honggfuzz)
* this upstream project is maintained by Google, but ...
* this is NOT an official Google product

### Description (from upstream project)
* It's __multi-process__ and __multi-threaded__: no need to run multiple copies of your fuzzer, as honggfuzz can unlock potential of all your available CPU cores with one process. The file corpus is automatically shared and improved between the fuzzing threads.
* It's blazingly fast when in the [persistent fuzzing mode](https://github.com/google/honggfuzz/blob/master/docs/PersistentFuzzing.md)). A simple/empty _LLVMFuzzerTestOneInput_ function can be tested with __up to 1mo iterations per second__ on a relatively modern CPU (e.g. i7-6700K)
* Has a [solid track record](#trophies) of uncovered security bugs: the __only__ (to the date) __vulnerability in OpenSSL with the [critical](https://www.openssl.org/news/secadv/20160926.txt) score mark__ was discovered by honggfuzz. See the [Trophies](#trophies) paragraph for the summary of findings to the date
* Uses low-level interfaces to monitor processes (e.g. _ptrace_ under Linux). As opposed to other fuzzers, it __will discover and report hijacked/ignored signals__ (intercepted and potentially hidden by signal handlers)
* Easy-to-use, feed it a simple corpus directory (can even be empty) and it will work its way up expanding it utilizing feedback-based coverage metrics
* Supports several (more than any other coverage-based feedback-driven fuzzer) hardware-based (CPU: branch/instruction counting, __Intel BTS__, __Intel PT__) and software-based [feedback-driven fuzzing](https://github.com/google/honggfuzz/blob/master/docs/FeedbackDrivenFuzzing.md) methods known from other fuzzers (libfuzzer, afl)
* Works (at least) under GNU/Linux, FreeBSD, Mac OS X, Windows/CygWin and [Android](https://github.com/google/honggfuzz/blob/master/docs/Android.md)
* Supports the __persistent fuzzing mode__ (long-lived process calling a fuzzed API repeatedly) with libhfuzz/libhfuzz.a. More on that can be found [here](https://github.com/google/honggfuzz/blob/master/docs/PersistentFuzzing.md)
* [Can fuzz remote/standalone long-lasting processes](https://github.com/google/honggfuzz/blob/master/docs/AttachingToPid.md) (e.g. network servers like __Apache's httpd__ and __ISC's bind__), though the [persistent fuzzing mode](https://github.com/google/honggfuzz/blob/master/docs/PersistentFuzzing.md) is suggested instead: as it's faster and multiple instances of a service can be fuzzed with this
* It comes with the __[examples](https://github.com/google/honggfuzz/tree/master/examples) directory__, consisting of real world fuzz setups for widely-used software (e.g. Apache and OpenSSL)

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

```rust
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
            if data.len() != 10 {return}
            if data[0] != b'q' {return}
            if data[1] != b'w' {return}
            if data[2] != b'e' {return}
            if data[3] != b'r' {return}
            if data[4] != b't' {return}
            if data[5] != b'y' {return}
            if data[6] != b'u' {return}
            if data[7] != b'i' {return}
            if data[8] != b'o' {return}
            if data[9] != b'p' {return}
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
# builds the target in debug mode and replays automatically the crash in gdb
cargo hfuzz run-debug example fuzzing_workspace/*.fuzz
```

Clean

```sh
# a wrapper on "cargo clean" which cleans the fuzzing_target directory
cargo hfuzz clean 
```

Optionally, fuzz with LLVM's [sanitizers](https://github.com/japaric/rust-san)

```sh
RUSTFLAGS="-Z sanitizer=address" cargo hfuzz run example
```

## Relevant documentation about honggfuzz usage
* [USAGE](https://github.com/google/honggfuzz/blob/master/docs/USAGE.md)
* [FeedbackDrivenFuzzing](https://github.com/google/honggfuzz/blob/master/docs/FeedbackDrivenFuzzing.md)
* [PersistentFuzzing](https://github.com/google/honggfuzz/blob/master/docs/PersistentFuzzing.md)

## About Rust fuzzing
 
There is other projects providing Rust fuzzing support at [github.com/rust-fuzz](https://github.com/rust-fuzz). 
 
You'll find support for [AFL](https://github.com/rust-fuzz/afl.rs) and LLVM's [LibFuzzer](https://github.com/rust-fuzz/cargo-fuzz) and there is also a [trophy case](https://github.com/rust-fuzz/trophy-case) ;-) .

This crate was inspired by those projects!
