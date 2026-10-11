//! Tests for the replaced-wrapper warning ([`cargo_cgp::launch::workspace_wrapper_warning`]).

use std::ffi::OsStr;
use std::path::Path;

use cargo_cgp::launch::workspace_wrapper_warning;

const DRIVER: &str = "/home/u/.cargo/bin/cargo-cgp-driver";

#[test]
fn a_users_wrapper_is_named_in_the_warning() {
    let warning =
        workspace_wrapper_warning(Some(OsStr::new("/usr/bin/my-wrapper")), Path::new(DRIVER))
            .expect("a wrapper of the user's own is replaced");
    assert!(warning.contains("RUSTC_WORKSPACE_WRAPPER"), "{warning}");
    assert!(warning.contains("`/usr/bin/my-wrapper`"), "{warning}");
    assert!(warning.contains("does not run"), "{warning}");
}

#[test]
fn nothing_to_replace_gives_no_warning() {
    let driver = Path::new(DRIVER);
    assert_eq!(workspace_wrapper_warning(None, driver), None);
    assert_eq!(
        workspace_wrapper_warning(Some(OsStr::new("")), driver),
        None
    );
    // A front-end run inside another one sees its own driver there already.
    assert_eq!(
        workspace_wrapper_warning(Some(OsStr::new(DRIVER)), driver),
        None
    );
}
