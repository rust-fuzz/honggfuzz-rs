use std::env;
use std::process::{self, Command};
use std::os::unix::process::CommandExt;

fn target_triple() -> &'static str {
	if !cfg!(target_arch="x86_64") {
		unimplemented!()
	}

	if cfg!(target_os="linux") {
		"x86_64-unknown-linux-gnu"
	} else if cfg!(target_os="macos") {
	    "x86_64-apple-darwin"
	} else {
		unimplemented!()
	}
}

fn main() {
	let mut args = env::args().skip(1).peekable();
	assert!(args.next() == Some("honggfuzz".to_string()), "please launch using \"cargo honggfuzz ... \"");

	let honggfuzz_target_dir = "fuzzing_target";
	if args.peek() == Some(&"build".to_string()) {
		args.next();
		let rustflags = env::var("RUSTFLAGS").unwrap_or_default();
		let rustflags = format!("{} \
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
		", rustflags);

		let cargo_bin = env::var("CARGO").unwrap();
		let status = Command::new(cargo_bin)
	        .args(&["build", "--target", target_triple()]) // HACK to avoid building build scripts with rustflags
	        .args(args)
	        .env("RUSTFLAGS", rustflags)
	        .env("CARGO_TARGET_DIR", honggfuzz_target_dir)
	        .env("CARGO_HONGGFUZZ_TARGET_DIR", honggfuzz_target_dir)
	        .status()
	        .unwrap();
	    process::exit(status.code().unwrap_or(1));
	} else {
		let asan_options = env::var("ASAN_OPTIONS").unwrap_or_default();
		let asan_options = format!("{}:detect_odr_violation=0", asan_options);

		let tsan_options = env::var("TSAN_OPTIONS").unwrap_or_default();
		let tsan_options = format!("{}:report_signal_unsafe=0", tsan_options);

		let command = format!("{}/honggfuzz", honggfuzz_target_dir);
		Command::new(&command)
	        .args(args)
	        .env("ASAN_OPTIONS", asan_options)
	        .env("TSAN_OPTIONS", tsan_options)
	        .exec();
	    println!("cannot execute {}, try to execute \"cargo honggfuzz build\" first", &command);
	}
}