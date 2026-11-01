//! Auxiliary-crate support for fixtures, modeled on Clippy's `aux-build`.
//!
//! A single throwaway crate cannot express a cross-crate scenario, since the orphan
//! rule and cross-crate coherence only exist *between* crates. A fixture that
//! needs a companion crate declares it with a header directive:
//!
//! ```text
//! //@aux-build: cgp-test-crate-a
//! ```
//!
//! The named crate's source lives under [`auxiliary/`](crate::paths::auxiliary_src_dir);
//! its manifest there is a template whose `cgp` dependency is the placeholder
//! `CGP_PLACEHOLDER`. Before a run the harness [materializes](materialize_all)
//! every stored auxiliary crate once, copying its `src/` and writing a manifest
//! with the placeholder replaced by the sibling `cgp` checkout the rest of the
//! harness already resolves, into [`aux_build_root`].
//! A fixture's directly-declared crates are then added as path dependencies of the
//! worker crate; a transitive aux dependency (one aux crate's `../other` path) is
//! pulled in by cargo through the materialized sibling, so only direct declarations
//! need listing.

use std::fs;
use std::path::{Path, PathBuf};

use crate::paths::{aux_build_root, auxiliary_src_dir, cgp_crate_dir};

/// The token a stored auxiliary manifest uses for the `cgp` crate path, replaced
/// with the resolved sibling checkout when the crate is materialized.
const CGP_PLACEHOLDER: &str = "__CGP_CRATE_DIR__";

/// Materialize every stored auxiliary crate once and return a map from crate name
/// to its materialized directory. Run at setup, before any worker starts, so the
/// static sources are copied and their manifests generated a single time; each
/// worker then references the shared directory as a path dependency and builds it
/// in its own target directory, so there is no cross-worker lock contention.
/// Returns an empty map when the checkout has no `auxiliary/` directory.
pub fn materialize_all() -> Vec<(String, PathBuf)> {
    let src_root = auxiliary_src_dir();
    let build_root = aux_build_root();
    let cgp = cgp_crate_dir();

    let mut out = Vec::new();
    let entries = match fs::read_dir(&src_root) {
        Ok(entries) => entries,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let src = entry.path();
        if !src.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let dest = build_root.join(&name);
        materialize_one(&src, &dest, &cgp);
        out.push((name, dest));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Copy one auxiliary crate's `src/` tree into `dest` and write its manifest with
/// the `cgp` path placeholder resolved. The previous copy of `src/` is removed first, so a
/// file deleted from the stored crate does not survive in the materialized one.
fn materialize_one(src: &Path, dest: &Path, cgp_crate_dir: &Path) {
    let dest_src = dest.join("src");
    if dest_src.exists() {
        fs::remove_dir_all(&dest_src)
            .unwrap_or_else(|e| panic!("clearing aux source {}: {e}", dest_src.display()));
    }
    copy_tree(&src.join("src"), &dest_src);

    let template = fs::read_to_string(src.join("Cargo.toml"))
        .unwrap_or_else(|e| panic!("reading aux manifest {}: {e}", src.display()));
    let manifest = template.replace(CGP_PLACEHOLDER, &cgp_crate_dir.display().to_string());
    fs::create_dir_all(dest).expect("creating the aux crate directory");
    fs::write(dest.join("Cargo.toml"), manifest).expect("writing the aux manifest");
}

/// Recursively copy a directory tree into `dest`, creating it as needed.
fn copy_tree(src: &Path, dest: &Path) {
    fs::create_dir_all(dest).expect("creating an aux source directory");
    let entries =
        fs::read_dir(src).unwrap_or_else(|e| panic!("reading aux source {}: {e}", src.display()));
    for entry in entries.flatten() {
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if from.is_dir() {
            copy_tree(&from, &to);
        } else {
            fs::copy(&from, &to).unwrap_or_else(|e| panic!("copying {}: {e}", from.display()));
        }
    }
}

/// Resolve a fixture's declared auxiliary crates (see [`crate::directives`]) to
/// `(name, materialized dir)` pairs, drawn from the map [`materialize_all`] produced. A
/// declared name with no matching materialized crate is skipped (the fixture will then fail
/// to compile, surfacing the typo through the snapshot rather than a harness panic).
pub fn required<'a>(
    declared: &[String],
    materialized: &'a [(String, PathBuf)],
) -> Vec<(&'a str, &'a Path)> {
    declared
        .iter()
        .filter_map(|name| {
            materialized
                .iter()
                .find(|(n, _)| n == name)
                .map(|(n, dir)| (n.as_str(), dir.as_path()))
        })
        .collect()
}
