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
///   next-generation trait solver ([`crate::config::NEXT_SOLVER_FLAG`]) — see that
///   constant for why.
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

    for flag in injected_flags {
        // A `-Zkey=value` flag is "already set" if any argument carries the same key, so
        // that an explicit `-Zkey=other` on the command line is left to take precedence.
        let key = flag.split('=').next().unwrap_or(flag);
        if !has_flag(&args, key) {
            args.push((*flag).to_owned());
        }
    }

    args
}

/// Whether `args[1]` is a path to `rustc`, i.e. cargo called us as its rustc wrapper.
fn is_wrapper_mode(args: &[String]) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn strips_injected_rustc_path_in_wrapper_mode() {
        let out = rustc_args(
            args(&[
                "cargo-cgp-driver",
                "/tk/bin/rustc",
                "--edition=2024",
                "lib.rs",
            ]),
            None,
            "--sysroot",
            &[],
        );
        assert_eq!(out, ["cargo-cgp-driver", "--edition=2024", "lib.rs"]);
    }

    #[test]
    fn injects_sysroot_when_absent() {
        let out = rustc_args(
            args(&["cargo-cgp-driver", "/tk/bin/rustc", "lib.rs"]),
            Some("/tk".to_owned()),
            "--sysroot",
            &[],
        );
        assert_eq!(out, ["cargo-cgp-driver", "lib.rs", "--sysroot", "/tk"]);
    }

    #[test]
    fn keeps_existing_sysroot() {
        let out = rustc_args(
            args(&["d", "/tk/bin/rustc", "--sysroot=/other", "lib.rs"]),
            Some("/tk".to_owned()),
            "--sysroot",
            &[],
        );
        assert_eq!(out, ["d", "--sysroot=/other", "lib.rs"]);
    }

    #[test]
    fn leaves_direct_invocation_untouched() {
        // No rustc path at args[1]: not wrapper mode, nothing removed.
        let out = rustc_args(
            args(&["cargo-cgp-driver", "--version"]),
            None,
            "--sysroot",
            &[],
        );
        assert_eq!(out, ["cargo-cgp-driver", "--version"]);
    }

    #[test]
    fn appends_injected_flags_when_absent() {
        let out = rustc_args(
            args(&["d", "/tk/bin/rustc", "lib.rs"]),
            None,
            "--sysroot",
            &["-Znext-solver=globally"],
        );
        assert_eq!(out, ["d", "lib.rs", "-Znext-solver=globally"]);
    }

    #[test]
    fn keeps_user_override_of_injected_flag() {
        // An explicit `-Znext-solver=no` shares the `-Znext-solver` key, so nothing is added.
        let out = rustc_args(
            args(&["d", "/tk/bin/rustc", "-Znext-solver=no", "lib.rs"]),
            None,
            "--sysroot",
            &["-Znext-solver=globally"],
        );
        assert_eq!(out, ["d", "-Znext-solver=no", "lib.rs"]);
    }
}
