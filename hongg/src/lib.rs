//! ## About Honggfuzz
//!
//! Honggfuzz is a security oriented fuzzer with powerful analysis options. Supports evolutionary, feedback-driven fuzzing based on code coverage (software- and hardware-based).
//!
//! * project homepage [honggfuzz.com](http://honggfuzz.com/)
//! * project repository [github.com/google/honggfuzz](https://github.com/google/honggfuzz)
//! * this upstream project is maintained by Google, but ...
//! * this is NOT an official Google product
//!
//! ## Compatibility
//!
//! * __Rust__: stable, beta, nightly
//! * __OS__: GNU/Linux, macOS, FreeBSD, NetBSD, Android, WSL (Windows Subsystem for Linux)
//! * __Arch__: x86_64, x86, arm64-v8a, armeabi-v7a, armeabi
//! * __Sanitizer__: none, address, thread, leak
//!
//! ## Dependencies
//!
//! ### Linux
//!
//! * C compiler: `cc`
//! * GNU Make: `make`
//! * GNU Binutils development files for the BFD library: `libbfd.h`
//! * libunwind development files: `libunwind.h`
//! * liblzma development files
//!
//! For example on Debian and its derivatives:
//!
//! ```sh
//! sudo apt install build-essential binutils-dev libunwind-dev
//! ```
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
//! ```rust,should_panic
//! use honggfuzz::fuzz;
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
//!             if data.len() != 3 {return}
//!             if data[0] != b'h' {return}
//!             if data[1] != b'e' {return}
//!             if data[2] != b'y' {return}
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
//! # builds with fuzzing instrumentation and then fuzz the "example" target
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
//! You can also build and run your project without compile-time software instrumentation (LLVM's SanCov passes)
//!
//! This allows you for example to try hardware-only feedback driven fuzzing:
//!
//! ```sh
//! # builds without fuzzing instrumentation and then fuzz the "example" target using hardware-based feedback
//! HFUZZ_RUN_ARGS="--linux_perf_ipt_block --linux_perf_instr --linux_perf_branch" cargo hfuzz run-no-instr example
//! ```
//!
//! Clean
//!
//! ```sh
//! # a wrapper on "cargo clean" which cleans the fuzzing_target directory
//! cargo hfuzz clean
//! ```
//!
//! Version
//!
//! ```sh
//! cargo hfuzz version
//! ```
//!
//! ### Environment variables
//!
//! #### `RUSTFLAGS`
//!
//! You can use `RUSTFLAGS` to send additional arguments to `rustc`.
//!
//! For instance, you can enable the use of LLVM's [sanitizers](https://github.com/japaric/rust-san).
//! This is a recommended option if you want to test your `unsafe` rust code but it will have an impact on performance.
//!
//! ```sh
//! RUSTFLAGS="-Z sanitizer=address" cargo hfuzz run example
//! ```
//!
//! #### `HFUZZ_BUILD_ARGS`
//!
//! You can use `HFUZZ_BUILD_ARGS` to send additional arguments to `cargo build`.
//!
//! #### `HFUZZ_RUN_ARGS`
//!
//! You can use `HFUZZ_RUN_ARGS` to send additional arguments to `honggfuzz`.
//! See [USAGE](https://github.com/google/honggfuzz/blob/master/docs/USAGE.md) for the list of those.
//!
//! For example:
//!
//! ```sh
//! # 1 second of timeout
//! # use 12 fuzzing thread
//! # be verbose
//! # stop after 1000000 fuzzing iteration
//! # exit upon crash
//! HFUZZ_RUN_ARGS="-t 1 -n 12 -v -N 1000000 --exit_upon_crash" cargo hfuzz run example
//! ```
//!
//! #### `HFUZZ_DEBUGGER`
//!
//! By default we use `rust-lldb` but you can change it to `rust-gdb`, `gdb`, `/usr/bin/lldb-7` ...
//!
//! #### `CARGO_TARGET_DIR`
//!
//! Target compilation directory, defaults to `hfuzz_target` to not clash with `cargo build`'s default `target` directory.
//!
//! #### `HFUZZ_WORKSPACE`
//!
//! Honggfuzz working directory, defaults to `hfuzz_workspace`.
//!
//! #### `HFUZZ_INPUT`
//!
//! Honggfuzz input files (also called "corpus"), defaults to `$HFUZZ_WORKSPACE/{TARGET}/input`.
//!
//! ## Conditionnal compilation
//!
//! Sometimes, it is necessary to make some specific adaptation to your code to yield a better fuzzing efficiency.
//!
//! For instance:
//! - Make you software behavior as much as possible deterministic on the fuzzing input
//!   - [PRNG](https://en.wikipedia.org/wiki/Pseudorandom_number_generator)s must be seeded with a constant or the fuzzer input
//!   - Behavior shouldn't change based on the computer's clock.
//!   - Avoid potential undeterministic behavior from racing threads.
//!   - ...
//! - Never ever call `std::process::exit()`.
//! - Disable logging and other unnecessary functionnalities.
//! - Try to avoid modifying global state when possible.
//!
//!
//! When building with `cargo hfuzz`, the argument `--cfg fuzzing` is passed to `rustc` to allow you to condition the compilation of thoses adaptations thanks to the `cfg` macro like so:
//!
//! ```rust
//! # use rand::{self, Rng, SeedableRng};
//! # use rand_chacha;
//! # fn main() {
//! #[cfg(fuzzing)]
//! let mut rng = rand_chacha::ChaCha8Rng::from_seed(&[0]);
//! #[cfg(not(fuzzing))]
//! let mut rng = rand::thread_rng();
//! # }
//! ```
//!
//! Also, when building in debug mode, the `fuzzing_debug` argument is added in addition to `fuzzing`.
//!
//! For more information about conditional compilation, please see the [reference](https://doc.rust-lang.org/reference/attributes.html#conditional-compilation).
//!
//! ## Relevant documentation about honggfuzz
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

