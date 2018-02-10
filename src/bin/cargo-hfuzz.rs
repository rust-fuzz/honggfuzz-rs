use std::env;
use std::process::{self, Command};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const HONGGFUZZ_TARGET_DIR: &'static str = "fuzzing_target";

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

fn hfuzz_build<T>(args: T, debug: bool) where T: std::iter::Iterator<Item=String> {
	// compilation flags to instrumentize generated code
	let rustflags = env::var("RUSTFLAGS").unwrap_or_default();
	let rustflags = format!("\
	--cfg fuzzing \
	-Cpanic=abort \
	-Copt-level=3 \
	-Cdebuginfo=0 \
	-Cdebug-assertions \
	-Coverflow_checks \
	-Cpasses=sancov \
	-Cllvm-args=-sanitizer-coverage-level=4 \
	-Cllvm-args=-sanitizer-coverage-trace-pc-guard \
	-Cllvm-args=-sanitizer-coverage-trace-compares \
	-Cllvm-args=-sanitizer-coverage-prune-blocks=0 \
	{}", rustflags);

	let cargo_bin = env::var("CARGO").unwrap();
	let status = Command::new(cargo_bin)
        .args(&["build", "--target", target_triple()]) // HACK to avoid building build scripts with rustflags
        .args(args)
        .env("RUSTFLAGS", rustflags)
        .env("CARGO_TARGET_DIR", HONGGFUZZ_TARGET_DIR) // change target_dir to not clash with regular builds
        .env("CARGO_HONGGFUZZ_BUILD_VERSION", VERSION)   // used by build.rs to check that versions are in sync
        .env("CARGO_HONGGFUZZ_TARGET_DIR", HONGGFUZZ_TARGET_DIR) // env variable to be read by build.rs script 
        .status()                                                // to place honggfuzz executable at a known location
        .unwrap();
    process::exit(status.code().unwrap_or(1));
}

fn hfuzz_clean<T>(args: T) where T: std::iter::Iterator<Item=String> {
	let cargo_bin = env::var("CARGO").unwrap();
	let status = Command::new(cargo_bin)
        .args(&["clean"])
        .args(args)
        .env("CARGO_TARGET_DIR", HONGGFUZZ_TARGET_DIR) // change target_dir to not clash with regular builds
        .status()
        .unwrap();
    process::exit(status.code().unwrap_or(1));
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
			unimplemented!()
		}
		Some(ref s) if s == "run" => {
			unimplemented!()
		}
		Some(ref s) if s == "run-debug" => {
			unimplemented!()
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