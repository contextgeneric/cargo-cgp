//! Tests for the rustc argument-vector preparation ([`cargo_cgp_driver::args::rustc_args`]).

use cargo_cgp_driver::args::{expand_arg_files, rustc_args};

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

#[test]
fn skips_injection_on_version_probe() {
    // cargo's `rustc -vV` version query already implies `-v`; injecting `--verbose` a
    // second time would make rustc reject it, so no flag is added to an info query.
    let out = rustc_args(
        args(&["d", "/tk/bin/rustc", "-vV"]),
        None,
        "--sysroot",
        &["-Znext-solver=globally", "--verbose"],
    );
    assert_eq!(out, ["d", "-vV"]);
}

#[test]
fn skips_injection_on_print_query() {
    // `--print` requests (e.g. `--print=cfg`) are cargo probes, not real compilations, so
    // they receive none of the diagnostic flags either — in both the split and `=` forms.
    let split = rustc_args(
        args(&["d", "/tk/bin/rustc", "--print", "cfg"]),
        None,
        "--sysroot",
        &["-Znext-solver=globally", "--verbose"],
    );
    assert_eq!(split, ["d", "--print", "cfg"]);

    let joined = rustc_args(
        args(&["d", "/tk/bin/rustc", "--print=file-names", "lib.rs"]),
        None,
        "--sysroot",
        &["-Znext-solver=globally", "--verbose"],
    );
    assert_eq!(joined, ["d", "--print=file-names", "lib.rs"]);
}

#[test]
fn injects_flags_on_a_real_compilation() {
    // A genuine crate build carries neither `-vV` nor `--print`, so it gets every flag.
    let out = rustc_args(
        args(&["d", "/tk/bin/rustc", "--edition=2024", "lib.rs"]),
        None,
        "--sysroot",
        &["-Znext-solver=globally", "--verbose"],
    );
    assert_eq!(
        out,
        [
            "d",
            "--edition=2024",
            "lib.rs",
            "-Znext-solver=globally",
            "--verbose"
        ]
    );
}

#[test]
fn expands_an_argument_file_before_the_driver_reads_its_flags() {
    // When a command line grows long, cargo keeps the wrapper and the rustc path visible and moves
    // the rest into `@<file>`, one argument per line. The driver's own flags travel there too, so
    // the file is expanded before anything reads them.
    let dir = std::env::temp_dir().join(format!("cargo-cgp-argfile-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a temporary directory");
    let file = dir.join("args");
    std::fs::write(
        &file,
        "--crate-name=demo\n--cgp-expand=/tmp/out.rs\n-Znext-solver=no\nlib.rs",
    )
    .expect("an argument file");

    let expanded = expand_arg_files(args(&[
        "d",
        "/tk/bin/rustc",
        &format!("@{}", file.display()),
    ]));
    std::fs::remove_dir_all(&dir).ok();

    assert_eq!(
        expanded,
        [
            "d",
            "/tk/bin/rustc",
            "--crate-name=demo",
            "--cgp-expand=/tmp/out.rs",
            "-Znext-solver=no",
            "lib.rs"
        ]
    );
    // The user's solver choice inside the file is now seen, so the injected one is not appended.
    let out = rustc_args(expanded, None, "--sysroot", &["-Znext-solver=globally"]);
    assert!(!out.iter().any(|arg| arg == "-Znext-solver=globally"));
}

#[test]
fn keeps_a_user_override_written_as_two_arguments() {
    // rustc accepts `-Z key=value` as two arguments as well as `-Zkey=value`; either is the
    // user's explicit choice, so the injected `-Znext-solver=globally` is not appended after it.
    let out = rustc_args(
        args(&["d", "/tk/bin/rustc", "-Z", "next-solver=no", "lib.rs"]),
        None,
        "--sysroot",
        &["-Znext-solver=globally"],
    );
    assert_eq!(out, ["d", "-Z", "next-solver=no", "lib.rs"]);
}

#[test]
fn treats_the_short_verbose_flag_as_verbose() {
    // `-v` is `--verbose`; appending `--verbose` beside it makes rustc reject the invocation with
    // "Option 'verbose' given more than once", as `RUSTFLAGS=-v` would.
    let out = rustc_args(
        args(&["d", "/tk/bin/rustc", "-v", "lib.rs"]),
        None,
        "--sysroot",
        &["--verbose"],
    );
    assert_eq!(out, ["d", "-v", "lib.rs"]);
}
