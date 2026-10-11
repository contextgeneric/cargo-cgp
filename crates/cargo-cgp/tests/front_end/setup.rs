//! Tests for the driver install command `setup` runs ([`cargo_cgp::setup::driver_install_args`]).

use std::ffi::OsString;
use std::path::Path;

use cargo_cgp::setup::driver_install_args;

fn strings(args: Vec<OsString>) -> Vec<String> {
    args.into_iter()
        .map(|arg| arg.into_string().expect("UTF-8"))
        .collect()
}

#[test]
fn installs_the_driver_locked_at_the_front_end_version() {
    // `--locked` builds the lockfile the release was tested with, rather than the newest
    // compatible version of every dependency.
    assert_eq!(
        strings(driver_install_args("0.1.0", None)),
        [
            "install",
            "cargo-cgp-driver",
            "--version",
            "0.1.0",
            "--locked"
        ]
    );
}

#[test]
fn installs_the_driver_beside_the_front_end() {
    assert_eq!(
        strings(driver_install_args(
            "0.1.0",
            Some(Path::new("/home/u/.cargo"))
        )),
        [
            "install",
            "cargo-cgp-driver",
            "--version",
            "0.1.0",
            "--locked",
            "--root",
            "/home/u/.cargo"
        ]
    );
}
