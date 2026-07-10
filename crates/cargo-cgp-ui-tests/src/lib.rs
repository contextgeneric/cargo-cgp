//! A custom UI-test harness for `cargo-cgp`, modeled on Clippy's `compile-test`.
//!
//! Each fixture under `tests/ui/` is checked through three passes that must agree, so the
//! tool's real output, the diagnostics it captures, and its pure processing pipeline all
//! stay consistent (see [`passes`]). It is driven by the `harness = false` test in
//! [`tests/ui.rs`](../../tests/ui.rs), which calls [`run`]; the logic lives here so it
//! stays small and out of the `bin`/test entrypoint.
//!
//! Two passes run the whole tool end to end (front-end and driver), so when the tool
//! begins reformatting diagnostics these snapshots are what change; the third parses the
//! committed JSON and runs only `process_cgp_errors`, needing no compilation. Fixtures are
//! checked in parallel across a pool of workers (see [`runner`]). See the
//! [testing document](../../docs/implementation/testing.md).

pub mod fixtures;
pub mod harness;
pub mod normalize;
pub mod options;
pub mod passes;
pub mod paths;
pub mod runner;
pub mod snapshot;

use std::path::{Path, PathBuf};

use crate::options::Options;
use crate::snapshot::Outcome;

/// Run the UI suite. `args` are the harness arguments (everything cargo passes after
/// `--`): `--bless` to regenerate snapshots, `--print` to print each fixture's raw output
/// instead of comparing, `--process-only` to run just the `process_cgp_errors` unit pass,
/// `--jobs N` (`-j N`) to set the worker count, and any bare words as path substring
/// filters.
///
/// Exits the process: `0` if every fixture matched (or was blessed/printed), `1` on a
/// snapshot mismatch, `2` if no fixture matched the filters.
pub fn run(args: Vec<String>) {
    let options = Options::parse(args);

    let fixtures_dir = paths::fixtures_dir();
    let fixtures = fixtures::collect(&fixtures_dir, &options);
    if fixtures.is_empty() {
        eprintln!("no fixtures matched");
        std::process::exit(2);
    }

    let cgp_root = paths::cgp_root();
    let jobs = options
        .jobs
        .unwrap_or_else(|| runner::default_jobs(fixtures.len()))
        .clamp(1, fixtures.len());

    // Each worker checks fixtures in its own throwaway crate, so the two cargo-invoking
    // passes need those crates (and the built binaries) first. The process-only pass
    // compiles nothing, so it just needs each worker's crate path to normalize output.
    let workers: Vec<PathBuf> = if options.process_only {
        (0..jobs).map(paths::worker_crate_dir).collect()
    } else {
        harness::build_binaries();
        harness::ensure_worker_crates(jobs)
    };

    if !options.print {
        eprintln!(
            "checking {} fixture(s) across {jobs} worker(s)",
            fixtures.len()
        );
    }

    let fixture_refs: Vec<&Path> = fixtures.iter().map(PathBuf::as_path).collect();
    let reports = runner::run(jobs, &fixture_refs, |worker, fixture| {
        let crate_dir = &workers[worker];
        let name = fixture
            .strip_prefix(&fixtures_dir)
            .unwrap_or(fixture)
            .display()
            .to_string();

        if options.print {
            Report {
                text: print_block(&options, crate_dir, fixture, &name),
                failed: false,
            }
        } else {
            let outcomes = run_passes(&options, crate_dir, fixture, &cgp_root);
            let failed = outcomes
                .iter()
                .any(|(_, outcome)| matches!(outcome, Outcome::Mismatch(_)));
            Report {
                text: report_block(&name, &outcomes),
                failed,
            }
        }
    });

    let mut failed = 0usize;
    for report in &reports {
        print!("{}", report.text);
        if report.failed {
            failed += 1;
        }
    }

    if failed > 0 {
        eprintln!(
            "\n{failed} fixture(s) with a snapshot mismatch; re-run with `--bless` to update after an intended change"
        );
        std::process::exit(1);
    }
}

/// One fixture's contribution to the run: the text to print for it (already fully
/// rendered) and whether it counts as a failure. Held rather than printed so a parallel
/// run can emit fixtures in order regardless of which worker finished first.
struct Report {
    text: String,
    failed: bool,
}

/// Run the passes for one fixture and return each pass's label and outcome. In
/// process-only mode this is just the unit pass (which may bless `.stderr`); otherwise it
/// is all three, and the process pass only verifies — the stderr pass owns `.stderr`.
fn run_passes(
    options: &Options,
    harness_crate: &Path,
    fixture: &Path,
    cgp_root: &Path,
) -> Vec<(&'static str, Outcome)> {
    if options.process_only {
        return vec![(
            "process",
            passes::process_pass(harness_crate, fixture, cgp_root, options.bless),
        )];
    }

    vec![
        (
            "stderr",
            passes::stderr_pass(harness_crate, fixture, cgp_root, options.bless),
        ),
        (
            "json",
            passes::json_pass(harness_crate, fixture, cgp_root, options.bless),
        ),
        (
            "process",
            passes::process_pass(harness_crate, fixture, cgp_root, false),
        ),
    ]
}

/// Render one fixture's summary line, plus any mismatch diffs beneath it. Returns the
/// block ending in a newline so the caller can print blocks back to back.
fn report_block(name: &str, outcomes: &[(&str, Outcome)]) -> String {
    let mut diffs = String::new();
    let mut mismatched: Vec<&str> = Vec::new();
    let mut blessed = false;

    for (label, outcome) in outcomes {
        match outcome {
            Outcome::Mismatch(diff) => {
                mismatched.push(label);
                diffs.push_str(diff);
            }
            Outcome::Blessed => blessed = true,
            Outcome::Ok => {}
        }
    }

    if !mismatched.is_empty() {
        format!("MISMATCH {name}  ({})\n{diffs}", mismatched.join(", "))
    } else if blessed {
        format!("blessed  {name}\n")
    } else {
        format!("ok       {name}\n")
    }
}

/// Render a fixture's raw output for interactive inspection: the tool's own stderr in full
/// mode, or the process pass's rendered output in process-only mode.
fn print_block(options: &Options, harness_crate: &Path, fixture: &Path, name: &str) -> String {
    let body = if options.process_only {
        passes::print_process_output(fixture)
    } else {
        harness::run_fixture(harness_crate, fixture)
    };
    format!("===== {name} =====\n{body}===== end {name} =====\n")
}
