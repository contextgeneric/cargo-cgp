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

/// A stdout whose reader is gone ends the driver quietly, rather than by a `println!` panic that
/// prints an ICE report and leaves a `rustc-ice-*.txt` file in the working directory. A user meets
/// it by piping a direct invocation into a reader that stops early, as
/// `cargo-cgp-driver <rustc> -vV | head -1` does.
// Property: for each invocation that writes to stdout, a closed stdout ends the process with no panic
//   on stderr and no ICE file in the working directory: wrapper mode's `-vV` with rustc's code for a
//   failed print, 1, and direct mode's `--version` with 0, as the front-end's `expand` ends.
// Reaches: the pipe's read end is closed before the driver starts, so its first write fails; in
//   wrapper mode that is rustc's own print, which `is_closed_stdout_panic`'s tests cover the
//   later `println!` of (`containment.rs`), since a reader closing midway is a race.
#[cfg(unix)]
#[test]
fn a_closed_stdout_ends_the_driver_quietly() {
    use std::fs;
    use std::process::{Command, Stdio};

    let dir = std::env::temp_dir().join(format!("cargo-cgp-closed-stdout-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("creating the working directory");

    for (args, code) in [
        (&["/toolchain/bin/rustc", "-vV"][..], 1),
        (&["--version"][..], 0),
    ] {
        let (reader, writer) = std::io::pipe().expect("creating a pipe");
        drop(reader);
        let output = Command::new(env!("CARGO_BIN_EXE_cargo-cgp-driver"))
            .args(args)
            .stdout(writer)
            .stderr(Stdio::piped())
            .current_dir(&dir)
            .output()
            .expect("running the driver");
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert_eq!(output.status.code(), Some(code), "{args:?}: {stderr}");
        assert!(!stderr.contains("panicked"), "{args:?}: {stderr}");
    }
    let leftovers: Vec<_> = fs::read_dir(&dir)
        .expect("reading the working directory")
        .map(|entry| entry.expect("a directory entry").file_name())
        .collect();
    let _ = fs::remove_dir_all(&dir);
    assert!(leftovers.is_empty(), "{leftovers:?}");
}
