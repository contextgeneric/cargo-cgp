//! Tests that run the built driver binary, for the behavior that lives in its entrypoint.

/// A non-Unicode argument is reported as rustc reports one, not a panic. rustc's own ICE hook reads
/// `env::args` when `RUST_BACKTRACE` is unset, so the case is run with it unset.
#[cfg(unix)]
#[test]
fn a_non_unicode_argument_is_an_error() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::process::Command;

    let output = Command::new(env!("CARGO_BIN_EXE_cargo-cgp-driver"))
        .arg("/toolchain/bin/rustc")
        .arg(OsString::from_vec(b"--cfg=\xff".to_vec()))
        .env_remove("RUST_BACKTRACE")
        .current_dir(std::env::temp_dir())
        .output()
        .expect("running the driver");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("argument 2 is not valid Unicode"),
        "{stderr}"
    );
    assert!(!stderr.contains("panicked"), "{stderr}");
}
