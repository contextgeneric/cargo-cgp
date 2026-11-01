//! Tests for the `expand` subcommand's pure helpers. Running the command itself spawns cargo and
//! the driver, so it is exercised end to end rather than here.

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use cargo_cgp::expand::{
    ExpansionDir, expansion_exit_code, forwards_profile, take_item, write_expansion,
};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

/// A fresh parent directory under the system temp directory, unique to this test and process.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "cargo-cgp-expand-test-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("creating a scratch directory");
    dir
}

/// A writer whose every write fails with `kind`, as stdout does once its reader is gone.
struct FailingWriter(io::ErrorKind);

impl Write for FailingWriter {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(self.0.into())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Property: `write_expansion` fails exactly when the write fails for a reason other than a closed
/// pipe. Regression: `cargo cgp expand | head` on a long expansion ended with a broken-pipe error.
#[test]
fn only_a_closed_pipe_is_not_a_write_failure() {
    let mut written = Vec::new();
    write_expansion(&mut written, "text").expect("a working writer");
    assert_eq!(written, b"text");

    write_expansion(&mut FailingWriter(io::ErrorKind::BrokenPipe), "text")
        .expect("a closed pipe is not a failure");
    let error = write_expansion(&mut FailingWriter(io::ErrorKind::StorageFull), "text")
        .expect_err("a full disk is a failure");
    assert_eq!(error.kind(), io::ErrorKind::StorageFull);
}

/// Property: for all cargo codes c, `expansion_exit_code(c, true) == c`, and
/// `expansion_exit_code(c, false)` is nonzero and equals c when c is nonzero. The inputs cover each
/// arm: a clean run, a compiler error that still printed an expansion (cargo's 101), an `--item` miss
/// (cargo's 0 with nothing printed), and a crate that failed before expanding.
#[test]
fn a_failure_is_reported_whether_or_not_an_expansion_came_back() {
    assert_eq!(expansion_exit_code(0, true), 0);
    assert_eq!(expansion_exit_code(101, true), 101);
    assert_eq!(expansion_exit_code(0, false), 1);
    assert_eq!(expansion_exit_code(101, false), 101);
}

#[test]
fn detects_a_forwarded_release_flag() {
    assert!(forwards_profile(&args(&["--lib", "--release"])));
}

#[test]
fn detects_a_forwarded_profile_in_either_form() {
    assert!(forwards_profile(&args(&["--profile", "dev"])));
    assert!(forwards_profile(&args(&["--profile=dev"])));
}

#[test]
fn no_profile_flag_leaves_the_default_to_be_added() {
    assert!(!forwards_profile(&args(&["--lib", "--features", "extra"])));
    // A target named `release` is a value, not the flag, so it must not count.
    assert!(!forwards_profile(&args(&["--bin", "release"])));
}

#[test]
fn each_expansion_gets_a_new_directory_removed_afterwards() {
    let parent = scratch("fresh");
    let first = ExpansionDir::create_in(&parent).expect("a directory");
    let second = ExpansionDir::create_in(&parent).expect("a second directory");

    assert_ne!(first.path(), second.path());
    assert!(first.output_path().starts_with(first.path()));
    let kept = first.path().to_path_buf();
    drop(first);
    assert!(
        !kept.exists(),
        "the directory should be removed with its handle"
    );
    drop(second);
    let _ = fs::remove_dir_all(&parent);
}

#[test]
fn a_name_another_process_holds_is_skipped_not_reused() {
    // Someone else's directory, or a symlink planted at the predictable name, must never become the
    // place the driver writes to.
    let parent = scratch("taken");
    let taken = parent.join(format!("cargo-cgp-expand-{}-0", std::process::id()));
    fs::create_dir(&taken).unwrap();
    fs::write(taken.join("expansion.rs"), "someone else's").unwrap();

    let dir = ExpansionDir::create_in(&parent).expect("a directory under another name");

    assert_ne!(dir.path(), taken);
    drop(dir);
    assert_eq!(
        fs::read_to_string(taken.join("expansion.rs")).unwrap(),
        "someone else's"
    );
    let _ = fs::remove_dir_all(&parent);
}

#[cfg(unix)]
#[test]
fn only_this_user_can_enter_the_directory() {
    use std::os::unix::fs::PermissionsExt;

    let parent = scratch("private");
    let dir = ExpansionDir::create_in(&parent).expect("a directory");
    let mode = fs::metadata(dir.path()).unwrap().permissions().mode();

    assert_eq!(mode & 0o077, 0, "group and others get no access: {mode:o}");
    drop(dir);
    let _ = fs::remove_dir_all(&parent);
}

#[test]
fn takes_the_item_filter_in_either_form() {
    let (forwarded, item) = take_item(&args(&["--lib", "--item", "shapes::Rectangle"])).unwrap();
    assert_eq!(forwarded, ["--lib"]);
    assert_eq!(item.as_deref(), Some("shapes::Rectangle"));

    let (forwarded, item) = take_item(&args(&["--item=shapes", "--lib"])).unwrap();
    assert_eq!(forwarded, ["--lib"]);
    assert_eq!(item.as_deref(), Some("shapes"));
}

#[test]
fn everything_else_is_forwarded_untouched() {
    let given = args(&["-p", "app", "--lib", "--features", "extra"]);
    let (forwarded, item) = take_item(&given).unwrap();

    assert_eq!(forwarded, given);
    assert!(item.is_none());
}

#[test]
fn a_bare_word_is_left_to_cargo() {
    // A positional cannot be told from a cargo flag's value — `--bin release` names a binary — so the
    // filter is only ever the explicit flag, and a bare word passes through.
    let (forwarded, item) = take_item(&args(&["--bin", "my_module"])).unwrap();

    assert_eq!(forwarded, ["--bin", "my_module"]);
    assert!(item.is_none());
}

#[test]
fn a_missing_or_repeated_item_filter_names_the_flag() {
    for bad in [
        vec!["--item"],
        vec!["--item="],
        vec!["--lib", "--item"],
        vec!["--item", "a", "--item", "b"],
    ] {
        let error = take_item(&args(&bad)).expect_err(&format!("{bad:?} should be rejected"));
        assert!(
            error.to_string().contains("--item"),
            "a flag-shaped mistake should name the flag: {error}"
        );
    }
}

#[test]
fn a_repeated_item_filter_says_what_to_do_in_a_help() {
    let error = take_item(&args(&["--item", "a", "--item", "b"])).unwrap_err();
    assert_eq!(
        error.to_string(),
        "`--item` was given twice (`a` and `b`)\n\nhelp: expand one path at a time"
    );
}

#[test]
fn a_crate_rooted_item_path_is_accepted() {
    // The front-end only checks the shape; the driver's parser strips the prefix.
    for spelling in [
        "crate::contexts::app",
        "::contexts::app",
        "self::contexts::app",
    ] {
        let (_, item) = take_item(&args(&["--item", spelling])).expect("a valid path");
        assert_eq!(item.as_deref(), Some(spelling));
    }
}

#[test]
fn a_malformed_item_path_names_the_path() {
    // Rejected here rather than after a build, so a typo costs nothing.
    for bad in [
        "not a path",
        "shapes::",
        "shapes:Rectangle",
        "shapes::<T>",
        "::",
    ] {
        let error =
            take_item(&args(&["--item", bad])).expect_err(&format!("{bad:?} should be rejected"));
        assert!(
            error.to_string().contains(bad) && error.to_string().contains("item path"),
            "the message should quote the path and say what was expected: {error}"
        );
    }
}
