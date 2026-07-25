//! Tests for expand-mode recognition ([`cargo_cgp_driver::expand::take_expand_request`]).
//!
//! This links `cargo-cgp-driver`, which links the compiler's `rustc_driver` dylib, so the
//! test crate carries the same `#![feature(rustc_private)]` gate the driver binary does.
//! Printing the expansion needs a live compiler, so only the flag handling is pinned here;
//! the printing is exercised end to end.

#![feature(rustc_private)]

use cargo_cgp_driver::config::EXPAND_FLAG;
use cargo_cgp_driver::expand::take_expand_request;

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn takes_the_flag_and_its_path() {
    let mut argv = args(&[
        "cargo-cgp-driver",
        "/tk/bin/rustc",
        "--crate-name=demo",
        "--cgp-expand=/tmp/out.rs",
        "lib.rs",
    ]);

    let request = take_expand_request(&mut argv, EXPAND_FLAG).expect("a request");

    assert_eq!(request.output, "/tmp/out.rs");
    // The flag is no flag of rustc's, so it must not survive into the compiler's argument vector.
    assert_eq!(
        argv,
        [
            "cargo-cgp-driver",
            "/tk/bin/rustc",
            "--crate-name=demo",
            "lib.rs"
        ]
    );
}

#[test]
fn an_ordinary_compilation_carries_no_request() {
    let mut argv = args(&["cargo-cgp-driver", "/tk/bin/rustc", "lib.rs"]);

    assert!(take_expand_request(&mut argv, EXPAND_FLAG).is_none());
    assert_eq!(argv, ["cargo-cgp-driver", "/tk/bin/rustc", "lib.rs"]);
}

#[test]
fn an_empty_path_is_not_a_request() {
    // Nowhere to write means there is nothing to do; the flag is still stripped, since rustc would
    // reject it either way.
    let mut argv = args(&["cargo-cgp-driver", "--cgp-expand=", "lib.rs"]);

    assert!(take_expand_request(&mut argv, EXPAND_FLAG).is_none());
    assert_eq!(argv, ["cargo-cgp-driver", "lib.rs"]);
}
