//! Tests for the rustc argument-vector preparation ([`cargo_cgp_driver::args::rustc_args`] and
//! [`cargo_cgp_driver::args::force_flags`]).

use cargo_cgp_driver::args::{ForcedFlag, expand_arg_files, force_flags, rustc_args};

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

// Property: for all argument vectors whose second argument is a path with the file stem `rustc`
//   (wrapper mode), `rustc_args` removes that argument and keeps the others in order.
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

// Property: given a sysroot and no argument that sets `sysroot_flag`, `rustc_args` appends
//   `sysroot_flag` and the sysroot after the invocation's own arguments.
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

// Property: an argument that already sets `sysroot_flag` (here `--sysroot=…`) wins, so no sysroot
//   is appended.
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

// Property: for all argument vectors whose second argument is not a path to `rustc` (a direct
//   invocation), `rustc_args` removes nothing.
#[test]
fn leaves_direct_invocation_untouched() {
    let out = rustc_args(
        args(&["cargo-cgp-driver", "--version"]),
        None,
        "--sysroot",
        &[],
    );
    assert_eq!(out, ["cargo-cgp-driver", "--version"]);
}

// Property: for a compilation whose arguments do not set an injected flag's key, `rustc_args`
//   appends that flag after the invocation's own arguments.
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

// Property: an argument with the key of an injected `-Zkey=value` flag (`-Znext-solver=no`) is
//   the user's choice, so the injected flag is not appended.
#[test]
fn keeps_user_override_of_injected_flag() {
    let out = rustc_args(
        args(&["d", "/tk/bin/rustc", "-Znext-solver=no", "lib.rs"]),
        None,
        "--sysroot",
        &["-Znext-solver=globally"],
    );
    assert_eq!(out, ["d", "-Znext-solver=no", "lib.rs"]);
}

// Property: cargo's `-vV` version probe, an info query, receives none of the injected flags:
//   `-vV` already implies `-v`, so an appended `--verbose` would make rustc reject it.
#[test]
fn skips_injection_on_version_probe() {
    let out = rustc_args(
        args(&["d", "/tk/bin/rustc", "-vV"]),
        None,
        "--sysroot",
        &["-Znext-solver=globally", "--verbose"],
    );
    assert_eq!(out, ["d", "-vV"]);
}

