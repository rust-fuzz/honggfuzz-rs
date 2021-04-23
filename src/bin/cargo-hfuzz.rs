use anyhow::Result;
use docopt::Docopt;
use fs_err as fs;
use serde::Deserialize;
use std::env;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const HONGGFUZZ_TARGET: &str = "hfuzz_target";
const HONGGFUZZ_WORKSPACE: &str = "hfuzz_workspace";

#[cfg(target_family="windows")]
compile_error!("honggfuzz-rs does not currently support Windows but works well under WSL (Windows Subsystem for Linux)");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuildType {
    ReleaseInstrumented,
    ReleaseNotInstrumented,
    ProfileWithGrcov,
    Debug
}


#[inline(always)]
fn target_triple() -> Result<String> {
    Ok(rustc_version::version_meta()?.host)
}

fn find_crate_root() -> Result<PathBuf> {
    let path = env::current_dir()
        .map_err(|e| anyhow::anyhow!("Current directory is not set for process.").context(e))?;
    let mut path = path.as_path();
    while !path.join("Cargo.toml").is_file() {
        // move to parent path
        path = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Reached root without finding Cargo.toml"))?;
    }

    Ok(path.to_path_buf())
}

fn debugger_command(target: &str, triple: &str) -> Command {
    let debugger = env::var("HFUZZ_DEBUGGER").unwrap_or_else(|_| "rust-lldb".into());
    let honggfuzz_target = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| HONGGFUZZ_TARGET.into());

    let mut cmd = Command::new(&debugger);

    let dest = format!("{}/{}/debug/{}", &honggfuzz_target, triple, target);
    match Path::new(&debugger)
        .file_name()
        .map(|f| f.to_string_lossy().contains("lldb"))
    {
        Some(true) => {
            cmd.args(&["-o", "b rust_panic", "-o", "r", "-o", "bt", "-f", &dest, "--"]);
        }
        _ => {
            cmd.args(&["-ex", "b rust_panic", "-ex", "r", "-ex", "bt", "--args", &dest]);
        }
    };

    cmd
}

fn hfuzz_version() {
    println!("cargo-hfuzz {}", VERSION);
}

fn hfuzz_run(args: Args, crate_root: &Path, build_type: BuildType) -> Result<()> {
    let target = args
        .arg_target
        .as_ref()
        .expect("Docopt USAGE meta definition guarantees target is set. qed");

    let honggfuzz_target = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| HONGGFUZZ_TARGET.into());
    let honggfuzz_workspace = env::var("HFUZZ_WORKSPACE").unwrap_or_else(|_| HONGGFUZZ_WORKSPACE.into());
    let honggfuzz_input = env::var("HFUZZ_INPUT").unwrap_or_else(|_| format!("{}/{}/input", honggfuzz_workspace, target));

    {
        let mut args = args.clone();
        let extra = vec!["--bin".to_owned(), target.clone()];
        args.arg_sub = extra.into();
        hfuzz_build(args, crate_root, build_type)?;
    }

    let triple = target_triple()?;
    match build_type {
        BuildType::Debug => {
            let crash_filename = args
                .arg_crash_filename
                .expect("Guaranteed by docopt meta desc. qed");

            let status = debugger_command(&target, &triple)
                .args(args.arg_sub)
                .env("CARGO_HONGGFUZZ_CRASH_FILENAME", crash_filename)
                .env("RUST_BACKTRACE", env::var("RUST_BACKTRACE").unwrap_or_else(|_| "1".into()))
                .status()?;
            if !status.success() {
                anyhow::bail!("Process did exit with code: {}", status.code().unwrap_or(1));
            }
        }
        _ => {
            // add some flags to sanitizers to make them work with Rust code
            let asan_options = env::var("ASAN_OPTIONS").unwrap_or_default();
            let asan_options = "detect_odr_violation=0:".to_owned() + asan_options.as_str();

            let tsan_options = env::var("TSAN_OPTIONS").unwrap_or_default();
            let tsan_options = "report_signal_unsafe=0:".to_owned() + tsan_options.as_str();

            // get user-defined args for honggfuzz
            let hfuzz_run_args = env::var("HFUZZ_RUN_ARGS").unwrap_or_default();
            log::debug!("HFUZZ_RUN_ARGS: {}", hfuzz_run_args);

            // FIXME: we split by whitespace without respecting escaping or quotes
            let hfuzz_run_args = hfuzz_run_args.split_whitespace();

            fs::create_dir_all(&format!("{}/{}/input", &honggfuzz_workspace, target))?;

            let command = format!("{}/honggfuzz", &honggfuzz_target);

            let mut arguments = vec!["-W".to_owned(), format!("{}/{}", &honggfuzz_workspace, target), "-f".to_owned(), honggfuzz_input.to_owned(), "-P".to_owned()];
            arguments.extend(hfuzz_run_args.map(ToString::to_string));
            arguments.extend(args.arg_sub.into_iter());

            // exec honggfuzz replacing current process
            let mut cmd = Command::new(&command);
            cmd
                .env("ASAN_OPTIONS", asan_options)
                .env("TSAN_OPTIONS", tsan_options);
            if let Some(timeout) = args.flag_timeout {
                arguments.extend(vec!["-t".to_owned(), timeout.to_string()]);
            }
            if let Some(n) = args.flag_iterations {
                arguments.extend(vec!["-N".to_owned(), n.to_string()]);
            }
            if args.flag_quiet {
                arguments.push("--quiet".to_owned());
            }
            if args.flag_verbose > 0 {
                arguments.push("--verbose".to_owned());
            }
            if let Some(exitcode) = args.flag_exit_upon_crash {
                arguments.push("--exit_upon_crash".to_owned());
                arguments.push("--exit_code_upon_crash".to_owned());
                arguments.push(exitcode.to_string());
            }
            arguments.extend(["--", &format!("{}/{}/release/{}", &honggfuzz_target, triple, target)].iter().map(ToString::to_string));

            log::debug!("Exec: {} {}", &command, arguments.join(" "));

            cmd.args(arguments).exec();

            anyhow::bail!("Failed to execute {} \"cargo hfuzz build\" from fuzzed project directory", &command)
        }
    }
    Ok(())
}

