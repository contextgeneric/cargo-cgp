//! Tests for fixture discovery ([`cargo_cgp_ui_tests::fixtures::collect`]).

use std::fs;
use std::path::PathBuf;

use cargo_cgp_ui_tests::fixtures::collect;
use cargo_cgp_ui_tests::options::Options;

/// A throwaway fixture tree, named after the test so two tests never share one.
fn tree(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cgp-ui-fixtures-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("creating the temp fixture tree");
    dir
}

#[test]
fn an_expand_snapshot_is_not_collected_as_a_fixture() {
    // `<name>.expand.rs` is a snapshot that happens to be Rust and sits beside the fixture it
    // belongs to, so the collector has to tell the two apart. Otherwise every blessed run would
    // grow a fixture of its own, which would then be expanded and snapshotted in turn.
    let dir = tree("snapshot");
    fs::write(dir.join("demo.rs"), "fn main() {}\n").expect("writing the fixture");
    fs::write(dir.join("demo.expand.rs"), "fn main() {}\n").expect("writing the snapshot");

    let collected = collect(&dir, &Options::parse(Vec::new()));

    assert_eq!(collected.len(), 1, "collected: {collected:?}");
    assert!(collected[0].ends_with("demo.rs"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn collects_fixtures_from_nested_directories_in_order() {
    let dir = tree("nested");
    fs::create_dir_all(dir.join("group")).expect("creating a sub-directory");
    fs::write(dir.join("b.rs"), "fn main() {}\n").expect("writing a fixture");
    fs::write(dir.join("group/a.rs"), "fn main() {}\n").expect("writing a fixture");
    // A committed `.stderr` snapshot is not Rust, so it was never a candidate.
    fs::write(dir.join("b.cgp.stderr"), "").expect("writing a snapshot");

    let collected = collect(&dir, &Options::parse(Vec::new()));

    let names: Vec<String> = collected
        .iter()
        .map(|path| {
            path.strip_prefix(&dir)
                .unwrap_or(path)
                .display()
                .to_string()
        })
        .collect();
    assert_eq!(names, ["b.rs", "group/a.rs"]);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_no_expansion_marker_is_a_comment_block() {
    // No fixture triggers this today, so nothing else would notice if the constant's line
    // continuation ever indented its second line into the snapshot.
    let lines: Vec<&str> = cargo_cgp_ui_tests::harness::NO_EXPANSION.lines().collect();

    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(
        lines.iter().all(|line| line.starts_with("// ")),
        "every line must be a comment, so the snapshot is valid Rust: {lines:?}"
    );
}
