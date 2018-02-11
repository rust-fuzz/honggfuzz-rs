//! ## About Honggfuzz
//! 
//! Honggfuzz is a security oriented fuzzer with powerful analysis options. Supports evolutionary, feedback-driven fuzzing based on code coverage (software- and hardware-based)
//! 
//! * project homepage [honggfuzz.com](http://honggfuzz.com/)
//! * project repository [github.com/google/honggfuzz](https://github.com/google/honggfuzz)
//! * this upstream project is maintained by Google, but ...
//! * this is NOT an official Google product
//! 
//! ### Description (from upstream project)
//! * It's __multi-process__ and __multi-threaded__: no need to run multiple copies of your fuzzer, as honggfuzz can unlock potential of all your available CPU cores with one process. The file corpus is automatically shared and improved between the fuzzing threads.
//! * It's blazingly fast when in the [persistent fuzzing mode](https://github.com/google/honggfuzz/blob/master/docs/PersistentFuzzing.md)). A simple/empty _LLVMFuzzerTestOneInput_ function can be tested with __up to 1mo iterations per second__ on a relatively modern CPU (e.g. i7-6700K)
//! * Has a [solid track record](#trophies) of uncovered security bugs: the __only__ (to the date) __vulnerability in OpenSSL with the [critical](https://www.openssl.org/news/secadv/20160926.txt) score mark__ was discovered by honggfuzz. See the [Trophies](#trophies) paragraph for the summary of findings to the date
//! * Uses low-level interfaces to monitor processes (e.g. _ptrace_ under Linux). As opposed to other fuzzers, it __will discover and report hijacked/ignored signals__ (intercepted and potentially hidden by signal handlers)
//! * Easy-to-use, feed it a simple corpus directory (can even be empty) and it will work its way up expanding it utilizing feedback-based coverage metrics
//! * Supports several (more than any other coverage-based feedback-driven fuzzer) hardware-based (CPU: branch/instruction counting, __Intel BTS__, __Intel PT__) and software-based [feedback-driven fuzzing](https://github.com/google/honggfuzz/blob/master/docs/FeedbackDrivenFuzzing.md) methods known from other fuzzers (libfuzzer, afl)
//! * Works (at least) under GNU/Linux, FreeBSD, Mac OS X, Windows/CygWin and [Android](https://github.com/google/honggfuzz/blob/master/docs/Android.md)
//! * Supports the __persistent fuzzing mode__ (long-lived process calling a fuzzed API repeatedly) with libhfuzz/libhfuzz.a. More on that can be found [here](https://github.com/google/honggfuzz/blob/master/docs/PersistentFuzzing.md)
//! * [Can fuzz remote/standalone long-lasting processes](https://github.com/google/honggfuzz/blob/master/docs/AttachingToPid.md) (e.g. network servers like __Apache's httpd__ and __ISC's bind__), though the [persistent fuzzing mode](https://github.com/google/honggfuzz/blob/master/docs/PersistentFuzzing.md) is suggested instead: as it's faster and multiple instances of a service can be fuzzed with this
//! * It comes with the __[examples](https://github.com/google/honggfuzz/tree/master/examples) directory__, consisting of real world fuzz setups for widely-used software (e.g. Apache and OpenSSL)
//! 
//! ## How to use this crate
//! 
//! Install honggfuzz commands to build with instrumentation and fuzz
//! 
//! ```sh
//! # installs hfuzz and honggfuzz subcommands in cargo
//! cargo install honggfuzz
//! ```
//! 
//! Add to your dependencies
//! 
//! ```toml
//! [dependencies]
//! honggfuzz = "0.5"
//! ```
//! 
//! Create a target to fuzz
//! 
//! ```rust
//! #[macro_use] extern crate honggfuzz;
//! 
//! fn main() {
//!     // Here you can parse `std::env::args and 
//!     // setup / initialize your project
//! 
//!     // You have full control over the loop but
//!     // you're supposed to call `fuzz` ad vitam aeternam
//!     loop {
//!         // The fuzz macro gives an arbitrary object (see `arbitrary crate`)
//!         // to a closure-like block of code.
//!         // For performance reasons, it is recommended that you use the native type
//!         // `&[u8]` when possible.
//!         // Here, this slice will contain a "random" quantity of "random" data.
//!         fuzz!(|data: &[u8]| {
//!             if data.len() != 10 {return}
//!             if data[0] != 'q' as u8 {return}
//!             if data[1] != 'w' as u8 {return}
//!             if data[2] != 'e' as u8 {return}
//!             if data[3] != 'r' as u8 {return}
//!             if data[4] != 't' as u8 {return}
//!             if data[5] != 'y' as u8 {return}
//!             if data[6] != 'u' as u8 {return}
//!             if data[7] != 'i' as u8 {return}
//!             if data[8] != 'o' as u8 {return}
//!             if data[9] != 'p' as u8 {return}
//!             panic!("BOOM")
//!         });
//!     }
//! }
//! 
//! ```
//! 
//! Fuzz for fun and profit !
//! 
//! ```sh
//! # builds with fuzzing instrumentation and then runs the "example" target
//! cargo hfuzz run example
//! ```
//! 
//! Once you got a crash, replay it easily in a debug environment
//! 
//! ```sh
//! # builds the target in debug mode and replays automatically the crash in gdb
//! cargo hfuzz run-debug example fuzzing_workspace/*.fuzz
//! ```
//! 
//! Clean
//! 
//! ```sh
//! # a wrapper on "cargo clean" which cleans the fuzzing_target directory
//! cargo hfuzz clean 
//! ```
//! 
//! Optionally, fuzz with LLVM's [sanitizers](https://github.com/japaric/rust-san)
//!
//! ```sh
//! RUSTFLAGS="-Z sanitizer=address" cargo hfuzz run example
//! ```
//! 
//! ## Relevant documentation about honggfuzz usage
//! * [USAGE](https://github.com/google/honggfuzz/blob/master/docs/USAGE.md)
//! * [FeedbackDrivenFuzzing](https://github.com/google/honggfuzz/blob/master/docs/FeedbackDrivenFuzzing.md)
//! * [PersistentFuzzing](https://github.com/google/honggfuzz/blob/master/docs/PersistentFuzzing.md)
//! 
//! ## About Rust fuzzing
//!  
//! There is other projects providing Rust fuzzing support at [github.com/rust-fuzz](https://github.com/rust-fuzz). 
//!  
//! You'll find support for [AFL](https://github.com/rust-fuzz/afl.rs) and LLVM's [LibFuzzer](https://github.com/rust-fuzz/cargo-fuzz) and there is also a [trophy case](https://github.com/rust-fuzz/trophy-case) ;-) .
//! 
//! This crate was inspired by those projects!

