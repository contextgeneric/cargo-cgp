//! Turning the wrapper's process arguments into a rustc argument vector.

use std::ffi::OsStr;
use std::path::Path;

/// Build the argument vector handed to [`rustc_driver::run_compiler`].
///
/// cargo invokes the wrapper as `cargo-cgp-driver <path-to-rustc> <rustc args...>`. Like
/// `clippy-driver`, we detect that "wrapper mode" — the second argument is a path whose
/// file stem is `rustc` — and drop the injected compiler path, leaving our own program
/// name in `args[0]` (which `run_compiler` ignores) followed by the genuine rustc flags.
///
/// We then adjust the flags in two ways:
///
/// - Inject `--sysroot <sysroot>` unless one is already present, because the driver lives
///   outside any toolchain and rustc cannot otherwise find `std`. `sysroot` is the value
///   from [`crate::config::SYSROOT_ENV`]; `sysroot_flag` is passed in rather than
///   hardcoded so the injection logic stays independent of the flag's spelling.
/// - Append each of `injected_flags` unless the invocation already sets that flag, so an
///   explicit flag on the command line wins. This is how cargo-cgp turns on the
///   next-generation trait solver ([`crate::config::NEXT_SOLVER_FLAG`]) and the diagnostic
///   `--verbose` flag ([`crate::config::VERBOSE_FLAG`]) — see those constants for why.
///
/// The injection is skipped for cargo's *info queries* — the `rustc -vV` version probe and
/// the `--print` requests cargo runs before a build. Those need no CGP handling, and one of
/// our flags actively breaks them: `-vV` already implies `-v`, so appending `--verbose` a
/// second time makes rustc reject the invocation with "given more than once". Clippy skips
/// its own added flags on the same queries, and for the same reason ([rust-lang/cargo#14385]
/// notes that a query carrying unexpected flags also poisons cargo's cache). The real crate
/// compilation — the only invocation whose diagnostics we care about — carries neither
/// marker, so it still receives every flag.
///
/// [rust-lang/cargo#14385]: https://github.com/rust-lang/cargo/issues/14385
pub fn rustc_args(
    raw_args: impl IntoIterator<Item = String>,
    sysroot: Option<String>,
    sysroot_flag: &str,
    injected_flags: &[&str],
) -> Vec<String> {
    let mut args: Vec<String> = raw_args.into_iter().collect();

    if is_wrapper_mode(&args) {
        args.remove(1);
    }

    if let Some(sysroot) = sysroot
        && !has_flag(&args, sysroot_flag)
    {
        args.push(sysroot_flag.to_owned());
        args.push(sysroot);
    }

    if !is_info_query(&args) {
        for flag in injected_flags {
            // A `-Zkey=value` flag is "already set" if any argument carries the same key, so
            // that an explicit `-Zkey=other` on the command line is left to take precedence.
            let key = flag.split('=').next().unwrap_or(flag);
            if !has_flag(&args, key) {
                args.push((*flag).to_owned());
            }
        }
    }

    args
}

/// Whether this invocation is one of cargo's pre-build info queries rather than a real
/// compilation: the `rustc -vV` version probe or any `--print` request. These carry no code
/// to type-check, so our injected flags would have no diagnostic to act on — and `--verbose`
/// would clash with the `-v` already inside `-vV`.
fn is_info_query(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "-vV" || arg == "--print" || arg.starts_with("--print="))
}

/// Whether `args[1]` is a path to `rustc`, i.e. cargo called us as its rustc wrapper.
///
/// Public because [`crate::run`] uses it to tell a cargo probe (wrapper mode, where `-vV`
/// or `--version` must reach the real compiler) from a direct `cargo-cgp-driver --version`
/// query (non-wrapper mode, which the driver answers itself — see [`crate::version`]).
pub fn is_wrapper_mode(args: &[String]) -> bool {
    args.get(1)
        .map(Path::new)
        .and_then(Path::file_stem)
        .and_then(OsStr::to_str)
        == Some("rustc")
}

/// Whether the flags already carry `key`, in either `key` or `key=value` form.
fn has_flag(args: &[String], key: &str) -> bool {
    let with_eq = format!("{key}=");
    args.iter()
        .any(|arg| arg == key || arg.starts_with(&with_eq))
}
