use std::env;
use std::process::{self, Command};

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

fn main() {
	let mut args = env::args().skip(1);
	if args.next() != Some("honggfuzz-build".to_string()) {
		eprintln!("please launch as a cargo subcommand: \"cargo honggfuzz-build ...\"");
		process::exit(1);
	}

	let honggfuzz_target_dir = "fuzzing_target";

	// compilation flags to instrumentize generated code
	let rustflags = env::var("RUSTFLAGS").unwrap_or_default();
	let rustflags = format!("\
	--cfg fuzzing \
	-Cpanic=abort \
	-Copt-level=3 \
	-Cdebuginfo=0 \
	-Cdebug-assertions \
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
        .env("CARGO_TARGET_DIR", honggfuzz_target_dir) // change target_dir to not clash with regular builds
        .env("CARGO_HONGGFUZZ_TARGET_DIR", honggfuzz_target_dir) // env variable to be read by build.rs script 
        .status()                                                // to place honggfuzz executable at a known location
        .unwrap();
    process::exit(status.code().unwrap_or(1));
}