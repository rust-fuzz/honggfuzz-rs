use std::fs;
use std::env;
use std::process::{self, Command};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use anyhow::Result;
use std::time::Duration;
use structopt::StructOpt;

/// The version of `cargo-hfuzz` cli tooling.
const VERSION: &str = env!("CARGO_PKG_VERSION");
const HONGGFUZZ_TARGET: &str = "hfuzz_target";
const HONGGFUZZ_WORKSPACE: &str = "hfuzz_workspace";

#[cfg(target_family="windows")]
compile_error!("honggfuzz-rs does not currently support Windows but works well under WSL (Windows Subsystem for Linux)");

#[derive(Debug, StructOpt)]
#[structopt(name = "cargo-hfuzz", about = "Fuzz your Rust code with Google-developed Honggfuzz !")]
struct Opt {
    #[structopt(subcommand)]
    command: SubCommand,
}

impl Opt {
    fn verbosity(&self) -> log::LevelFilter {
        self.command.verbosity()
    }
}
/// Shared options for multiple sub-commands.
#[derive(Debug, StructOpt)]
struct CommonOpts {
    /// only build binary but don't execute it
    #[structopt(long)]
    only_build: bool,

    /// flags given to `rustc`, for example "-Z sanitizer=address"
    #[structopt(long, env = "RUSTFLAGS")]
    rustflags: Option<String>,

    /// args given to `cargo build`
    #[structopt(long, env = "HFUZZ_BUILD_ARGS")]
    build_args: Option<String>,

    /// path to working directory
    #[structopt(short, long, default_value = "hfuzz_workspace", env = "HFUZZ_WORKSPACE")]
    workspace: String,
}

#[derive(Debug, StructOpt)]
enum SubCommand {
    /// build and run fuzzing
    Fuzz {

        #[structopt(flatten)]
        common: CommonOpts,

        /// path to fuzzer's input files (aka "corpus"), relative to `$HFUZZ_WORKSPACE/{TARGET}`
        #[structopt(short, long, default_value = "input", env = "HFUZZ_INPUT")]
        input: String,

        /// which fuzzing target binary to fuzz
        #[structopt(short="b", long="bin")]
        binary: String,

        /// do no build with compiler instrumentation
        #[structopt(long)]
        no_instr: bool,

        /// use grcov coverage information
        #[structopt(long)]
        grcov: bool,

        #[structopt(flatten)]
        launch: HonggfuzzLaunchArgs,

        /// args to the binary, followed by an optional `--` which are interpreted by the fuzzer itself
        /// ( https://github.com/google/honggfuzz/blob/master/docs/USAGE.md )
        args: Vec<String>,
    },

    /// Debug
    Debug {
        #[structopt(flatten)]
        common: CommonOpts,

        /// name or path to debugger, like `rust-gdb`, `gdb`, `/usr/bin/lldb-7`..
        #[structopt(short, long, default_value = "rust-lldb", env = "HFUZZ_DEBUGGER")]
        debugger: String,

        /// which binary target to fuzz
        #[structopt(short, long)]
        binary: String,

        /// path to crash file, typically like `hfuzz_workspace/[TARGET]/[..].fuzz`
        #[structopt(short, long)]
        crash_file: PathBuf,

        /// args to target
        target_args: Vec<String>,
    },


    /// Minimize
    Minimize,
    /// Clean the saved fuzzing state and all related files.
    Clean { args: Vec<String> },
}

impl SubCommand {
    pub fn verbosity(&self) -> log::LevelFilter {
        log::LevelFilter::Trace
    }
}

impl SubCommand {
    pub fn launch(self, crate_root: impl AsRef<Path>) -> Result<()> {
        let crate_root = crate_root.as_ref();
        let target_triple = target_triple()?;
        let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| HONGGFUZZ_TARGET.into());