fn hfuzz_build(args: Args, crate_root: &Path, build_type: BuildType) -> Result<()> {
    let honggfuzz_target = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| HONGGFUZZ_TARGET.into());

    let mut rustflags = "\
    --cfg fuzzing \
    -C debug-assertions \
    -C overflow_checks \
    ".to_string();

    let mut cargo_incremental = "1";
    match build_type {
        BuildType::Debug => {
            rustflags.push_str("\
            --cfg fuzzing_debug \
            -C opt-level=0 \
            -C debuginfo=2 \
            ");
        }

        BuildType::ProfileWithGrcov => {
            rustflags.push_str("\
            --cfg fuzzing_debug \
            -Zprofile \
            -Cpanic=abort \
            -C opt-level=0 \
            -C debuginfo=2 \
            -Ccodegen-units=1 \
            -Cinline-threshold=0 \
            -Clink-dead-code \
            ");
            //-Coverflow-checks=off \
            cargo_incremental = "0";
        }

        _ => {
            rustflags.push_str("\
            -C opt-level=3 \
            -C target-cpu=native \
            -C debuginfo=0 \
            ");

            if build_type == BuildType::ReleaseInstrumented {
                rustflags.push_str("\
                -C passes=sancov \
                -C llvm-args=-sanitizer-coverage-level=4 \
                -C llvm-args=-sanitizer-coverage-trace-pc-guard \
                -C llvm-args=-sanitizer-coverage-trace-divs \
                ");

                // trace-compares doesn't work on macOS without a sanitizer
                if cfg!(not(target_os="macos")) {
                    rustflags.push_str("\
                    -C llvm-args=-sanitizer-coverage-trace-compares \
                    ");
                }

                // HACK: temporary fix, see https://github.com/rust-lang/rust/issues/53945#issuecomment-426824324
                // HACK: check if the gold linker is available
                if which::which("ld.gold").is_ok() {
                    rustflags.push_str("-Clink-arg=-fuse-ld=gold ");
                }
            }
        }
    }

    // add user provided flags
    rustflags.push_str(&env::var("RUSTFLAGS").unwrap_or_default());

    // get user-defined args for building
    let hfuzz_build_args = env::var("HFUZZ_BUILD_ARGS").unwrap_or_default();
    log::debug!("HFUZZ_BUILD_ARGS: {}", hfuzz_build_args);

    // FIXME: we split by whitespace without respecting escaping or quotes
    let hfuzz_build_args = hfuzz_build_args.split_whitespace();

    let cargo_bin = env::var("CARGO").unwrap();
    let mut command = Command::new(&cargo_bin);
    // HACK to avoid building build scripts with rustflags
    let mut arguments = vec!["build".to_owned(), "--target".to_owned(), target_triple()?];
    arguments.extend(hfuzz_build_args.map(ToString::to_string));
    arguments.extend(args.arg_sub.iter().map(ToString::to_string));

    log::debug!("Spawn: {} {}", &cargo_bin, arguments.join(" "));

    command
        .env("RUSTFLAGS", rustflags)
        .env("CARGO_INCREMENTAL", cargo_incremental)
        .env("CARGO_TARGET_DIR", &honggfuzz_target) // change target_dir to not clash with regular builds
        .env("CRATE_ROOT", &crate_root);

    // used by build.rs to check that versions are in sync
    // env variable to be read by build.rs script
    // to place honggfuzz executable at a known location
    if build_type == BuildType::ProfileWithGrcov {
        command
            .env("CARGO_HONGGFUZZ_BUILD_VERSION", VERSION)
            .env("CARGO_HONGGFUZZ_TARGET_DIR", &honggfuzz_target);
    }
    else if build_type != BuildType::Debug {
        command
            .env("CARGO_HONGGFUZZ_BUILD_VERSION", VERSION)
            .env("CARGO_HONGGFUZZ_TARGET_DIR", &honggfuzz_target);
        arguments.push("--release".to_owned());
    }

    let status = command.args(arguments).status()?;
    if !status.success() {
        anyhow::bail!("Execution failed with status code {:?}", status.code());
    }
    Ok(())
}

