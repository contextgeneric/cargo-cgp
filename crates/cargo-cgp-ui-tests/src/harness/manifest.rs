//! The worker crate's manifest.

use std::fs;
use std::path::Path;

use crate::paths::cgp_crate_dir;

/// Write the worker crate's `Cargo.toml`: a `cgp` path dependency plus a path
/// dependency on each auxiliary crate the current fixture declared (see
/// [`crate::aux`]). The empty `[workspace]` table stops cargo from folding the
/// crate into the cargo-cgp workspace above it.
///
/// Rewritten per fixture, because which auxiliary crates are needed varies by
/// fixture, but only when the content actually changes, so a run of ordinary
/// (no-aux) fixtures leaves the manifest untouched and never disturbs cargo's
/// fingerprints for the cached `cgp` build.
pub fn write_manifest(harness_crate: &Path, aux: &[(&str, &Path)]) {
    let mut deps = format!("cgp = {{ path = \"{}\" }}\n", cgp_crate_dir().display());
    for (name, dir) in aux {
        deps.push_str(&format!("{name} = {{ path = \"{}\" }}\n", dir.display()));
    }
    let manifest = format!(
        "[package]\n\
         name    = \"ui\"\n\
         version = \"0.0.0\"\n\
         edition = \"2024\"\n\
         publish = false\n\
         \n\
         [dependencies]\n\
         {deps}\
         \n\
         [workspace]\n",
    );

    let path = harness_crate.join("Cargo.toml");
    if fs::read_to_string(&path).ok().as_deref() != Some(manifest.as_str()) {
        fs::write(&path, manifest).expect("writing the throwaway manifest");
    }
}
