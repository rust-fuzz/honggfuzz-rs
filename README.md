# honggfuzz-rs
Fuzz your Rust code with Honggfuzz !

[![asciicast](https://asciinema.org/a/rJ8P4e3enW6gOTseJ8w84OLYd.png)](https://asciinema.org/a/rJ8P4e3enW6gOTseJ8w84OLYd)

## About Honggfuzz
 - project homepage http://honggfuzz.com/
 - project repository https://github.com/google/honggfuzz
 
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
Install honggfuzz command to build with instrumentation and fuzz
```sh
cargo install honggfuzz # will install honggfuzz and honggfuzz-build subcommands in cargo
```
Add to your dependencies
```toml
[dependencies]
honggfuzz = "0.2"
```
Add code snippet to fuzz
```rust
#![no_main]
#[macro_use] extern crate honggfuzz;

fuzz_target!(|data: &[u8]| {
    if data.len() != 10 {return}
    if data[0] != 'q' as u8 {return}
    if data[1] != 'w' as u8 {return}
    if data[2] != 'e' as u8 {return}
    if data[3] != 'r' as u8 {return}
    if data[4] != 't' as u8 {return}
    if data[5] != 'y' as u8 {return}
    if data[6] != 'u' as u8 {return}
    if data[7] != 'i' as u8 {return}
    if data[8] != 'o' as u8 {return}
    if data[9] != 'p' as u8 {return}
    panic!("BOOM")
});
```
Build with instrumentation
```sh
cargo honggfuzz-build # a wrapper on "cargo build" with fuzzing instrumentation enabled. produces binaries in "fuzzing_target" directory
```

Fuzz
```sh
mkdir in
cargo honggfuzz -f in -P -- fuzzing_target/x86_64-unknown-linux-gnu/debug/fuzzme # a wrapper on honggfuzz executable with settings adapted to work with Rust code
```
