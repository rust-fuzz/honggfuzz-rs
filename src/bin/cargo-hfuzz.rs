
#[cfg(target_family="windows")]
compile_error!("honggfuzz-rs does not currently support Windows but works well under WSL (Windows Subsystem for Linux)");

#[cfg(feature = "cli")]
mod hfuzz;
#[cfg(not(feature = "cli"))]
mod hfuzz {
    pub fn main() {
        compile_error!("to build the `cargo-hfuzz` command-line tool, enable the `cli` feature with `--features=cli`");
    }
}

fn main() {
    hfuzz::main();
}