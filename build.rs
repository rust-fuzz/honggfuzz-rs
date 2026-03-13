use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(target_family = "windows")]
compile_error!("honggfuzz-rs does not currently support Windows but works well under WSL (Windows Subsystem for Linux)");

// TODO: maybe use `make-cmd` crate
#[cfg(not(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd"
)))]
const GNU_MAKE: &str = "make";
#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd"
))]
const GNU_MAKE: &str = "gmake";

/// Recursively copy a directory tree from `src` to `dst`.
fn copy_dir_all(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let ty = entry.file_type().unwrap();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path);
        } else if ty.is_symlink() {
            let target = fs::read_link(entry.path()).unwrap();
            #[cfg(unix)]
            std::os::unix::fs::symlink(&target, &dst_path).unwrap();
        } else {
            fs::copy(entry.path(), &dst_path).unwrap();
        }
    }
}

fn main() {
    // Only build honggfuzz binaries if we are in the process of building an instrumentized binary
    let honggfuzz_target = match env::var("CARGO_HONGGFUZZ_TARGET_DIR") {
        Ok(path) => path, // path where to place honggfuzz binary. provided by cargo-hfuzz command.
        Err(_) => return,
    };

    // check that "cargo hfuzz" command is at the same version as this file
    let honggfuzz_build_version =
        env::var("CARGO_HONGGFUZZ_BUILD_VERSION").unwrap_or("unknown".to_string());
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

    // Copy honggfuzz source tree into OUT_DIR so we can build in a writable
    // directory. This is required for Nix builds where the source directory
    // is read-only.
    let build_dir = out_dir.join("honggfuzz");
    if build_dir.exists() {
        fs::remove_dir_all(&build_dir).unwrap();
    }
    copy_dir_all(Path::new("honggfuzz"), &build_dir);

    let build_dir_str = build_dir.to_str().unwrap();

    // build honggfuzz command and hfuzz static library
    let status = Command::new(GNU_MAKE)
        .args(&["-C", build_dir_str, "honggfuzz", "libhfuzz/libhfuzz.a", "libhfcommon/libhfcommon.a"])
        .status()
        .unwrap_or_else(|_e| panic!("failed to run \"{} -C {} honggfuzz libhfuzz/libhfuzz.a libhfcommon/libhfcommon.a\"", GNU_MAKE, build_dir_str));
    assert!(status.success());

    fs::copy(build_dir.join("libhfuzz/libhfuzz.a"), out_dir.join("libhfuzz.a")).unwrap();
    fs::copy(
        build_dir.join("libhfcommon/libhfcommon.a"),
        out_dir.join("libhfcommon.a"),
    )
    .unwrap();
    fs::copy(build_dir.join("honggfuzz"), honggfuzz_target.join("honggfuzz")).unwrap();

    // tell cargo how to link final executable to hfuzz static library
    println!("cargo:rustc-link-lib=static={}", "hfuzz");
    println!("cargo:rustc-link-lib=static={}", "hfcommon");
    println!("cargo:rustc-link-search=native={}", out_dir.display());
}
