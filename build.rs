use std::env;
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
    let honggfuzz_target=  match env::var("CARGO_HONGGFUZZ_TARGET_DIR") {
        Ok(path) => path, // path where to place honggfuzz binary. provided by cargo-hfuzz command.
        Err(_) => return
    };

    // check that "cargo hfuzz" command is at the same version as this file
    let honggfuzz_build_version = env::var("CARGO_HONGGFUZZ_BUILD_VERSION").unwrap_or("unknown".to_string());
    if VERSION != honggfuzz_build_version {
        eprintln!("honggfuzz dependency ({}) and build command ({}) versions do not match",
                  VERSION, honggfuzz_build_version);
        process::exit(1);
    }

    let out_dir = env::var("OUT_DIR").unwrap(); // from cargo
    let crate_root = env::var("CRATE_ROOT").unwrap(); //from honggfuzz

    // clean upsteam honggfuzz directory
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

    // copy hfuzz static library to output directory
    let status = Command::new("cp")
        .args(&["honggfuzz/libhfuzz/libhfuzz.a", &out_dir])
        .status()
        .expect(&format!("failed to run \"cp honggfuzz/libhfuzz/libhfuzz.a {}\"", &out_dir));
    assert!(status.success());
    let status = Command::new("cp")
        .args(&["honggfuzz/libhfcommon/libhfcommon.a", &out_dir])
        .status()
        .expect(&format!("failed to run \"cp honggfuzz/libhfcommon/libhfcommon.a {}\"", &out_dir));
    assert!(status.success());

    // copy honggfuzz executable to honggfuzz target directory
    let status = Command::new("cp")
        .args(&["honggfuzz/honggfuzz", &format!("{}/{}", &crate_root, &honggfuzz_target)])
        .status()
        .expect(&format!("failed to run \"cp honggfuzz/honggfuzz {}\"", &honggfuzz_target));
    assert!(status.success());

    // tell cargo how to link final executable to hfuzz static library
    println!("cargo:rustc-link-lib=static={}", "hfuzz");
    println!("cargo:rustc-link-lib=static={}", "hfcommon");
    println!("cargo:rustc-link-search=native={}", &out_dir);
}
