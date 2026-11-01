//! Where a fixture's snapshots live: beside it, named by what each records.

use std::path::{Path, PathBuf};

/// The `.cgp.stderr` snapshot path beside a fixture (`foo.rs` → `foo.cgp.stderr`): the
/// output `cargo-cgp` renders, the "after" the tool exists to produce.
pub fn cgp_stderr_path(fixture: &Path) -> PathBuf {
    fixture.with_extension("cgp.stderr")
}

/// The `.rust.stderr` snapshot path beside a fixture (`foo.rs` → `foo.rust.stderr`): the
/// output plain `cargo check` produces for the same fixture, recorded as the "before" so a
/// reader can see what `cargo-cgp` changes.
pub fn rust_stderr_path(fixture: &Path) -> PathBuf {
    fixture.with_extension("rust.stderr")
}

/// The `.expand.rs` snapshot path beside a fixture (`foo.rs` → `foo.expand.rs`): the Rust the
/// fixture's CGP macros generate, as `cargo cgp expand` shows it. It records what the macros
/// *produce*, where the two `.stderr` snapshots record what the compiler says about it.
pub fn expand_path(fixture: &Path) -> PathBuf {
    fixture.with_extension("expand.rs")
}

/// The `.cgp.json` snapshot path beside a fixture (`foo.rs` → `foo.cgp.json`): the tool's
/// diagnostics as cargo's JSON stream carries them, for the fixtures that ask for the JSON pass.
pub fn json_path(fixture: &Path) -> PathBuf {
    fixture.with_extension("cgp.json")
}
