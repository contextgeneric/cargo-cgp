//! Assembling the argument list of a wrapped cargo command.

use std::ffi::OsString;
use std::path::Path;

/// Split the user's forwarded arguments at their first `--`: the arguments cargo reads, and the
/// ones after the separator, which cargo hands to rustc (`None` when there is no separator).
pub fn split_forwarded(args: &[String]) -> (&[String], Option<&[String]>) {
    match args.iter().position(|arg| arg == "--") {
        Some(separator) => (&args[..separator], Some(&args[separator + 1..])),
        None => (args, None),
    }
}

/// The arguments of `cargo <subcommand>`, in the order cargo reads them.
///
/// The user's cargo arguments come first, then `tool_cargo_args` (flags the tool adds, such as
/// `expand`'s `--profile check`), then `--target-dir <target_dir>` when the tool chose the
/// directory. One `--` follows when anything is bound for rustc: the user's own arguments after
/// their `--`, then `tool_rustc_args` (`expand`'s marker flags). Everything the tool adds for cargo
/// therefore stays in front of the separator, and a user's `--` is never doubled.
pub fn cargo_invocation(
    subcommand: &str,
    forwarded_args: &[String],
    tool_cargo_args: &[String],
    target_dir: Option<&Path>,
    tool_rustc_args: &[String],
) -> Vec<OsString> {
    let (cargo_args, rustc_args) = split_forwarded(forwarded_args);

    let mut args: Vec<OsString> = vec![subcommand.into()];
    args.extend(cargo_args.iter().map(OsString::from));
    args.extend(tool_cargo_args.iter().map(OsString::from));
    if let Some(target_dir) = target_dir {
        args.push("--target-dir".into());
        args.push(target_dir.into());
    }

    if rustc_args.is_some() || !tool_rustc_args.is_empty() {
        args.push("--".into());
        args.extend(rustc_args.unwrap_or_default().iter().map(OsString::from));
        args.extend(tool_rustc_args.iter().map(OsString::from));
    }
    args
}
