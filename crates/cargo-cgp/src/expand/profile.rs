//! Choosing the cargo profile the expansion builds under.

/// Whether the forwarded arguments already choose a profile, in which case the default is not
/// added and the caller's choice stands.
///
/// `--release` and `--profile` are the two ways to say it; both are matched, `--profile` in either
/// its spaced or `=`-joined form.
pub fn forwards_profile(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "--release" || arg == "--profile" || arg.starts_with("--profile="))
}
