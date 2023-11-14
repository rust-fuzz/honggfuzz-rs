use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(target_family="windows")]
compile_error!("honggfuzz-rs does not currently support Windows but works well under WSL (Windows Subsystem for Linux)");

// TODO: maybe use `make-cmd` crate
#[cfg(not(any(target_os = "freebsd", target_os = "dragonfly", target_os = "bitrig", target_os = "openbsd", target_os = "netbsd")))]
const GNU_MAKE: &str = "make";
#[cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "bitrig", target_os = "openbsd", target_os = "netbsd"))]
const GNU_MAKE: &str = "gmake";

fn main() {
    // Only build honggfuzz binaries if we are in the process of building an instrumentized binary
    let honggfuzz_target = match env::var("CARGO_HONGGFUZZ_TARGET_DIR") {
        Ok(path) => path, // path where to place honggfuzz binary. provided by cargo-hfuzz command.
        Err(_) => return
    };

    // check that "cargo hfuzz" command is at the same version as this file
    let honggfuzz_build_version = env::var("CARGO_HONGGFUZZ_BUILD_VERSION").unwrap_or("unknown".to_string());
    if VERSION != honggfuzz_build_version {
        eprintln!("The version of the honggfuzz library dependency ({0}) and the version of the `cargo-hfuzz` executable ({1}) do not match.\n\
                   If updating both by running `cargo update` and `cargo install honggfuzz` does not work, you can either:\n\
                   - change the dependency in `Cargo.toml` to `honggfuzz = \"={1}\"`\n\
                   - or run `cargo install honggfuzz --version {0}`",
                  VERSION, honggfuzz_build_version);
        process::exit(1);
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap()); // from cargo
    let honggfuzz_target = Path::new(&env::var("CRATE_ROOT").unwrap()) // from honggfuzz
        .join(honggfuzz_target); // resolve the original honggfuzz_target relative to CRATE_ROOT

    // clean upstream honggfuzz directory
    let status = Command::new(GNU_MAKE)
        .args(&["-C", "honggfuzz", "clean"])
        .status()
        .expect("failed to run \"make -C honggfuzz clean\"");
    assert!(status.success());
    // TODO: maybe it's not a good idea to always clean the sources..

    // build honggfuzz command and hfuzz static library
    let status = Command::new(GNU_MAKE)
        .args(&["-C", "honggfuzz", "honggfuzz", "libhfuzz/libhfuzz.a", "libhfcommon/libhfcommon.a"])
        .status()
        .expect("failed to run \"make -C honggfuzz hongfuzz libhfuzz/libhfuzz.a libhfcommon/libhfcommon.a\"");
    assert!(status.success());

    fs::copy("honggfuzz/libhfuzz/libhfuzz.a", out_dir.join("libhfuzz.a")).unwrap();
    fs::copy("honggfuzz/libhfcommon/libhfcommon.a", out_dir.join("libhfcommon.a")).unwrap();
    fs::copy("honggfuzz/honggfuzz", honggfuzz_target.join("honggfuzz")).unwrap();

    // tell cargo how to link final executable to hfuzz static library
    println!("cargo:rustc-link-lib=static={}", "hfuzz");
    println!("cargo:rustc-link-lib=static={}", "hfcommon");
    println!("cargo:rustc-link-search=native={}", out_dir.display());
}