fn hfuzz_clean(args: Args) -> Result<()> {
    let honggfuzz_target = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| HONGGFUZZ_TARGET.into());
    let cargo_bin = env::var("CARGO").unwrap();
    let status = Command::new(cargo_bin)
        .args(&["clean"])
        .args(args.arg_sub)
        .env("CARGO_TARGET_DIR", &honggfuzz_target) // change target_dir to not clash with regular builds
        .status()?;

    if !status.success() {
        anyhow::bail!("Process execution completed with exit code {:?}", status.code())
    }

    Ok(())
}



const USAGE: &str = r#"
cargo-hfuzz

Usage:
  cargo-hfuzz (build|build-debug|build-no-instr|build-grcov) [(-v...|-q)] [<target>] [[--] <sub>...]
  cargo-hfuzz (run|run-no-instr) [(-v...|-q)] [--iterations=<n>] [--timeout=<sec>] [--exit-upon-crash=<exitcode>] <target> [[--] <sub>...]
  cargo-hfuzz (run-debug) [(-v...|-q)] <target> <crash_filename> [[--] <sub>...]
  cargo-hfuzz clean [(-v...|-q)]
  cargo-hfuzz (-h | --help | help)
  cargo-hfuzz (--version | version)

Options:
  <target>                      The particular cargo target binary to use for fuzzing.
  <crash_filename>              A particular crash dump to use for debuging.
  <sub>...                      Additional arguments passed to the sub process.

  -v --verbose                  Pass --verbose to honggfuzz, enable various log levels.
  -q --quiet                    Silence.
  --exit-upon-crash=<exitcode>  Exit upon the first crash with non-zero exit code.
  -t --timeout=<sec>            Total time allowed to fuzz, in seconds.
  -N --iterations=<n>           Total of fuzzing iterations to run.
  -h --help                     Show this screen.
  --version                     Show version.
"#;


#[derive(Debug, Deserialize, Clone)]
pub(crate) struct Args {
    flag_version: bool,
    flag_help: bool,
    cmd_version: bool, // backwards compat
    cmd_help: bool,    // backwards compat
    cmd_build_debug: bool,
    cmd_build_no_instr: bool,
    cmd_build: bool,
    cmd_build_grcov: bool,
    cmd_run_debug: bool,
    cmd_run_no_instr: bool,
    cmd_run: bool,
    cmd_clean: bool,
    flag_iterations: Option<u64>,
    flag_timeout: Option<u64>,
    flag_verbose: usize,
    flag_quiet: bool,
    flag_exit_upon_crash: Option<usize>,
    arg_target: Option<String>,
    arg_crash_filename: Option<String>,
    // the remaining arguments
    arg_sub: Vec<String>,
}

impl Args {
    pub fn parse<S, I>(argv_iter: I) -> Result<Self, docopt::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        const CARGO_HFUZZ: &str = "cargo-hfuzz";
        const CARGO: &str = "cargo";
        const HFUZZ: &str = "hfuzz";

