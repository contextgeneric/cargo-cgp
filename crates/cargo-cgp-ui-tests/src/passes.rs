//! The verification passes each fixture goes through.
//!
//! Two passes run per fixture. The tool pass renders the transformed diagnostics; the plain
//! pass records the untransformed compiler baseline the tool improves on:
//!
//! - [`cgp_stderr_pass`] runs `cargo-cgp` directly and compares its stderr to `.cgp.stderr` —
//!   the end-to-end check that the whole binary produces the expected output.
//! - [`rust_stderr_pass`] runs plain `cargo check` and records its stderr in `.rust.stderr`,
//!   the "before" the tool improves on. It stands alone — nothing cross-checks it, because it
//!   is the untransformed compiler output, not a tool result.
//!
//! There is no separate capture-or-process pass: the driver applies every CGP transform
//! in-process and renders the result, so `.cgp.stderr` is simply what `cargo-cgp` prints.

use std::path::Path;

use crate::harness;
use crate::normalize::normalize;
use crate::snapshot::{Outcome, cgp_stderr_path, review, rust_stderr_path};

/// Run `cargo-cgp` on the fixture and review its stderr against `.cgp.stderr`.
pub fn cgp_stderr_pass(
    harness_crate: &Path,
    fixture: &Path,
    cgp_root: &Path,
    bless: bool,
) -> Outcome {
    let raw = harness::run_fixture(harness_crate, fixture);
    let actual = normalize(&raw, harness_crate, cgp_root);
    review(&cgp_stderr_path(fixture), &actual, bless)
}

/// Run plain `cargo check` on the fixture and review its stderr against `.rust.stderr` — the
/// original compiler output the tool sets out to improve, recorded here so the diff against
/// `.cgp.stderr` shows what `cargo-cgp` changes.
pub fn rust_stderr_pass(
    harness_crate: &Path,
    fixture: &Path,
    cgp_root: &Path,
    bless: bool,
) -> Outcome {
    let raw = harness::run_fixture_rust(harness_crate, fixture);
    let actual = normalize(&raw, harness_crate, cgp_root);
    review(&rust_stderr_path(fixture), &actual, bless)
}
