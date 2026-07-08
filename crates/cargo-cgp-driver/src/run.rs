//! The driver entrypoint — the function the `cargo-cgp-driver` binary calls.

use std::env;
use std::process::ExitCode;

use crate::args::rustc_args;
use crate::callbacks::CgpCallbacks;
use crate::config::{SYSROOT_ENV, SYSROOT_FLAG};

/// Run the compiler in-process over the wrapper's arguments and return the exit code.
///
/// The sysroot supplied by `cargo-cgp` (via [`SYSROOT_ENV`]) is folded into the rustc
/// arguments by [`rustc_args`]. [`rustc_driver::catch_with_exit_code`] runs the compiler
/// and turns a compiler-signalled failure into the process exit code, matching what
/// plain `rustc` would return.
pub fn run() -> ExitCode {
    let sysroot = env::var(SYSROOT_ENV).ok();
    let args = rustc_args(env::args(), sysroot, SYSROOT_FLAG);

    rustc_driver::catch_with_exit_code(|| {
        let mut callbacks = CgpCallbacks;
        rustc_driver::run_compiler(&args, &mut callbacks);
    })
}
