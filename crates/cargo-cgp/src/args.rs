//! Normalizing the process arguments.
//!
//! `cargo-cgp` is reachable two ways, and this module reduces both to the same shape:
//! the arguments *after* the tool's own name and the cargo-inserted subcommand.
//!
//! - `cargo cgp check ...` → cargo runs `cargo-cgp cgp check ...`
//! - `cargo-cgp check ...` → run directly, no inserted subcommand
//!
//! In both cases the caller wants `["check", ...]`.

/// Drop the program name and, if present, the cargo-inserted subcommand token, so the
/// remaining slice starts at the user's actual subcommand.
///
/// `subcommand` is the token cargo inserts (see [`crate::config::CARGO_SUBCOMMAND`]); it
/// is passed in rather than hardcoded here. Only a *leading* occurrence is stripped, so
/// a later argument that happens to equal it is left untouched.
pub fn strip_subcommand(
    raw_args: impl IntoIterator<Item = String>,
    subcommand: &str,
) -> Vec<String> {
    let mut args = raw_args.into_iter();

    // Drop argv[0], the path to the cargo-cgp binary itself.
    let _program = args.next();

    let mut rest: Vec<String> = args.collect();
    if rest.first().map(String::as_str) == Some(subcommand) {
        rest.remove(0);
    }

    rest
}