        Docopt::new(USAGE).and_then(|d| {
            // if ends with file name CARGO_HFUZZ
            let mut argv_iter = argv_iter.into_iter();
            if let Some(arg0) = argv_iter.next() {
                let arg0 = arg0.as_ref();
                match PathBuf::from(arg0)
                    .file_name()
                    .map(|x| x.to_str())
                    .flatten()
                {
                    Some(file_name) => {

                        // allow all variants to be parsed
                        // cargo hfuzz ...
                        // cargo-hfuzz ...
                        //
                        // so preprocess them to unified `cargo-spellcheck`
                        let mut next = vec![CARGO_HFUZZ.to_owned()];

                        match argv_iter.next() {
                            Some(arg) if file_name.starts_with(CARGO_HFUZZ) && arg.as_ref() == HFUZZ => {
                                // drop the first arg HFUZZ`
                            }
                            Some(arg) if file_name.starts_with(CARGO) && arg.as_ref() == HFUZZ => {
                                // drop it, we replace it with CARGO_HFUZZ
                            }
                            Some(arg) if arg.as_ref() == HFUZZ => {
                                // HFUZZ but the binary got renamed
                                // drop the HFUZZ part
                            }
                            Some(arg) => {
                                // not HFUZZ so retain it
                                next.push(arg.as_ref().to_owned())
                            }
                            None => {}
                        };
                        let collected = next.into_iter().chain(argv_iter.map(|s| s.as_ref().to_owned()));
                        d.argv(collected)
                    }
                    _ => d,
                }
            } else {
                d
            }
            .deserialize()
        })
    }

    fn action(&self) -> Action {
        if self.cmd_build {
            Action::Build(BuildType::ReleaseInstrumented)
        } else if self.cmd_build_no_instr {
            Action::Build(BuildType::ReleaseNotInstrumented)
        } else if self.cmd_build_debug {
            Action::Build(BuildType::Debug)
        } else if self.cmd_build_grcov {
            Action::Build(BuildType::ProfileWithGrcov)
        } else if self.cmd_run {
            Action::Run(BuildType::ReleaseInstrumented)
        } else if self.cmd_run_no_instr {
            Action::Run(BuildType::ReleaseNotInstrumented)
        } else if self.cmd_run_debug {
            Action::Run(BuildType::Debug)
        } else if self.cmd_version || self.flag_version {
            Action::Version
        } else if self.cmd_clean {
            Action::Clean
        } else {
            Action::Help
        }
    }

    fn verbosity(&self) -> log::LevelFilter {
        match self.flag_verbose {
            _ if self.flag_quiet => log::LevelFilter::Off,
            2 => log::LevelFilter::Warn,
            3 => log::LevelFilter::Info,
            4 => log::LevelFilter::Debug,
            n if n > 4 => log::LevelFilter::Trace,
            _ => log::LevelFilter::Error,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Action {
    Build(BuildType),
    Run(BuildType),
    Version,
    Clean,
    Help,
}

fn main() -> Result<()> {
    let args = Args::parse(env::args())?;
    pretty_env_logger::formatted_timed_builder()
        .filter_level(args.verbosity())
        .init();

    // change to crate root to have the same behavior as cargo build/run
    let crate_root = find_crate_root().map_err(|e| {
        e.context(anyhow::anyhow!("could not find `Cargo.toml` in current directory or any parent directory"))
    })?;
    env::set_current_dir(&crate_root).unwrap();

    match args.action() {
        Action::Build(ty) => hfuzz_build(args, &crate_root, ty)?,
        Action::Run(ty) => hfuzz_run(args, &crate_root, ty)?,
        Action::Clean => hfuzz_clean(args)?,
        Action::Version => hfuzz_version(),
        Action::Help => println!("{}", USAGE),
    };
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn args() {
        fn check(cl: &'static str) -> Args {
            let args = Args::parse(
                cl.split_ascii_whitespace()
            ).expect("Must parse. qed ");
            args
        }


        assert_matches!(
            check("cargo hfuzz run -vvv some-binary --exit-upon-crash=77 -- fff --xyz"),
            Args {
                cmd_run,
                flag_verbose,
                arg_target,
                flag_exit_upon_crash,
                arg_sub,
                ..
            } => {
                assert!(cmd_run);
                assert_eq!(flag_verbose, 3);
                assert_eq!(arg_target, Some("some-binary".to_owned()));
                assert_eq!(flag_exit_upon_crash, Some(77));
                assert_eq!(arg_sub.as_slice(), &["fff", "--xyz"]);
            });


        assert_matches!(
            check("cargo hfuzz run -q foo --exit-upon-crash=0 -- --xyz"),
            Args {
                cmd_run,
                flag_verbose,
                flag_quiet,
                arg_target,
                flag_exit_upon_crash,
                arg_sub,
                ..
            } => {
                assert!(cmd_run);
                assert_eq!(flag_verbose, 0);
                assert!(flag_quiet);
                assert_eq!(arg_target, Some("foo".to_owned()));
                assert_eq!(flag_exit_upon_crash, Some(0));
                assert_eq!(arg_sub.as_slice(), ["--xyz"]);
            });
    }
}