// Property: a `--print` request, a cargo probe, receives none of the injected flags, in both its
//   split (`--print cfg`) and joined (`--print=…`) forms.
#[test]
fn skips_injection_on_print_query() {
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

// Property: a compilation carrying neither `-vV` nor `--print` receives every injected flag, in
//   the given order, after its own arguments.
#[test]
fn injects_flags_on_a_real_compilation() {
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

// Property: `expand_arg_files` replaces an `@file` argument, in place, with the file's lines, so
//   `rustc_args` sees a user's choice inside the file (`-Znext-solver=no`) and does not append
//   the injected flag.
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

// Property: a `-Z` option written as two arguments (`-Z next-solver=no`) sets its key as
//   `-Znext-solver=no` does, so the injected flag is not appended after it.
#[test]
fn keeps_a_user_override_written_as_two_arguments() {
    let out = rustc_args(
        args(&["d", "/tk/bin/rustc", "-Z", "next-solver=no", "lib.rs"]),
        None,
        "--sysroot",
        &["-Znext-solver=globally"],
    );
    assert_eq!(out, ["d", "-Z", "next-solver=no", "lib.rs"]);
}

// Property: `-v` sets `--verbose`, so `--verbose` is not appended beside it, which would make
//   rustc reject the invocation with "Option 'verbose' given more than once".
#[test]
fn treats_the_short_verbose_flag_as_verbose() {
    let out = rustc_args(
        args(&["d", "/tk/bin/rustc", "-v", "lib.rs"]),
        None,
        "--sysroot",
        &["--verbose"],
    );
    assert_eq!(out, ["d", "-v", "lib.rs"]);
}

/// The one-thread flag the driver forces, with the options it replaces.
const ONE_THREAD: ForcedFlag<'static> = ForcedFlag {
    flag: "-Zthreads=1",
    replaces: &["-Zthreads", "--jobs-frontend"],
};

// Property: the postcondition of `force_flags`, for a compilation that sets no thread count: its
//   arguments are kept and the forced flag follows them.
#[test]
fn forces_one_thread_when_no_count_is_set() {
    let out = force_flags(args(&["d", "--edition=2024", "lib.rs"]), &[ONE_THREAD]);
    assert_eq!(out, ["d", "--edition=2024", "lib.rs", "-Zthreads=1"]);
}

// Property: the postcondition of `force_flags`, for every spelling rustc accepts for `-Zthreads`
//   and `--jobs-frontend`: the thread count is removed, with its value when the value is a separate
//   argument, the other arguments keep their order, and the forced flag is appended once.
#[test]
fn replaces_every_spelling_of_a_thread_count() {
    let cases: [(&[&str], &[&str]); 6] = [
        (
            &["d", "-Zthreads=8", "lib.rs"],
            &["d", "lib.rs", "-Zthreads=1"],
        ),
        (
            &["d", "-Zthreads", "lib.rs"],
            &["d", "lib.rs", "-Zthreads=1"],
        ),
        (
            &["d", "-Z", "threads=8", "lib.rs"],
            &["d", "lib.rs", "-Zthreads=1"],
        ),
        (
            &["d", "--jobs-frontend=4", "lib.rs"],
            &["d", "lib.rs", "-Zthreads=1"],
        ),
        (
            &["d", "--jobs-frontend", "4", "lib.rs"],
            &["d", "lib.rs", "-Zthreads=1"],
        ),
        // Both options at once, around an unrelated two-argument `-Z` option that stays.
        (
            &[
                "d",
                "-Zthreads=8",
                "-Z",
                "unstable-options",
                "--jobs-frontend",
                "4",
                "lib.rs",
            ],
            &["d", "-Z", "unstable-options", "lib.rs", "-Zthreads=1"],
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(
            force_flags(args(input), &[ONE_THREAD]),
            expected,
            "for {input:?}"
        );
    }
}

// Property: an option that only shares a prefix with a thread-count option (`--jobs`,
//   `--jobs-backend`, another `-Z` option) is kept.
#[test]
fn keeps_options_that_only_share_a_prefix() {
    let out = force_flags(
        args(&[
            "d",
            "--jobs",
            "4",
            "--jobs-backend=2",
            "-Z",
            "next-solver=no",
            "lib.rs",
        ]),
        &[ONE_THREAD],
    );
    assert_eq!(
        out,
        [
            "d",
            "--jobs",
            "4",
            "--jobs-backend=2",
            "-Z",
            "next-solver=no",
            "lib.rs",
            "-Zthreads=1"
        ]
    );
}

// Property: cargo's info queries (`-vV`, `--print`) are returned unchanged, a thread count
//   included.
#[test]
fn leaves_info_queries_unchanged() {
    for query in [
        &["d", "-vV"][..],
        &["d", "-Zthreads=8", "--print", "cfg"][..],
    ] {
        assert_eq!(force_flags(args(query), &[ONE_THREAD]), query);
    }
}

// Property: with no forced flag, as in expand mode, the arguments are returned unchanged.
#[test]
fn forces_nothing_without_a_forced_flag() {
    let input = args(&["d", "-Zthreads=8", "lib.rs"]);
    assert_eq!(force_flags(input.clone(), &[]), input);
}

// Property: a thread count inside an `@file` argument is seen once the file is expanded, so it is
//   replaced like one on the command line.
#[test]
fn replaces_a_thread_count_inside_an_argument_file() {
    let dir = std::env::temp_dir().join(format!("cargo-cgp-threads-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a temporary directory");
    let file = dir.join("args");
    std::fs::write(&file, "--crate-name=demo\n-Z\nthreads=8\nlib.rs").expect("an argument file");

    let expanded = expand_arg_files(args(&[
        "d",
        "/tk/bin/rustc",
        &format!("@{}", file.display()),
    ]));
    std::fs::remove_dir_all(&dir).ok();

    let out = force_flags(rustc_args(expanded, None, "--sysroot", &[]), &[ONE_THREAD]);
    assert_eq!(out, ["d", "--crate-name=demo", "lib.rs", "-Zthreads=1"]);
}
