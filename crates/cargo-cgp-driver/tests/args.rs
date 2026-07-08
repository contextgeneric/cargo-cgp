//! Tests for the rustc argument-vector preparation ([`cargo_cgp_driver::args::rustc_args`]).
//!
//! This links `cargo-cgp-driver`, which links the compiler's `rustc_driver` dylib, so the
//! test crate carries the same `#![feature(rustc_private)]` gate the driver binary does —
//! the link is only permitted when the linking crate opts into the feature.

#![feature(rustc_private)]

use cargo_cgp_driver::args::rustc_args;

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