#[cfg(all(fuzzing, fuzzing_debug))]
extern crate memmap;

#[cfg(all(fuzzing, not(fuzzing_debug)))]
extern "C" {
    fn HF_ITER(buf_ptr: *mut *const u8, len_ptr: *mut usize );
}

/// Fuzz a closure by passing it a `&[u8]`
///
/// This slice contains a "random" quantity of "random" data.
///
/// For perstistent fuzzing to work, you have to call it ad vita aeternam in an infinite loop.
///
/// ```
/// extern crate honggfuzz;
/// use honggfuzz::fuzz;
///
/// loop {
///     fuzz(|data|{
///         if data.len() != 10 {return}
///         if data[0] != 'q' as u8 {return}
///         if data[1] != 'w' as u8 {return}
///         if data[2] != 'e' as u8 {return}
///         if data[3] != 'r' as u8 {return}
///         if data[4] != 't' as u8 {return}
///         if data[5] != 'y' as u8 {return}
///         if data[6] != 'u' as u8 {return}
///         if data[7] != 'i' as u8 {return}
///         if data[8] != 'o' as u8 {return}
///         if data[9] != 'p' as u8 {return}
///         panic!("BOOM")
///     });
/// }
/// ```
#[cfg(not(fuzzing))]
#[allow(unused_variables)]
pub fn fuzz<F>(closure: F) where F: Fn(&[u8]) {
    eprintln!("This executable hasn't been built with honggfuzz instrumentation.");
    eprintln!("Try executing \"cargo hfuzz build\" and check out \"fuzzing_target\" directory.");
    eprintln!("Or execute \"cargo hfuzz run TARGET\"");
    std::process::exit(17);
}

