use std::fs;
use std::env;
use std::process::{self, Command};
use std::os::unix::process::CommandExt;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const HONGGFUZZ_TARGET: &'static str = "hfuzz_target";
const HONGGFUZZ_WORKSPACE: &'static str = "hfuzz_workspace";

#[cfg(not(target_arch="x86_64"))]
compile_error!("honggfuzz currently only support x86_64 architecture");

#[cfg(not(any(target_os="linux", target_os="macos")))]
compile_error!("honggfuzz currently only support Linux and OS X operating systems");

fn target_triple() -> &'static str {
    if cfg!(target_os="linux") {
        "x86_64-unknown-linux-gnu"
    } else if cfg!(target_os="macos") {
        "x86_64-apple-darwin"
    } else {
        unreachable!()
    }
}

fn debugger_command(target: &str) -> Command {
    let mut cmd;

    if cfg!(target_os="linux") {
        cmd = Command::new("gdb");
        cmd.args(&["-ex", "b rust_panic", "-ex", "r", "-ex", "bt", "--args", &format!("{}/{}/debug/{}", HONGGFUZZ_TARGET, target_triple(), target)]);
    } else if cfg!(target_os="macos") {
        cmd = Command::new("lldb");
        cmd.args(&["-o", "b rust_panic", "-o", "r", "-o", "bt", "-f", &format!("{}/{}/debug/{}", HONGGFUZZ_TARGET, target_triple(), target), "--"]);
    } else {
        unreachable!()
    }

    cmd 
}

fn hfuzz_run<T>(mut args: T, debug: bool) where T: std::iter::Iterator<Item=String> {

    let target = args.next().unwrap_or_else(||{
        eprintln!("please specify the name of the target like this \"cargo hfuzz run[-debug] TARGET [ ARGS ... ]\"");
        process::exit(1);
    });

    hfuzz_build(vec!["--bin".to_string(), target.clone()].into_iter(), debug);

    if debug {
        let crash_filename = args.next().unwrap_or_else(||{
            eprintln!("please specify the crash filename like this \"cargo hfuzz run-debug TARGET CRASH_FILENAME [ ARGS ... ]\"");
            process::exit(1);
        });

        let status = debugger_command(&target)
            .args(args)
            .env("CARGO_HONGGFUZZ_CRASH_FILENAME", crash_filename)
            .env("RUST_BACKTRACE", env::var("RUST_BACKTRACE").unwrap_or("1".to_string()))
            .status()
            .unwrap();
        if !status.success() {
             process::exit(status.code().unwrap_or(1));
        }
    } else {
        // add some flags to sanitizers to make them work with Rust code
        let asan_options = env::var("ASAN_OPTIONS").unwrap_or_default();
        let asan_options = format!("detect_odr_violation=0:{}", asan_options);

        let tsan_options = env::var("TSAN_OPTIONS").unwrap_or_default();
        let tsan_options = format!("report_signal_unsafe=0:{}", tsan_options);

        // get user-defined args for honggfuzz
        let hfuzz_run_args = env::var("HFUZZ_RUN_ARGS").unwrap_or_default();
        // FIXME: we split by whitespace without respecting escaping or quotes
        let hfuzz_run_args = hfuzz_run_args.split_whitespace();

        fs::create_dir_all(&format!("{}/{}/input", HONGGFUZZ_WORKSPACE, target)).unwrap_or_else(|_| {
            println!("error: failed to create \"{}/{}/input\"", HONGGFUZZ_WORKSPACE, target);
        });

        let command = format!("{}/honggfuzz", HONGGFUZZ_TARGET);
        Command::new(&command) // exec honggfuzz replacing current process
            .args(&["-W", &format!("{}/{}", HONGGFUZZ_WORKSPACE, target), "-f", &format!("{}/{}/input", HONGGFUZZ_WORKSPACE, target), "-P"])
            .args(hfuzz_run_args) // allows user-specified arguments to be given to honggfuzz
            .args(&["--", &format!("{}/{}/release/{}", HONGGFUZZ_TARGET, target_triple(), target)])
            .args(args)
            .env("ASAN_OPTIONS", asan_options)
            .env("TSAN_OPTIONS", tsan_options)
            .exec();

        // code flow will only reach here if honggfuzz failed to execute
        eprintln!("cannot execute {}, try to execute \"cargo hfuzz-build\" from fuzzed project directory", &command);
        process::exit(1);
    }
}

