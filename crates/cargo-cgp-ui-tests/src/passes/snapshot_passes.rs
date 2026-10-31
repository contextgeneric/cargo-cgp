//! The passes that compare a fixture's output with a committed snapshot.

use std::path::Path;

use crate::harness::{self, Run};
use crate::normalize::{normalize, normalize_baseline, normalize_source, replace_paths};
use crate::passes::pass_paths::Paths;
use crate::passes::reduce_json::reduce_json;
use crate::snapshot::{Outcome, cgp_stderr_path, expand_path, json_path, review, rust_stderr_path};

/// Review the tool's stderr from `run` against `.cgp.stderr`.
pub fn cgp_stderr_pass(run: &Run, fixture: &Path, paths: &Paths<'_>, bless: bool) -> Outcome {
    let actual = normalize(
        &run.stderr,
        paths.harness_crate,
        paths.cgp_root,
        paths.sysroot,
    );
    review(&cgp_stderr_path(fixture), &actual, bless, "cgp")
}

/// Review plain `cargo check`'s stderr from `run` against `.rust.stderr`.
pub fn rust_stderr_pass(run: &Run, fixture: &Path, paths: &Paths<'_>, bless: bool) -> Outcome {
    let actual = normalize_baseline(
        &run.stderr,
        paths.harness_crate,
        paths.cgp_root,
        paths.sysroot,
    );
    review(&rust_stderr_path(fixture), &actual, bless, "rust")
}

/// Run `cargo cgp expand` on the fixture and review its expansion against `.expand.rs` — the Rust
/// its CGP macros generate, with CGP's type-level constructs resugared.
pub fn expand_pass(
    fixture: &Path,
    env: &[(String, String)],
    paths: &Paths<'_>,
    bless: bool,
) -> Outcome {
    let raw = harness::run_fixture_expand(paths.harness_crate, fixture, env);
    let actual = normalize_source(&raw, paths.harness_crate, paths.cgp_root, paths.sysroot);
    review(&expand_path(fixture), &actual, bless, "expand")
}

/// Run `cargo cgp check --message-format=json` on the fixture and review its compiler messages,
/// reduced by [`reduce_json`], against `.cgp.json`.
pub fn json_pass(
    fixture: &Path,
    env: &[(String, String)],
    paths: &Paths<'_>,
    bless: bool,
) -> Outcome {
    let stream = harness::run_fixture_json(paths.harness_crate, fixture, env);
    let reduced = reduce_json(&stream);
    let actual = replace_paths(&reduced, paths.harness_crate, paths.cgp_root, paths.sysroot);
    review(&json_path(fixture), &actual, bless, "json")
}
