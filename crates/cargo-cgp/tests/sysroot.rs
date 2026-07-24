//! Formatting a failed sysroot probe's stderr into its error message.

use cargo_cgp::check::format_stderr;

#[test]
fn a_loader_failure_is_appended_to_the_message() {
    // The archetype: the process never started, and the only account of why is on stderr.
    let stderr = b"rustc: error while loading shared libraries: libz.so.1: cannot open shared object file: No such file or directory\n";

    assert_eq!(
        format_stderr(stderr),
        ":\n\nrustc: error while loading shared libraries: libz.so.1: cannot open shared object file: No such file or directory"
    );
}

#[test]
fn empty_stderr_adds_nothing() {
    assert_eq!(format_stderr(b""), "");
    assert_eq!(format_stderr(b"  \n \n"), "");
}

#[test]
fn non_utf8_stderr_is_still_reported() {
    // A loader message comes from the OS, not from rustc, so it is not guaranteed UTF-8 —
    // and a mangled byte is no reason to withhold the diagnosis.
    let stderr = b"cannot open \xff\xfe shared object file";

    let formatted = format_stderr(stderr);

    assert!(formatted.starts_with(":\n\ncannot open "));
    assert!(formatted.ends_with(" shared object file"));
}
