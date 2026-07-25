//! The verification passes each fixture goes through.
//!
//! Three passes run per fixture. Two are about what the compiler *says* — the tool's transformed
//! diagnostics and the untransformed baseline they improve on — and the third is about what the
//! macros *generate*:
//!
//! - [`cgp_stderr_pass`] runs `cargo-cgp` directly and compares its stderr to `.cgp.stderr` —
//!   the end-to-end check that the whole binary produces the expected output.
//! - [`rust_stderr_pass`] runs plain `cargo check` and records its stderr in `.rust.stderr`,
//!   the "before" the tool improves on. It stands alone — nothing cross-checks it, because it
//!   is the untransformed compiler output, not a tool result.
//! - [`expand_pass`] runs `cargo cgp expand` and compares its stdout to `.expand.rs`, so every
//!   fixture also pins the Rust its CGP macros generate. That makes the snapshot pair
//!   complementary: `.cgp.stderr` is what the tool says about the code, `.expand.rs` is the code
//!   the compiler was actually given — which is where the answer to "why does it say that?"
//!   usually is. It is also the end-to-end coverage of the expand command and of the
//!   syntax-tree resugaring it drives.
//!
//! There is no separate capture-or-process pass: the driver applies every CGP transform
//! in-process and renders the result, so `.cgp.stderr` is simply what `cargo-cgp` prints.

use std::path::Path;

use crate::harness;
use crate::normalize::{normalize, normalize_source};
use crate::snapshot::{Outcome, cgp_stderr_path, expand_path, review, rust_stderr_path};

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

/// Run `cargo cgp expand` on the fixture and review its expansion against `.expand.rs` — the Rust
/// its CGP macros generate, with CGP's type-level constructs resugared.
pub fn expand_pass(harness_crate: &Path, fixture: &Path, cgp_root: &Path, bless: bool) -> Outcome {
    let raw = harness::run_fixture_expand(harness_crate, fixture);
    let actual = normalize_source(&raw, harness_crate, cgp_root);
    review(&expand_path(fixture), &actual, bless)
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
