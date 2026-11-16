//! The passes that compare a fixture's output with a committed snapshot.

use std::path::Path;
use std::time::Duration;

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

/// Review a fixture's `cargo cgp expand` output against `.expand.rs`: the Rust
/// its CGP macros generate, with CGP's type-level constructs resugared.
pub fn expand_pass(expansion: &str, fixture: &Path, paths: &Paths<'_>, bless: bool) -> Outcome {
    let actual = normalize_source(
        expansion,
        paths.harness_crate,
        paths.cgp_root,
        paths.sysroot,
    );
    review(&expand_path(fixture), &actual, bless, "expand")
}

/// Run `cargo cgp check --message-format=json` on the fixture and review its compiler messages,
/// reduced by [`reduce_json`], against `.cgp.json`. `skip` leaves the snapshot uncompared, for a
/// fixture whose output rustc varies under the run's threads. The command's timeout, if any, is
/// returned beside the outcome, for the timeout check.
pub fn json_pass(
    fixture: &Path,
    env: &[(String, String)],
    paths: &Paths<'_>,
    bless: bool,
    skip: bool,
    timeout: Duration,
) -> (Outcome, bool) {
    let (stream, timed_out) = harness::run_fixture_json(paths.harness_crate, fixture, env, timeout);
    if skip {
        return (Outcome::Ok, timed_out);
    }
    let reduced = reduce_json(&stream);
    let actual = replace_paths(&reduced, paths.harness_crate, paths.cgp_root, paths.sysroot);
    (
        review(&json_path(fixture), &actual, bless, "json"),
        timed_out,
    )
}
