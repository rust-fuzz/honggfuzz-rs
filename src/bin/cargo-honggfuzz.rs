use std::env;
use std::os::unix::process::CommandExt;
use std::process::{self, Command};

const HONGGFUZZ_TARGET: &str = "hfuzz_target";

#[cfg(target_family = "windows")]
compile_error!("honggfuzz-rs does not currently support Windows but works well under WSL (Windows Subsystem for Linux)");

fn main() {
    let mut args = env::args().skip(1);
    if args.next() != Some("honggfuzz".to_string()) {
        eprintln!("please launch as a cargo subcommand: \"cargo honggfuzz ...\"");
        process::exit(1);
    }

    // add some flags to sanitizers to make them work with Rust code
    let asan_options = env::var("ASAN_OPTIONS").unwrap_or_default();
    let asan_options = format!("detect_odr_violation=0:{}", asan_options);

    let tsan_options = env::var("TSAN_OPTIONS").unwrap_or_default();
    let tsan_options = format!("report_signal_unsafe=0:{}", tsan_options);

    let command = format!("{}/honggfuzz", HONGGFUZZ_TARGET);
    Command::new(&command) // exec honggfuzz replacing current process
        .args(args)
        .env("ASAN_OPTIONS", asan_options)
        .env("TSAN_OPTIONS", tsan_options)
        .exec();

    // code flow will only reach here if honggfuzz failed to execute
    eprintln!(
        "cannot execute {}, try to execute \"cargo hfuzz build\" from fuzzed project directory",
        &command,
    );
    process::exit(1);
}
