use std::env;
use std::path::PathBuf;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(target_family = "windows")]
compile_error!("honggfuzz-rs does not currently support Windows but works well under WSL (Windows Subsystem for Linux)");

// TODO: maybe use `make-cmd` crate
#[cfg(not(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "bitrig",
    target_os = "openbsd",
    target_os = "netbsd"
)))]
const GNU_MAKE: &str = "make";
#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "bitrig",
    target_os = "openbsd",
    target_os = "netbsd"
))]
const GNU_MAKE: &str = "gmake";

#[track_caller]
fn run_cmd(cmd: impl std::convert::AsRef<str>) {
    let full = cmd.as_ref();
    let mut iter = full.split_whitespace();
    let cmd = iter.next().expect("Command is never empty. qed");
    let status = ::std::process::Command::new(cmd)
        .args(iter)
        .status()
        .expect(format!("Failed to spawn process \"{}\"", &full).as_str());

    assert!(
        status.success(),
        "Command failed ({:?}): \"{}\"",
        &status,
        &full
    );
}

macro_rules! run_cmd {
    ($fmtcmd:expr $(, $args:expr )* $(,)? ) => {
        let full: String = format!($fmtcmd $(, $args )*);
        run_cmd(full);
    };
}

fn main() {
    // Only build honggfuzz binaries if we are in the process of building an instrumentized binary
    let honggfuzz_target = match env::var("CARGO_HONGGFUZZ_TARGET_DIR") {
        Ok(path) => PathBuf::from(path), // path where to place honggfuzz binary. provided by cargo-hfuzz command.
        Err(_) => return,
    };

    let out_dir = env::var("OUT_DIR").unwrap(); // from cargo
    let crate_root = env::var("CRATE_ROOT").unwrap(); //from honggfuzz

    let honggfuzz_target = if honggfuzz_target.is_absolute() {
        // in case CARGO_HONGGFUZZ_TARGET_DIR was initialized
        // from an absolute CARGO_TARGET_DIR we should not
        // prepend the crate root again
        honggfuzz_target
    } else {
        PathBuf::from(crate_root).join(honggfuzz_target)
    };

    // check that "cargo hongg" command is at the same version as this file
    let honggfuzz_build_version =
        env::var("CARGO_HONGGFUZZ_BUILD_VERSION").unwrap_or("unknown".to_string());
    if VERSION != honggfuzz_build_version {
        eprintln!("The version of the honggfuzz library dependency ({0}) and the version of the `cargo-hfuzz` executable ({1}) do not match.\n\
                   If updating both by running `cargo update` and `cargo install honggfuzz` does not work, you can either:\n\
                   - change the dependency in `Cargo.toml` to `honggfuzz = \"={1}\"`\n\
                   - or run `cargo install honggfuzz --version {0}`",
                  VERSION, honggfuzz_build_version);
        std::process::exit(1);
    }

    // clean upsteam honggfuzz directory
    run_cmd!("{} -C honggfuzz clean", GNU_MAKE);
    // TODO: maybe it's not a good idea to always clean the sources..

    // build honggfuzz command and hfuzz static library
    run_cmd!(
        "{} -C honggfuzz honggfuzz libhfuzz/libhfuzz.a libhfcommon/libhfcommon.a",
        GNU_MAKE
    );

    // copy hfuzz static library to output directory
    run_cmd!("cp honggfuzz/libhfuzz/libhfuzz.a {}", &out_dir);

    run_cmd!("cp honggfuzz/libhfcommon/libhfcommon.a {}", &out_dir);

    // copy honggfuzz executable to honggfuzz target directory
    run_cmd!("cp honggfuzz/honggfuzz {}", honggfuzz_target.display());

    // tell cargo how to link final executable to hfuzz static library
    println!("cargo:rustc-link-lib=static={}", "hfuzz");
    println!("cargo:rustc-link-lib=static={}", "hfcommon");
    println!("cargo:rustc-link-search=native={}", &out_dir);
}
