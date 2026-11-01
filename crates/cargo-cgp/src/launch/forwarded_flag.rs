//! Reading a flag out of the forwarded arguments.

use crate::launch::invocation::split_forwarded;

/// One flag's occurrences in a list of arguments.
#[derive(Debug, PartialEq, Eq)]
pub struct FlagScan {
    /// Each occurrence's value in argument order, `None` for a spaced flag with no argument after it.
    pub values: Vec<Option<String>>,
    /// The other arguments, in order, with each occurrence and its value removed.
    pub remainder: Vec<String>,
}

/// Find `flag` in the arguments cargo reads, in either `flag X` or `flag=X` form.
///
/// The argument after a spaced flag is its value, even when it looks like a flag, and a flag that
/// only starts with `flag` is another flag. The first `--` ends the scan, since cargo hands what
/// follows it to rustc ([`split_forwarded`]), and the remainder keeps the separator and the rest.
pub fn scan_flag(args: &[String], flag: &str) -> FlagScan {
    let (cargo_args, rustc_args) = split_forwarded(args);
    let mut values = Vec::new();
    let mut remainder = Vec::with_capacity(args.len());
    let mut arguments = cargo_args.iter();
    while let Some(arg) = arguments.next() {
        if arg == flag {
            values.push(arguments.next().cloned());
        } else if let Some(value) = arg
            .strip_prefix(flag)
            .and_then(|rest| rest.strip_prefix('='))
        {
            values.push(Some(value.to_owned()));
        } else {
            remainder.push(arg.clone());
        }
    }
    if let Some(rustc_args) = rustc_args {
        remainder.push("--".to_owned());
        remainder.extend_from_slice(rustc_args);
    }
    FlagScan { values, remainder }
}

/// Whether the forwarded arguments already set `--target-dir` (in either `--target-dir X`
/// or `--target-dir=X` form), in which case the default is not injected.
pub fn forwards_target_dir(args: &[String]) -> bool {
    !scan_flag(args, "--target-dir").values.is_empty()
}

/// The `--manifest-path` the caller forwarded, in either `--manifest-path X` or
/// `--manifest-path=X` form, so the target-directory query asks about the same package the
/// wrapped command builds.
pub fn forwarded_manifest_path(args: &[String]) -> Option<String> {
    scan_flag(args, "--manifest-path")
        .values
        .into_iter()
        .next()
        .flatten()
}

/// The values of every `--config` the caller forwarded, in either `--config X` or `--config=X`
/// form, in order. A `build.target-dir` among them decides the target directory, and any other
/// value may change how cargo reads the project, so the query sees them all.
pub fn forwarded_config(args: &[String]) -> Vec<String> {
    scan_flag(args, "--config")
        .values
        .into_iter()
        .flatten()
        .collect()
}
