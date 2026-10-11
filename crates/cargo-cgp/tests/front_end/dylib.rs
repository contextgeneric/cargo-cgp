//! Tests for building the dynamic-library search path ([`cargo_cgp::launch::dylib_search_path`]).

use std::ffi::OsString;
use std::path::Path;

use cargo_cgp::launch::{DYLIB_PATH_VAR, dylib_search_path};

#[test]
fn the_sysroot_library_comes_first() {
    let existing = std::env::join_paths(["/usr/lib", "/opt/lib"]).unwrap();
    let joined = dylib_search_path(Path::new("/sysroot/lib"), Some(&existing)).unwrap();

    let entries: Vec<_> = std::env::split_paths(&joined).collect();
    assert_eq!(
        entries,
        [
            Path::new("/sysroot/lib"),
            Path::new("/usr/lib"),
            Path::new("/opt/lib")
        ]
    );
}

#[test]
fn an_unset_search_path_holds_the_sysroot_library_alone() {
    let joined = dylib_search_path(Path::new("/sysroot/lib"), None).unwrap();
    assert_eq!(joined, OsString::from("/sysroot/lib"));
}

#[cfg(unix)]
#[test]
fn a_directory_holding_the_separator_is_an_error() {
    // A path with `:` in it cannot be one entry of a `:`-separated list.
    let error = dylib_search_path(Path::new("/home/a:b/sysroot/lib"), None)
        .expect_err("an unjoinable directory is reported, not a panic");
    let message = format!("{error:#}");
    assert!(message.contains("/home/a:b/sysroot/lib"), "{message}");
    assert!(message.contains(DYLIB_PATH_VAR), "{message}");
}
