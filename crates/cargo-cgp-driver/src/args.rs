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
/// We then inject `--sysroot <sysroot>` unless the flags already carry one, because the
/// driver lives outside any toolchain and rustc cannot otherwise find `std`. `sysroot`
/// is the value from [`crate::config::SYSROOT_ENV`]; `sysroot_flag` is passed in rather
/// than hardcoded so the injection logic stays independent of the flag's spelling.
pub fn rustc_args(
    raw_args: impl IntoIterator<Item = String>,
    sysroot: Option<String>,
    sysroot_flag: &str,
) -> Vec<String> {
    let mut args: Vec<String> = raw_args.into_iter().collect();

    if is_wrapper_mode(&args) {
        args.remove(1);
    }

    if let Some(sysroot) = sysroot
        && !has_sysroot(&args, sysroot_flag)
    {
        args.push(sysroot_flag.to_owned());
        args.push(sysroot);
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

/// Whether the flags already set a sysroot, in either `--sysroot X` or `--sysroot=X` form.
fn has_sysroot(args: &[String], sysroot_flag: &str) -> bool {
    let with_eq = format!("{sysroot_flag}=");
    args.iter()
        .any(|arg| arg == sysroot_flag || arg.starts_with(&with_eq))
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
        );
        assert_eq!(out, ["cargo-cgp-driver", "--edition=2024", "lib.rs"]);
    }

    #[test]
    fn injects_sysroot_when_absent() {
        let out = rustc_args(
            args(&["cargo-cgp-driver", "/tk/bin/rustc", "lib.rs"]),
            Some("/tk".to_owned()),
            "--sysroot",
        );
        assert_eq!(out, ["cargo-cgp-driver", "lib.rs", "--sysroot", "/tk"]);
    }

    #[test]
    fn keeps_existing_sysroot() {
        let out = rustc_args(
            args(&["d", "/tk/bin/rustc", "--sysroot=/other", "lib.rs"]),
            Some("/tk".to_owned()),
            "--sysroot",
        );
        assert_eq!(out, ["d", "--sysroot=/other", "lib.rs"]);
    }

    #[test]
    fn leaves_direct_invocation_untouched() {
        // No rustc path at args[1]: not wrapper mode, nothing removed.
        let out = rustc_args(args(&["cargo-cgp-driver", "--version"]), None, "--sysroot");
        assert_eq!(out, ["cargo-cgp-driver", "--version"]);
    }
}
