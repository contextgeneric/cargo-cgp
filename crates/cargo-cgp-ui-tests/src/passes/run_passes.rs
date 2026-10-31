//! Running every pass and check for one fixture.

use std::path::Path;

use crate::directives::Directives;
use crate::options::Options;
use crate::passes::checks::{header_check, pass_through_check, status_check};
use crate::passes::pass_paths::Paths;
use crate::passes::snapshot_passes::{cgp_stderr_pass, expand_pass, json_pass, rust_stderr_pass};
use crate::snapshot::Outcome;
use crate::{harness, normalize};

/// Run the passes and checks for one fixture and return each one's label and outcome: the rust
/// pass records the plain-compiler `.rust.stderr` baseline, the cgp pass owns the tool's
/// `.cgp.stderr`, the expand pass owns the `.expand.rs` generated code, the status check compares
/// the two compilers' exit statuses, and, where they apply, the pass-through check and the JSON
/// pass.
pub fn run_passes(
    options: &Options,
    fixture: &Path,
    name: &str,
    directives: &Directives,
    paths: &Paths<'_>,
) -> Vec<(&'static str, Outcome)> {
    let rust = harness::run_fixture_rust(paths.harness_crate, fixture);
    let cgp = harness::run_fixture(paths.harness_crate, fixture, &directives.env);

    let mut outcomes = vec![
        (
            "rust",
            rust_stderr_pass(&rust, fixture, paths, options.bless),
        ),
        ("cgp", cgp_stderr_pass(&cgp, fixture, paths, options.bless)),
        (
            "expand",
            expand_pass(fixture, &directives.env, paths, options.bless),
        ),
        ("status", status_check(rust.success, cgp.success)),
    ];

    let exempt = ["ok/", "pass-through/", "robustness/"]
        .iter()
        .any(|dir| name.starts_with(dir));
    let tool_output = normalize::normalize(
        &cgp.stderr,
        paths.harness_crate,
        paths.cgp_root,
        paths.sysroot,
    );
    outcomes.push(("header", header_check(exempt, directives, &tool_output)));

    if name.starts_with("pass-through/") {
        let baseline = |stderr: &str| {
            normalize::normalize_baseline(
                stderr,
                paths.harness_crate,
                paths.cgp_root,
                paths.sysroot,
            )
        };
        outcomes.push((
            "pass-through",
            pass_through_check(
                &baseline(&rust.stderr),
                &baseline(&cgp.stderr),
                directives.rewrites_falsely.as_deref(),
            ),
        ));
    }

    if directives.json {
        outcomes.push((
            "json",
            json_pass(fixture, &directives.env, paths, options.bless),
        ));
    }

    outcomes
}
