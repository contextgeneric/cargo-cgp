//! Discovering the fixture files.

use std::fs;
use std::path::{Path, PathBuf};

use crate::options::Options;

/// Collect every `.rs` fixture under `dir` (recursively) that passes the filters,
/// sorted for a stable run order. A fixture's own `.expand.rs` snapshot lives beside it and also
/// ends in `.rs`, so it is skipped — see [`is_snapshot`].
pub fn collect(dir: &Path, options: &Options) -> Vec<PathBuf> {
    let mut fixtures = Vec::new();
    walk(dir, dir, options, &mut fixtures);
    fixtures.sort();
    fixtures
}

/// Recurse into `current`, pushing matching `.rs` files. `root` is the fixtures
/// directory, used to compute the relative path the filters match against.
fn walk(root: &Path, current: &Path, options: &Options, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(current) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, options, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") && !is_snapshot(&path) {
            let relative = path.strip_prefix(root).unwrap_or(&path);
            if options.matches(&relative.to_string_lossy()) {
                out.push(path);
            }
        }
    }
}

/// Whether `path` is a committed snapshot rather than a fixture. The `.expand.rs` snapshot of the
/// generated code is Rust and sits beside the fixture it belongs to, so without this it would be
/// collected as a fixture of its own — and then expanded, snapshotted, and expanded again.
fn is_snapshot(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".expand.rs"))
}