        match self {
            Self::Clean { args }  => {
                hfuzz_clean( args, &target_dir )?;
            }
            Self::Minimize => {
                // https://github.com/rust-fuzz/honggfuzz-rs/issues/26
                todo!(" --minimize --input .. --output ..")
            }
            Self::Fuzz { launch, common, no_instr, binary, input, grcov, args } => {
                let build_type = if no_instr {
                    BuildType::ReleaseNotInstrumented
                } else if grcov {
                    // grcov and instrumentation are mutually exclusive,
                    // only due to the fact, grcov is used in debug mode
                    // where instrumentation is commonly used in release
                    // mode.
                    BuildType::ProfileWithGrcov
                } else {
                    BuildType::ReleaseInstrumented
                };

                // FIXME split args in cargo build args and target args

                let mut args = args.into_iter();
                let build_args = (&mut args).take_while(|arg| arg != "--").collect::<Vec<_>>();
                let target_args = args.collect::<Vec<_>>();
                let rustflags = common.rustflags.as_ref().map(|x| x.as_ref()).unwrap_or_default();
                let workspace = common.workspace;
                hfuzz_build(&binary, rustflags, build_args, &crate_root, build_type, &workspace, &target_dir)?;
                if common.only_build {
                    return Ok(())
                }
                hfuzz_run(launch, &target_triple, &binary, target_args, &crate_root, build_type, &workspace, &target_dir)?;
            }
            Self::Debug { common, binary, target_args, crash_file, debugger } => {
                let build_type = BuildType::Debug;

                let rustflags = common.rustflags.as_ref().map(|x| x.as_ref()).unwrap_or_default();
                let workspace = common.workspace;

                hfuzz_build(&binary, rustflags, common.build_args, &crate_root, build_type, &workspace, &target_dir);

                if common.only_build {
                    return Ok(())
                }
                let status = debugger_command(&binary.to_string(), &target_dir, &target_triple, &debugger)
                    .args(target_args)
                    .env("CARGO_HONGGFUZZ_CRASH_FILENAME", crash_file)
                    .env("RUST_BACKTRACE", env::var("RUST_BACKTRACE").unwrap_or_else(|_| "1".into()))
                    .status()?;

                if !status.success() {
                     process::exit(status.code().unwrap_or(1));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct TimeoutDuration(Duration);

impl std::str::FromStr for TimeoutDuration {
    type Err = <u64 as std::str::FromStr>::Err;
    fn from_str(s: &str) ->  Result<Self, Self::Err> {
        Ok(Self(Duration::from_secs(u64::from_str(s)?)))
    }
}

impl TimeoutDuration {
    fn as_secs(&self) -> u64 {
        self.0.as_secs() as _
    }
}

#[derive(Debug, Clone, Default, StructOpt)]
struct HonggfuzzLaunchArgs {
    #[structopt(long)]
    timeout: Option<TimeoutDuration>,
    #[structopt(long)]
    exit_upon_crash: Option<bool>,
    #[structopt(long)]
    n_iterations: Option<u64>,
    #[structopt(short, long)]
    quiet: bool,
}

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

fn debugger_command(binary: &str, target_dir: &str, target_triple: &str, debugger: &str) -> Command {
    let honggfuzz_target = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| HONGGFUZZ_TARGET.into());

    let mut cmd = Command::new(&debugger);

    let dest = format!("{}/{}/debug/{}", &target_dir, target_triple, binary);
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



fn hfuzz_run(
    launch: HonggfuzzLaunchArgs,
    target_triple: &str,
    binary: &str,
    args: impl IntoIterator<Item = impl ToString>,
    crate_root: &Path,
    build_type: BuildType,
    workspace: &str,
    target_dir: &str,
) -> Result<()> {

    let honggfuzz_input = env::var("HFUZZ_INPUT").unwrap_or_else(|_| format!("{}/{}/input", workspace, binary.to_string()));

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

    fs::create_dir_all(&format!("{}/{}/input", &workspace, binary.to_string()))?;

    let command = format!("{}/honggfuzz", &target_dir);

    let mut arguments: Vec<String> = vec![
        "-W".to_owned(),
        format!("{}/{}", &workspace, binary.to_string()),
        "-f".to_owned(),
        honggfuzz_input.to_owned(),
        "-P".to_owned()
    ];
    arguments.extend(hfuzz_run_args.map(|x| x.to_string()));
    arguments.extend(args.into_iter().map(|x| x.to_string()));

    // exec honggfuzz replacing current process
    let mut cmd = Command::new(&command);
    cmd
        .env("ASAN_OPTIONS", asan_options)
        .env("TSAN_OPTIONS", tsan_options);
    if let Some(timeout) = launch.timeout {
        arguments.extend(vec!["-t".to_owned(), timeout.as_secs().to_string() ]);
    }
    if let Some(n) = launch.n_iterations {
        arguments.extend(vec!["-N".to_owned(), n.to_string()]);
    }
    if launch.quiet {
        arguments.push("--quiet".to_owned());
    }
    // if launch.verbose > 0 {
    //     arguments.push("--verbose".to_owned());
    // }
    if let Some(exitcode) = launch.exit_upon_crash {
        arguments.push("--exit_upon_crash".to_owned());
        arguments.push("--exit_code_upon_crash".to_owned());
        arguments.push(exitcode.to_string());
    }
    arguments.extend(
        ["--", &format!("{}/{}/release/{}", &target_dir, target_triple, &binary)]
        .iter()
        .map(ToString::to_string));

    log::debug!("Exec: {} {}", &command, arguments.join(" "));

    cmd.args(arguments).exec();

    anyhow::bail!("Failed to execute {} \"cargo hfuzz build\" from fuzzed project directory", &command)
}

fn hfuzz_build(
    binary: &str,
    extra_rustflags: &str,
    args: impl IntoIterator<Item = impl ToString>,
    crate_root: &Path,
    build_type: BuildType,
    workspace: &str,
    target_dir: &str) -> Result<()> {

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
    rustflags.push_str(extra_rustflags);

    // get user-defined args for building
    let hfuzz_build_args = env::var("HFUZZ_BUILD_ARGS").unwrap_or_default();
    log::debug!("HFUZZ_BUILD_ARGS: {}", hfuzz_build_args);

    // FIXME: we split by whitespace without respecting escaping or quotes
    let hfuzz_build_args = hfuzz_build_args.split_whitespace();

    let cargo_bin = env::var("CARGO").unwrap();
    let mut command = Command::new(&cargo_bin);
    // HACK to avoid building build scripts with rustflags
    let mut arguments = vec![
        "build".to_owned(),
        "--bin".to_owned(),
        binary.to_string(),
        "--target".to_owned(),
        target_triple()?
    ];
    arguments.extend(hfuzz_build_args.map(|x| x.to_string()));
    arguments.extend(args.into_iter().map(|x| x.to_string()));

    log::debug!("Spawn: {} {}", &cargo_bin, arguments.join(" "));

    command
        .env("RUSTFLAGS", rustflags)
        .env("CARGO_INCREMENTAL", cargo_incremental)
        .env("CARGO_TARGET_DIR", &target_dir) // change target_dir to not clash with regular builds
        .env("CRATE_ROOT", &crate_root);

    // used by build.rs to check that versions are in sync
    // env variable to be read by build.rs script
    // to place honggfuzz executable at a known location
    if build_type == BuildType::ProfileWithGrcov {
        command
            .env("CARGO_HONGGFUZZ_BUILD_VERSION", VERSION)
            .env("CARGO_HONGGFUZZ_TARGET_DIR", &target_dir);
    }
    else if build_type != BuildType::Debug {
        command
            .env("CARGO_HONGGFUZZ_BUILD_VERSION", VERSION)
            .env("CARGO_HONGGFUZZ_TARGET_DIR", &target_dir);
        arguments.push("--release".to_owned());
    }

    let status = command.args(arguments).status()?;
    if !status.success() {
        anyhow::bail!("Execution failed with status code {:?}", status.code());
    }
    Ok(())
}

fn hfuzz_clean(args: impl IntoIterator<Item = impl ToString>, target_dir: &str) -> Result<()> {
    let cargo_bin = env::var("CARGO").unwrap();
    let status = Command::new(cargo_bin)
        .args(&["clean"])
        .args(args.into_iter().map(|x| x.to_string()))
        .env("CARGO_TARGET_DIR", target_dir) // change target_dir to not clash with regular builds
        .status()?;

    if !status.success() {
        anyhow::bail!("Process execution completed with exit code {:?}", status.code())
    }

    Ok(())
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    pretty_env_logger::formatted_timed_builder()
        .filter_level(opt.verbosity())
        .init();

    // change to crate root to have the same behavior as cargo build/run
    let crate_root = find_crate_root().map_err(|e| {
        e.context(anyhow::anyhow!("could not find `Cargo.toml` in current directory or any parent directory"))
    })?;
    env::set_current_dir(&crate_root).unwrap();

    opt.command.launch(crate_root)?;
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
