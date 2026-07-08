//! Discovering the fixture files.

use std::fs;
use std::path::{Path, PathBuf};

use crate::options::Options;

/// Collect every `.rs` fixture under `dir` (recursively) that passes the filters,
/// sorted for a stable run order.
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
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let relative = path.strip_prefix(root).unwrap_or(&path);
            if options.matches(&relative.to_string_lossy()) {
                out.push(path);
            }
        }
    }
}