#[cfg(all(fuzzing, not(fuzzing_debug)))]
pub fn fuzz<F>(closure: F) where F: Fn(&[u8]) {
    let buf;
    unsafe {
        let mut buf_ptr: *const u8 = std::mem::uninitialized();
        let mut len_ptr: usize = std::mem::uninitialized();
        HF_ITER(&mut buf_ptr, &mut len_ptr);
        buf = ::std::slice::from_raw_parts(buf_ptr, len_ptr);
    }
    closure(buf);
}

#[cfg(all(fuzzing, fuzzing_debug))]
pub fn fuzz<F>(closure: F) where F: Fn(&[u8]) {
    use std::env;
    use std::fs::File;
    use memmap::MmapOptions;
    
    let filename = env::var("CARGO_HONGGFUZZ_CRASH_FILENAME").unwrap_or_else(|_|{
        eprintln!("error: Environment variable CARGO_HONGGFUZZ_CRASH_FILENAME not set. Try launching with \"cargo hfuzz run-debug TARGET CRASH_FILENAME [ ARGS ... ]\"");
        std::process::exit(1)
    });

    let file = File::open(&filename).unwrap_or_else(|_|{
        eprintln!("error: failed to open \"{}\"", &filename);
        std::process::exit(1)
    });

    let mmap = unsafe {MmapOptions::new().map(&file)}.unwrap_or_else(|_|{
        eprintln!("error: failed to mmap file \"{}\"", &filename);
        std::process::exit(1)
    });

    closure(&mmap);
}

/// Fuzz a closure-like block of code by passing it an object of arbitrary type.
///
/// You can choose the type of the argument using the syntax as in the example below.
/// Please check out the `arbitrary` crate to see which types are available.
///
/// For performance reasons, it is recommended that you use the native type `&[u8]` when possible.
///
/// For perstistent fuzzing to work, you have to call it ad vita aeternam in an infinite loop.
///
/// ```
/// #[macro_use] extern crate honggfuzz;
///
/// loop {
///     fuzz!(|data: &[u8]| {
///         if data.len() != 10 {return}
///         if data[0] != 'q' as u8 {return}
///         if data[1] != 'w' as u8 {return}
///         if data[2] != 'e' as u8 {return}
///         if data[3] != 'r' as u8 {return}
///         if data[4] != 't' as u8 {return}
///         if data[5] != 'y' as u8 {return}
///         if data[6] != 'u' as u8 {return}
///         if data[7] != 'i' as u8 {return}
///         if data[8] != 'o' as u8 {return}
///         if data[9] != 'p' as u8 {return}
///         panic!("BOOM")
///     });
/// }
/// ```
#[cfg(not(fuzzing))]
#[macro_export]
macro_rules! fuzz {
    (|$buf:ident| $body:block) => {
        honggfuzz::fuzz(|_| {});
    };
    (|$buf:ident: &[u8]| $body:block) => {
        honggfuzz::fuzz(|_| {});
    };
    (|$buf:ident: $dty: ty| $body:block) => {
        honggfuzz::fuzz(|_| {});
    };
}

#[cfg(all(fuzzing))]
#[macro_export]
macro_rules! fuzz {
    (|$buf:ident| $body:block) => {
        honggfuzz::fuzz(|$buf| $body);
    };
    (|$buf:ident: &[u8]| $body:block) => {
        honggfuzz::fuzz(|$buf| $body);
    };
    (|$buf:ident: $dty: ty| $body:block) => {
        honggfuzz::fuzz(|$buf| {
            let $buf: $dty = {
                use arbitrary::{Arbitrary, RingBuffer};
                if let Ok(d) = RingBuffer::new($buf, $buf.len()).and_then(|mut b|{
                        Arbitrary::arbitrary(&mut b).map_err(|_| "")
                    }) {
                    d
                } else {
                    return
                }
            };

            $body
        });
    };
}

