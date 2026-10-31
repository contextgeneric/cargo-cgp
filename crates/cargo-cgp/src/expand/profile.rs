//! Choosing the cargo profile the expansion builds under.

use crate::launch::scan_flag;

/// Whether the forwarded arguments already choose a profile, in which case the default is not
/// added and the caller's choice stands.
///
/// `--release` and `--profile` are the two ways to say it; both are matched, `--profile` in either
/// its spaced or `=`-joined form.
pub fn forwards_profile(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--release") || !scan_flag(args, "--profile").values.is_empty()
}
