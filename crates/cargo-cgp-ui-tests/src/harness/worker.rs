//! The throwaway crate each worker compiles fixtures in.

use std::fs;
use std::path::PathBuf;

use crate::harness::manifest::write_manifest;
use crate::paths::worker_crate_dir;

/// Create or refresh the `count` per-worker throwaway crates and return their directories.
/// Giving each worker its own crate is what lets fixtures run in parallel: two workers
/// never share a `src/main.rs` nor a cargo target lock. The cost is that cgp is built once
/// per worker rather than once overall, since each crate has its own target directory.
pub fn ensure_worker_crates(count: usize) -> Vec<PathBuf> {
    (0..count).map(ensure_worker_crate).collect()
}

/// Create or refresh one worker's throwaway crate and return its directory. Every worker
/// crate is identical apart from its directory: it lives under `target/` (git-ignored) and
/// is named `ui` (a stable package name keeps cargo's output stable regardless of which
/// worker ran a fixture, since the differing directory is normalized to `$DIR`), and the empty
/// `[workspace]` table stops cargo from folding it into the cargo-cgp workspace above it.
pub fn ensure_worker_crate(index: usize) -> PathBuf {
    let dir = worker_crate_dir(index);
    fs::create_dir_all(dir.join("src")).expect("creating the throwaway crate");

    write_manifest(&dir, &[]);

    let main = dir.join("src/main.rs");
    if !main.exists() {
        fs::write(&main, "fn main() {}\n").expect("seeding the throwaway main.rs");
    }

    dir
}
