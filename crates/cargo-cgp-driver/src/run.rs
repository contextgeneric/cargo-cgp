//! The driver entrypoint: the function the `cargo-cgp-driver` binary calls.

use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

use rustc_session::EarlyDiagCtxt;
use rustc_session::config::ErrorOutputType;

use crate::args::{ForcedFlag, expand_arg_files, force_flags, is_wrapper_mode, rustc_args};
use crate::callbacks::CgpCallbacks;
use crate::config::{
    BUG_REPORT_URL, EXPAND_FLAG, EXPAND_ITEM_FLAG, NEXT_SOLVER_FLAG, SYSROOT_ENV, SYSROOT_FLAG,
    THREAD_COUNT_OPTIONS, THREADS_FLAG, VERBOSE_FLAG,
};
use crate::emitter::install_panic_hooks;
use crate::expand::take_expand_request;
use crate::help::{help_text, wants_help};
use crate::version::{version_string, wants_version};

/// Run the compiler in-process over the wrapper's arguments and return the exit code.
///
/// The sysroot supplied by `cargo-cgp` (via [`SYSROOT_ENV`]), the next-gen trait solver
/// flag ([`NEXT_SOLVER_FLAG`]), and the diagnostic [`VERBOSE_FLAG`] are folded into the
/// rustc arguments by [`rustc_args`], and the one-thread [`THREADS_FLAG`] replaces any thread count
/// the invocation carries through [`force_flags`]. [`rustc_driver::catch_with_exit_code`] runs the
/// compiler and turns a compiler-signalled failure into the process exit code, matching
/// what plain `rustc` would return. Before it runs, the driver sets up what rustc's own
/// driver does: the `RUSTC_LOG` logger, and an ICE hook, here naming cargo-cgp's issue tracker
/// and wrapped so a panic the emitter contains is not reported (see [`install_panic_hooks`]).
pub fn run() -> ExitCode {
    // A direct invocation (not cargo's wrapper call) answers the tool's own queries here,
    // before touching the compiler: `--help`/`-h` or no arguments prints the help, and
    // `--version`/`-V` prints the version handshake the front-end preflight reads. In
    // wrapper mode these flags belong to the real compiler, so we fall through. The flags are
    // matched on a lossy copy of the arguments: a non-Unicode one matches no flag, and is
    // reported below, where the compiler reads the arguments for real.
    let direct: Vec<String> = env::args_os()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    if !is_wrapper_mode(&direct) {
        if wants_help(&direct) || direct.len() <= 1 {
            return print_answer(&help_text());
        }
        if wants_version(&direct) {
            return print_answer(&version_string());
        }
    }

    let early_dcx = EarlyDiagCtxt::new(ErrorOutputType::default());
    rustc_driver::init_rustc_env_logger(&early_dcx);

    // Read the arguments as rustc reads them, which reports a non-Unicode one as an error where
    // `env::args` would panic. This comes before the panic hooks, because rustc's ICE hook reads
    // `env::args` itself when `RUST_BACKTRACE` is unset, and would panic on such an argument.
    let Ok(raw) = rustc_driver::catch_fatal_errors(|| rustc_driver::args::raw_args(&early_dcx))
    else {
        return ExitCode::FAILURE;
    };
    install_panic_hooks(BUG_REPORT_URL);

    rustc_driver::catch_with_exit_code(|| {
        // Cargo may move all but the wrapper and the `rustc` path into an `@file`, so it is
        // expanded before the driver reads its own flags or looks for the user's. Inside the catch,
        // so a file that cannot be read ends the compilation as rustc's own fatal error would.
        let mut raw = expand_arg_files(raw);

        // Expand mode is requested by a flag of ours rather than an environment variable, so cargo
        // scopes it to the one target the user asked about. It is not a rustc flag, so it is taken
        // out of the vector here, before the arguments are prepared.
        let expand = take_expand_request(&mut raw, EXPAND_FLAG, EXPAND_ITEM_FLAG);

        // The injected flags shape diagnostics produced during analysis, which expand mode never
        // reaches (it stops once the crate is expanded), so they are left off there.
        let (injected_flags, forced_flags): (&[&str], &[ForcedFlag<'_>]) = if expand.is_some() {
            (&[], &[])
        } else {
            (
                &[NEXT_SOLVER_FLAG, VERBOSE_FLAG],
                &[ForcedFlag {
                    flag: THREADS_FLAG,
                    replaces: THREAD_COUNT_OPTIONS,
                }],
            )
        };

        let sysroot = env::var(SYSROOT_ENV).ok();
        let args = force_flags(
            rustc_args(raw, sysroot, SYSROOT_FLAG, injected_flags),
            forced_flags,
        );

        let mut callbacks = CgpCallbacks { expand };
        rustc_driver::compiler_entrypoint(&args, &mut callbacks);
    })
}

/// Print a direct query's answer and return the exit code. A reader that closed the pipe, as
/// `head` does once it has enough, got what it wanted, so that ends the driver successfully, as it
/// ends the front-end's `expand`; any other write error is a failure.
fn print_answer(text: &str) -> ExitCode {
    match writeln!(io::stdout(), "{text}") {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
