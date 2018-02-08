use std::env;
use std::process::{self, Command};

#[cfg(not(target_arch="x86_64"))]
compile_error!("honggfuzz currently only support x86_64 architecture");

#[cfg(not(any(target_os="linux", target_os="macos")))]
compile_error!("honggfuzz currently only support Linux and OS X operating systems");

fn main() {
	let mut args = env::args().skip(1);
	if args.next() != Some("hfuzz-clean".to_string()) {
		eprintln!("please launch as a cargo subcommand: \"cargo hfuzz-clean ...\"");
		process::exit(1);
	}

	let honggfuzz_target_dir = "fuzzing_target";

	let cargo_bin = env::var("CARGO").unwrap();
	let status = Command::new(cargo_bin)
        .args(&["clean"]) // HACK to avoid building build scripts with rustflags
        .args(args)
        .env("CARGO_TARGET_DIR", honggfuzz_target_dir) // change target_dir to not clash with regular builds
        .status()
        .unwrap();
    process::exit(status.code().unwrap_or(1));
}