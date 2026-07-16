//! The driver entrypoint — the function the `cargo-cgp-driver` binary calls.

use std::env;
use std::process::ExitCode;

use crate::args::{is_wrapper_mode, rustc_args};
use crate::callbacks::CgpCallbacks;
use crate::config::{NEXT_SOLVER_FLAG, SYSROOT_ENV, SYSROOT_FLAG, VERBOSE_FLAG};
use crate::help::{help_text, wants_help};
use crate::version::{version_string, wants_version};

/// Run the compiler in-process over the wrapper's arguments and return the exit code.
///
/// The sysroot supplied by `cargo-cgp` (via [`SYSROOT_ENV`]), the next-gen trait solver
/// flag ([`NEXT_SOLVER_FLAG`]), and the diagnostic [`VERBOSE_FLAG`] are folded into the
/// rustc arguments by [`rustc_args`]. [`rustc_driver::catch_with_exit_code`] runs the
/// compiler and turns a compiler-signalled failure into the process exit code, matching
/// what plain `rustc` would return.
pub fn run() -> ExitCode {
    let raw: Vec<String> = env::args().collect();

    // A direct invocation (not cargo's wrapper call) answers the tool's own queries here,
    // before touching the compiler: `--help`/`-h` or no arguments prints the help, and
    // `--version`/`-V` prints the version handshake the front-end preflight reads. In
    // wrapper mode these flags belong to the real compiler, so we fall through.
    if !is_wrapper_mode(&raw) {
        if wants_help(&raw) || raw.len() <= 1 {
            println!("{}", help_text());
            return ExitCode::SUCCESS;
        }
        if wants_version(&raw) {
            println!("{}", version_string());
            return ExitCode::SUCCESS;
        }
    }

    let sysroot = env::var(SYSROOT_ENV).ok();
    let args = rustc_args(
        raw,
        sysroot,
        SYSROOT_FLAG,
        &[NEXT_SOLVER_FLAG, VERBOSE_FLAG],
    );

    rustc_driver::catch_with_exit_code(|| {
        let mut callbacks = CgpCallbacks;
        rustc_driver::run_compiler(&args, &mut callbacks);
    })
}