/// Re-export of arbitrary crate used to generate structured inputs
pub use arbitrary;

#[cfg(all(fuzzing, not(fuzzing_debug)))]
extern "C" {
    fn HF_ITER(buf_ptr: *mut *const u8, len_ptr: *mut usize);
}

/// Fuzz a closure by passing it a `&[u8]`
///
/// This slice contains a "random" quantity of "random" data.
///
/// For perstistent fuzzing to work, you have to call it ad vita aeternam in an infinite loop.
///
/// The closure is assumed to be unwind-safe, which might be unsafe. For more info, check the
/// [`std::panic::UnwindSafe`] trait.
///
/// ```rust,should_panic
/// # use honggfuzz::fuzz;
/// # fn main() {
/// loop {
///     fuzz(|data|{
///         if data.len() != 3 {return}
///         if data[0] != b'h' {return}
///         if data[1] != b'e' {return}
///         if data[2] != b'y' {return}
///         panic!("BOOM")
///     });
/// }
/// # }
/// ```
#[cfg(not(fuzzing))]
#[allow(unused_variables)]
pub fn fuzz<F>(closure: F)
where
    F: FnOnce(&[u8]),
{
    eprintln!("This executable hasn't been built with \"cargo hongg\".");
    eprintln!("Try executing \"cargo hongg build\" and check out \"hfuzz_target\" directory.");
    eprintln!("Or execute \"cargo cargo run --help\"");
    std::process::exit(17);
}

// Registers a panic hook that aborts the process before unwinding.
// It is useful to abort before unwinding so that the fuzzer will then be
// able to analyse the process stack frames to tell different bugs appart.
#[cfg(all(fuzzing, not(fuzzing_debug)))]
lazy_static::lazy_static! {
    static ref PANIC_HOOK: () = {
        std::panic::set_hook(Box::new(|_| {
            std::process::abort();
        }))
    };
}

#[cfg(all(fuzzing, not(fuzzing_debug)))]
pub fn fuzz<F>(closure: F)
where
    F: FnOnce(&[u8]),
{
    use std::mem::MaybeUninit;

    // sets panic hook if not already done
    lazy_static::initialize(&PANIC_HOOK);

    // get buffer from honggfuzz runtime
    let buf;

    let mut buf_ptr = MaybeUninit::<*const u8>::uninit();
    let mut len_ptr = MaybeUninit::<usize>::uninit();

    unsafe {
        HF_ITER(buf_ptr.as_mut_ptr(), len_ptr.as_mut_ptr());
        buf = ::std::slice::from_raw_parts(buf_ptr.assume_init(), len_ptr.assume_init());
    }

    // We still catch unwinding panics just in case the fuzzed code modifies
    // the panic hook.
    // If so, the fuzzer will be unable to tell different bugs appart and you will
    // only be able to find one bug at a time before fixing it to then find a new one.
    // The closure is assumed to be unwind-safe, which might be unsafe. For more info, check the
    // [`std::panic::UnwindSafe`] trait.
    let did_panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        closure(buf);
    }))
    .is_err();

    if did_panic {
        // hopefully the custom panic hook will be called before and abort the
        // process before the stack frames are unwinded.
        std::process::abort();
    }
}

#[cfg(all(fuzzing, fuzzing_debug))]
pub fn fuzz<F>(closure: F)
where
    F: FnOnce(&[u8]),
{
    use fs_err::File;
    use memmap::MmapOptions;
    use std::env;

    let filename = env::var("CARGO_HONGGFUZZ_CRASH_FILENAME").unwrap_or_else(|_|{
        eprintln!("error: Environment variable CARGO_HONGGFUZZ_CRASH_FILENAME not set. Try launching with \"cargo hfuzz run-debug TARGET CRASH_FILENAME [ ARGS ... ]\"");
        std::process::exit(1);
    });

    let file = File::open(&filename).unwrap_or_else(|_| {
        eprintln!("error: failed to open \"{}\"", &filename);
        std::process::exit(1);
    });

    let mmap = unsafe { MmapOptions::new().map(&file) }.unwrap_or_else(|_| {
        eprintln!("error: failed to mmap file \"{}\"", &filename);
        std::process::exit(1);
    });

    closure(&mmap);

    eprintln!("This crashfile didn't trigger any panics...");
    eprintln!("Are you sure that you selected the correct crashfile and that your program's behavior is entirely deterministic and only dependent on the fuzzing input?");
    std::process::exit(2);
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
/// ```rust,should_panic
/// # use honggfuzz::fuzz;
/// # fn main() {
/// loop {
///     fuzz!(|data: &[u8]| {
///         if data.len() != 3 {return}
///         if data[0] != b'h' {return}
///         if data[1] != b'e' {return}
///         if data[2] != b'y' {return}
///         panic!("BOOM")
///     });
/// }
/// # }
/// ```

#[macro_export]
macro_rules! fuzz {
    (|$buf:ident| $body:block) => {
        $crate::fuzz(|$buf| $body);
    };
    (|$buf:ident: &[u8]| $body:block) => {
        $crate::fuzz(|$buf| $body);
    };
    (|$buf:ident: $dty:ty| $body:block) => {
        $crate::fuzz(|$buf| {
            let $buf: $dty = {
                use $crate::arbitrary::{Arbitrary, Unstructured};

                let mut buf = Unstructured::new($buf);
                if let Ok(buf) = Arbitrary::arbitrary(&mut buf) {
                    buf
                } else {
                    return;
                }
            };

            $body
        });
    };
}
