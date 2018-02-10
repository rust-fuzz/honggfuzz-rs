use std::env;
use std::process::{self, Command};
use std::os::unix::process::CommandExt;

const HONGGFUZZ_TARGET_DIR: &'static str = "fuzzing_target";

#[cfg(not(target_arch="x86_64"))]
compile_error!("honggfuzz currently only support x86_64 architecture");

#[cfg(not(any(target_os="linux", target_os="macos")))]
compile_error!("honggfuzz currently only support Linux and OS X operating systems");

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

	let command = format!("{}/honggfuzz", HONGGFUZZ_TARGET_DIR);
	Command::new(&command) // exec honggfuzz replacing current process
        .args(args)
        .env("ASAN_OPTIONS", asan_options)
        .env("TSAN_OPTIONS", tsan_options)
        .exec();

    // code flow will only reach here if honggfuzz failed to execute
    eprintln!("cannot execute {}, try to execute \"cargo hfuzz build\" from fuzzed project directory", &command);
    process::exit(1);
}