fn hfuzz_build<T>(args: T, debug: bool) where T: std::iter::Iterator<Item=String> {
    let mut rustflags = "\
    --cfg fuzzing \
    -C debug-assertions \
    -C overflow_checks \
    ".to_string();

    if debug {
        rustflags.push_str("\
        --cfg fuzzing_debug \
        -C panic=unwind \
        -C opt-level=0 \
        -C debuginfo=2 \
        ");
    } else {
        rustflags.push_str("\
        -C panic=abort \
        -C opt-level=3 \
        -C debuginfo=0 \
        -C passes=sancov \
        -C llvm-args=-sanitizer-coverage-level=4 \
        -C llvm-args=-sanitizer-coverage-trace-pc-guard \
        -C llvm-args=-sanitizer-coverage-prune-blocks=0 \
        ");
    }

    // trace-compares doesn't work on macOS without sanitizer
    if cfg!(not(target_os="macos")) {
        rustflags.push_str("\
        -C llvm-args=-sanitizer-coverage-trace-compares \
        ");
    }

    // add user provided flags
    rustflags.push_str(&env::var("RUSTFLAGS").unwrap_or_default());

    // get user-defined args for building
    let hfuzz_build_args = env::var("HFUZZ_BUILD_ARGS").unwrap_or_default();
    // FIXME: we split by whitespace without respecting escaping or quotes
    let hfuzz_build_args = hfuzz_build_args.split_whitespace();

    let cargo_bin = env::var("CARGO").unwrap();
    let mut command = Command::new(cargo_bin);
    command.args(&["build", "--target", target_triple()]) // HACK to avoid building build scripts with rustflags
        .args(args)
        .args(hfuzz_build_args) // allows user-specified arguments to be given to cargo build
        .env("RUSTFLAGS", rustflags)
        .env("CARGO_TARGET_DIR", HONGGFUZZ_TARGET); // change target_dir to not clash with regular builds
    
    if !debug {
        command.arg("--release")
            .env("CARGO_HONGGFUZZ_BUILD_VERSION", VERSION)   // used by build.rs to check that versions are in sync
            .env("CARGO_HONGGFUZZ_TARGET_DIR", HONGGFUZZ_TARGET); // env variable to be read by build.rs script 
    }                                                                 // to place honggfuzz executable at a known location

    let status = command.status().unwrap();
    if !status.success() {
         process::exit(status.code().unwrap_or(1));
    }
}

fn hfuzz_clean<T>(args: T) where T: std::iter::Iterator<Item=String> {
    let cargo_bin = env::var("CARGO").unwrap();
    let status = Command::new(cargo_bin)
        .args(&["clean"])
        .args(args)
        .env("CARGO_TARGET_DIR", HONGGFUZZ_TARGET) // change target_dir to not clash with regular builds
        .status()
        .unwrap();
    if !status.success() {
         process::exit(status.code().unwrap_or(1));
    }
}

fn main() {
    let mut args = env::args().skip(1);
    if args.next() != Some("hfuzz".to_string()) {
        eprintln!("please launch as a cargo subcommand: \"cargo hfuzz ...\"");
        process::exit(1);
    }

    match args.next() {
        Some(ref s) if s == "build" => {
            hfuzz_build(args, false);
        }
        Some(ref s) if s == "build-debug" => {
            hfuzz_build(args, true);
        }
        Some(ref s) if s == "run" => {
            hfuzz_run(args, false);
        }
        Some(ref s) if s == "run-debug" => {
            hfuzz_run(args, true);
        }
        Some(ref s) if s == "clean" => {
            hfuzz_clean(args);
        }
        _ => {
            eprintln!("possible commands are: run, run-debug, build, build-debug, clean");
            process::exit(1);
        }
    }
}
