//! Formatting a failed sysroot probe's stderr into its error message.

use cargo_cgp::launch::{format_stderr, sysroot_command};

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

#[test]
fn a_forced_toolchain_is_queried_through_rustup() {
    // `rustup run` answers for that toolchain whatever `rustc` is first on `PATH`.
    let (program, args) = sysroot_command("rustc", Some("nightly-2026-10-10"));
    assert_eq!(program, "rustup");
    assert_eq!(
        args,
        ["run", "nightly-2026-10-10", "rustc", "--print", "sysroot"]
    );
}

#[test]
fn an_unforced_query_asks_the_ambient_compiler() {
    let (program, args) = sysroot_command("/nix/store/x/bin/rustc", None);
    assert_eq!(program, "/nix/store/x/bin/rustc");
    assert_eq!(args, ["--print", "sysroot"]);
}
