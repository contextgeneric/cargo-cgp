//! Choosing the cargo profile the expansion builds under.

use crate::launch::{scan_flag, split_forwarded};

/// Whether the forwarded arguments already choose a profile, in which case the default is not
/// added and the caller's choice stands.
///
/// `--release` and `--profile` are the two ways to say it; both are matched, `--profile` in either
/// its spaced or `=`-joined form, and neither after a `--`, which goes to rustc.
pub fn forwards_profile(args: &[String]) -> bool {
    split_forwarded(args).0.iter().any(|arg| arg == "--release")
        || !scan_flag(args, "--profile").values.is_empty()
}
