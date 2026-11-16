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

/// Build the argument vector handed to [`rustc_driver::compiler_entrypoint`].
///
/// cargo invokes the wrapper as `cargo-cgp-driver <path-to-rustc> <rustc args...>`. Like
/// `clippy-driver`, we detect that "wrapper mode" (the second argument is a path whose
/// file stem is `rustc`) and drop the injected compiler path, leaving our own program
/// name in `args[0]` (which `compiler_entrypoint` ignores) followed by the genuine rustc flags.
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
///   `--verbose` flag ([`crate::config::VERBOSE_FLAG`]); see those constants for why.
///
/// The injection is skipped for cargo's *info queries*: the `rustc -vV` version probe and
/// the `--print` requests cargo runs before a build. Those need no CGP handling, and one of
/// our flags actively breaks them: `-vV` already implies `-v`, so appending `--verbose` a
/// second time makes rustc reject the invocation with "given more than once". Clippy skips
/// its own added flags on the same queries, and for the same reason ([rust-lang/cargo#14385]
/// notes that a query carrying unexpected flags also poisons cargo's cache). The real crate
/// compilation, the only invocation whose diagnostics we care about, carries neither
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

/// A flag the driver sets in place of the invocation's own choice, unlike the injected flags of
/// [`rustc_args`], which give way to it.
pub struct ForcedFlag<'a> {
    /// The flag appended to the arguments.
    pub flag: &'a str,
    /// The options removed from the arguments first, each in every spelling rustc accepts.
    pub replaces: &'a [&'a str],
}

/// Replace, for each forced flag, every argument that sets one of its `replaces` options with the
/// flag itself, appended after the other arguments.
///
/// An option is recognized in each spelling rustc accepts: `key` and `key=value`; for a `-Z` option
/// (`-Zthreads`), the two-argument `-Z threads=value`; and for a `--` option that takes a value
/// (`--jobs-frontend`), the two-argument `--jobs-frontend value`. The arguments reach this function
/// with their `@file` arguments already expanded, so an option inside such a file is removed too.
/// Cargo's info queries are returned unchanged, for the reason [`rustc_args`] gives.
///
/// Postcondition: unless `args` is an info query or `forced` is empty, the result holds the
/// arguments of `args` that set none of the `replaces` options, in their order, followed by each
/// forced `flag` once, in the order of `forced`.
pub fn force_flags(args: Vec<String>, forced: &[ForcedFlag<'_>]) -> Vec<String> {
    if forced.is_empty() || is_info_query(&args) {
        return args;
    }
    let mut out = Vec::with_capacity(args.len() + forced.len());
    let mut rest = args.into_iter().peekable();
    while let Some(arg) = rest.next() {
        let next = rest.peek().map(String::as_str);
        let width = forced
            .iter()
            .flat_map(|forced| forced.replaces)
            .find_map(|key| option_width(&arg, next, key));
        match width {
            Some(OptionWidth::OneArgument) => {}
            Some(OptionWidth::TwoArguments) => {
                rest.next();
            }
            None => out.push(arg),
        }
    }
    out.extend(forced.iter().map(|forced| forced.flag.to_owned()));
    out
}

/// How many arguments a spelling of an option occupies.
enum OptionWidth {
    OneArgument,
    TwoArguments,
}

/// Whether `arg`, followed by `next`, sets the option `key`, and in how many arguments.
fn option_width(arg: &str, next: Option<&str>, key: &str) -> Option<OptionWidth> {
    let sets = |arg: &str, key: &str| {
        arg == key
            || arg
                .strip_prefix(key)
                .is_some_and(|rest| rest.starts_with('='))
    };
    if key.starts_with("--") && arg == key {
        return Some(OptionWidth::TwoArguments);
    }
    if sets(arg, key) {
        return Some(OptionWidth::OneArgument);
    }
    let option = key.strip_prefix("-Z")?;
    (arg == "-Z" && next.is_some_and(|next| sets(next, option)))
        .then_some(OptionWidth::TwoArguments)
}
