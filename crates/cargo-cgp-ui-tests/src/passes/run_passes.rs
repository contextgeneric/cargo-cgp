//! Running every pass and check for one fixture.

use std::path::Path;

use crate::directives::Directives;
use crate::options::Options;
use crate::passes::checks::{
    expand_status_check, header_check, pass_through_check, status_check, timeout_check,
};
use crate::passes::pass_paths::Paths;
use crate::passes::snapshot_passes::{cgp_stderr_pass, expand_pass, json_pass, rust_stderr_pass};
use crate::snapshot::Outcome;
use crate::{harness, normalize};

/// Run the passes and checks for one fixture and return each one's label and outcome: the rust
/// pass records the plain-compiler `.rust.stderr` baseline, the cgp pass owns the tool's
/// `.cgp.stderr`, the expand pass owns the `.expand.rs` generated code, the status check compares
/// the two compilers' exit statuses, the timeout check reports a command killed at its deadline,
/// and, where they apply, the pass-through check and the JSON pass.
///
/// `tool_env` is the environment of the tool's passes, from [`harness::tool_env`]. Under a
/// parallel `--threads`, a fixture marked `//@varies-under-threads:` skips the tool's snapshot
/// comparisons and keeps every check.
pub fn run_passes(
    options: &Options,
    fixture: &Path,
    name: &str,
    directives: &Directives,
    tool_env: &[(String, String)],
    paths: &Paths<'_>,
) -> Vec<(&'static str, Outcome)> {
    let timeout = options.timeout;
    let rust = harness::run_fixture_rust(paths.harness_crate, fixture, timeout);
    let cgp = harness::run_fixture(paths.harness_crate, fixture, tool_env, timeout);
    let expand = harness::run_fixture_expand(paths.harness_crate, fixture, tool_env, timeout);

    let skip_tool_snapshots =
        options.threads.is_parallel() && directives.varies_under_threads.is_some();
    let cgp_outcome = if skip_tool_snapshots {
        Outcome::Ok
    } else {
        cgp_stderr_pass(&cgp, fixture, paths, options.bless)
    };

    let mut outcomes = vec![
        (
            "rust",
            rust_stderr_pass(&rust, fixture, paths, options.bless),
        ),
        ("cgp", cgp_outcome),
        (
            "expand",
            expand_pass(&expand.expansion, fixture, paths, options.bless),
        ),
        ("status", status_check(rust.success, cgp.success)),
        (
            "expand status",
            expand_status_check(expand.run.success, &expand.run.stderr),
        ),
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

    let mut json_timed_out = false;
    if directives.json {
        let (outcome, timed_out) = json_pass(
            fixture,
            tool_env,
            paths,
            options.bless,
            skip_tool_snapshots,
            timeout,
        );
        json_timed_out = timed_out;
        outcomes.push(("json", outcome));
    }

    let commands = [
        ("cargo check", rust.timed_out),
        ("cargo cgp check", cgp.timed_out),
        ("cargo cgp expand", expand.run.timed_out),
        ("cargo cgp check --message-format=json", json_timed_out),
    ];
    outcomes.push(("timeout", timeout_check(&commands, timeout)));

    outcomes
}
