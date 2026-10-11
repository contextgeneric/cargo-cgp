//! Locating the built binaries from the harness executable.
//!
//! `debug_dir` searches upwards for the directory holding the `cargo-cgp` front-end rather
//! than counting parent directories, because cargo does not put a test executable in a fixed
//! place across versions. These tests pin that search against both layouts cargo has used, so
//! the next move does not silently break every fixture with a `NotFound`.

use std::path::{Path, PathBuf};
use std::{env, fs};

use cargo_cgp_ui_tests::paths::ancestor_holding;

/// A throwaway directory tree under the system temp directory, removed on drop so a failing
/// assertion still cleans up. The harness takes no non-std dependencies, so this stands in
/// for a temp-directory crate.
struct TempTree(PathBuf);

impl TempTree {
    fn new(tag: &str) -> Self {
        let unique = format!(
            "cargo-cgp-paths-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        );
        let root = env::temp_dir().join(unique);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("creating the temp tree");
        TempTree(root)
    }

    /// Create `relative` as a directory under the tree and return its path.
    fn dir(&self, relative: &str) -> PathBuf {
        let path = self.0.join(relative);
        fs::create_dir_all(&path).expect("creating a directory in the temp tree");
        path
    }

    /// Create `relative` as an empty file under the tree, with its parents.
    fn file(&self, relative: &str) -> PathBuf {
        let path = self.0.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("creating a parent in the temp tree");
        }
        fs::write(&path, b"").expect("creating a file in the temp tree");
        path
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn finds_the_front_end_beside_a_deps_test_binary() {
    // The layout cargo used when the test binary sat in `<profile>/deps/`.
    let tree = TempTree::new("deps");
    let profile = tree.dir("target/debug");
    tree.file("target/debug/cargo-cgp");
    let exe = tree.file("target/debug/deps/ui-abc123");

    assert_eq!(ancestor_holding(&exe, "cargo-cgp"), Some(profile));
}

#[test]
fn finds_the_front_end_above_a_build_dir_test_binary() {
    // The layout cargo uses when the test binary sits under `<profile>/build/<pkg>/<hash>/out/`,
    // four levels below the front-end instead of one.
    let tree = TempTree::new("build");
    let profile = tree.dir("target/debug");
    tree.file("target/debug/cargo-cgp");
    let exe = tree.file("target/debug/build/cargo-cgp-ui-tests/abc123/out/ui-abc123");

    assert_eq!(ancestor_holding(&exe, "cargo-cgp"), Some(profile));
}

#[test]
fn prefers_the_nearest_ancestor_holding_the_binary() {
    // A binary of the same name higher up must not win over the one beside the test binary,
    // so an outer checkout's stale build cannot shadow this workspace's.
    let tree = TempTree::new("nearest");
    tree.file("cargo-cgp");
    let profile = tree.dir("target/debug");
    tree.file("target/debug/cargo-cgp");
    let exe = tree.file("target/debug/deps/ui-abc123");

    assert_eq!(ancestor_holding(&exe, "cargo-cgp"), Some(profile));
}

#[test]
fn declines_when_no_ancestor_holds_the_binary() {
    // `debug_dir` turns this into a "run `cargo build` first" panic rather than a path that
    // fails later as a `NotFound` when a fixture is run.
    let tree = TempTree::new("absent");
    let exe = tree.file("target/debug/deps/ui-abc123");

    assert_eq!(ancestor_holding(&exe, "cargo-cgp"), None);
}

#[test]
fn ignores_a_directory_of_the_binary_name() {
    // Only a *file* counts: cargo makes a `build/cargo-cgp/` directory for the front-end's
    // build script, which sits on the search path and must not be mistaken for the binary.
    let tree = TempTree::new("dir-named");
    tree.dir("target/debug/build/cargo-cgp");
    let exe = tree.file("target/debug/build/cargo-cgp-ui-tests/abc123/out/ui-abc123");

    assert_eq!(ancestor_holding(&exe, "cargo-cgp"), None);
}

#[test]
fn searching_from_a_directory_considers_that_directory() {
    let tree = TempTree::new("self");
    let profile = tree.dir("target/debug");
    tree.file("target/debug/cargo-cgp");

    assert_eq!(
        ancestor_holding(Path::new(&profile), "cargo-cgp"),
        Some(profile)
    );
}
