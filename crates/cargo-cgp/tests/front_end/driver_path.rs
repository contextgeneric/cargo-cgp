//! Tests for locating the driver ([`cargo_cgp::launch::locate_driver`]) and naming it in messages.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use cargo_cgp::launch::{DriverLocation, DriverOrigin, locate_driver};

const DRIVER: &str = "cargo-cgp-driver";

/// A fresh directory under the system temp directory, unique to this test and process.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "cargo-cgp-driver-path-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("creating a scratch directory");
    dir
}

fn with_driver(dir: &Path) -> PathBuf {
    let driver = dir.join(DRIVER);
    fs::write(&driver, "").expect("writing a stand-in driver");
    driver
}

#[test]
fn the_override_wins() {
    let front_end = scratch("override");
    with_driver(&front_end);

    let located = locate_driver(
        Some(OsString::from("/opt/driver")),
        Some(&front_end),
        None,
        DRIVER,
    );
    assert_eq!(
        located,
        DriverLocation {
            path: "/opt/driver".into(),
            origin: DriverOrigin::Override,
        }
    );
}

#[test]
fn a_sibling_comes_before_path() {
    let front_end = scratch("sibling");
    let elsewhere = scratch("sibling-path");
    let sibling = with_driver(&front_end);
    with_driver(&elsewhere);

    let path_var = std::env::join_paths([&elsewhere]).unwrap();
    let located = locate_driver(None, Some(&front_end), Some(&path_var), DRIVER);
    assert_eq!(located.path, sibling);
    assert_eq!(located.origin, DriverOrigin::Sibling);
}

#[test]
fn path_is_searched_when_no_sibling_exists() {
    let front_end = scratch("on-path");
    let empty = scratch("on-path-empty");
    let elsewhere = scratch("on-path-bin");
    let found = with_driver(&elsewhere);

    let path_var = std::env::join_paths([&empty, &elsewhere]).unwrap();
    let located = locate_driver(None, Some(&front_end), Some(&path_var), DRIVER);
    assert_eq!(located.path, found);
    assert_eq!(located.origin, DriverOrigin::Path);
    assert!(
        located.describe().contains("found on PATH"),
        "{}",
        located.describe()
    );
}

#[test]
fn a_driver_found_nowhere_keeps_the_bare_name() {
    let front_end = scratch("missing");
    let empty = scratch("missing-path");

    let path_var = std::env::join_paths([&empty]).unwrap();
    let located = locate_driver(None, Some(&front_end), Some(&path_var), DRIVER);
    assert_eq!(located.path, PathBuf::from(DRIVER));
    assert_eq!(located.origin, DriverOrigin::Missing);
    assert!(
        located
            .describe()
            .contains("none is beside cargo-cgp or on PATH"),
        "{}",
        located.describe()
    );
}
