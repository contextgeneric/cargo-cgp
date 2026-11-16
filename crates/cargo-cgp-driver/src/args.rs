//! Turning the wrapper's process arguments into a rustc argument vector.

use std::ffi::OsStr;
use std::path::Path;

use rustc_session::EarlyDiagCtxt;
use rustc_session::config::ErrorOutputType;

use crate::config::{VERBOSE_FLAG, VERBOSE_SHORT_FLAG};

/// Replace each `@file` argument with the arguments the file holds, one per line, exactly as rustc
/// does when it starts ([`rustc_driver::args::arg_expand_all`]).
///
/// Cargo moves a long command line into such a file, keeping only the wrapper and the `rustc` path
/// visible, so the driver's own flags (`--cgp-expand=`) and a user's explicit choices
/// (`-Znext-solver`, `--verbose`, `--sysroot`) may sit inside it. Expanding first lets the driver
/// read them; rustc then receives the arguments already expanded. A file that cannot be read is a
/// fatal error, as it is for rustc.
pub fn expand_arg_files(raw: Vec<String>) -> Vec<String> {
    let early_dcx = EarlyDiagCtxt::new(ErrorOutputType::default());
    rustc_driver::args::arg_expand_all(&early_dcx, &raw)
}

/// Build the argument vector handed to [`rustc_driver::compiler_entrypoint`]: drop the compiler
/// path cargo passes in wrapper mode, inject `--sysroot` from `sysroot` and each of
/// `injected_flags` ([`crate::config::NEXT_SOLVER_FLAG`], [`crate::config::VERBOSE_FLAG`]) unless
/// the invocation already sets it, and skip the injection for cargo's `-vV` and `--print` info
/// queries.
///
/// See cgp-knowledge-base/cargo-cgp/implementation/driver/program.md for the order of the steps and
/// why the info queries take no flags.
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
/// to type-check, so our injected flags would have no diagnostic to act on, and `--verbose`
/// would clash with the `-v` already inside `-vV`.
fn is_info_query(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "-vV" || arg == "--print" || arg.starts_with("--print="))
}

/// Whether `args[1]` is a path to `rustc`, i.e. cargo called us as its rustc wrapper.
///
/// Public because [`crate::run`] uses it to tell a cargo probe (wrapper mode, where `-vV`
/// or `--version` must reach the real compiler) from a direct `cargo-cgp-driver --version`
/// query (non-wrapper mode, which the driver answers itself; see [`crate::version`]).
pub fn is_wrapper_mode(args: &[String]) -> bool {
    args.get(1)
        .map(Path::new)
        .and_then(Path::file_stem)
        .and_then(OsStr::to_str)
        == Some("rustc")
}

/// Whether the flags already carry `key`, in any spelling rustc accepts for it: `key` or
/// `key=value`; for a `-Z` option (`-Znext-solver`), also the two-argument `-Z next-solver=value`;
/// and for `--verbose`, its short form `-v`.
fn has_flag(args: &[String], key: &str) -> bool {
    let matches = |arg: &str, key: &str| {
        arg == key
            || arg
                .strip_prefix(key)
                .is_some_and(|rest| rest.starts_with('='))
    };
    if args.iter().any(|arg| matches(arg, key)) {
        return true;
    }
    if let Some(option) = key.strip_prefix("-Z") {
        return args
            .windows(2)
            .any(|pair| pair[0] == "-Z" && matches(&pair[1], option));
    }
    key == VERBOSE_FLAG && args.iter().any(|arg| arg == VERBOSE_SHORT_FLAG)
}
