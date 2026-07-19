//! A custom UI-test harness for `cargo-cgp`, modeled on Clippy's `compile-test`.
//!
//! Each fixture under `tests/ui/` is checked through two passes (see [`passes`]): one runs
//! the whole tool end to end (front-end and driver) and pins its rendered `.cgp.stderr`, so
//! when the tool reformats diagnostics that snapshot is what changes; the other runs plain
//! `cargo check` to record the untransformed `.rust.stderr` baseline the tool improves on.
//! It is driven by the `harness = false` test in [`tests/ui.rs`](../../tests/ui.rs), which
//! calls [`run`]; the logic lives here so it stays small and out of the `bin`/test
//! entrypoint. Fixtures are checked in parallel across a pool of workers (see [`runner`]).
//! See the [testing document](../../docs/implementation/testing.md).

pub mod fixtures;
pub mod harness;
pub mod normalize;
pub mod options;
pub mod passes;
pub mod paths;
pub mod runner;
pub mod snapshot;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::options::Options;
use crate::snapshot::Outcome;

/// Run the UI suite. `args` are the harness arguments (everything cargo passes after
/// `--`): `--bless` to regenerate snapshots, `--print` to print each fixture's raw output
/// instead of comparing, `--jobs N` (`-j N`) to set the worker count, and any bare words as
/// path substring filters.
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

    // Each worker checks fixtures in its own throwaway crate, so both passes need those
    // crates (and the built binaries) first.
    harness::build_binaries();
    let workers: Vec<PathBuf> = harness::ensure_worker_crates(jobs);

    if !options.print {
        eprintln!(
            "checking {} fixture(s) across {jobs} worker(s)",
            fixtures.len()
        );
    }

    let fixture_refs: Vec<&Path> = fixtures.iter().map(PathBuf::as_path).collect();
    let failed = AtomicUsize::new(0);
    runner::run(
        jobs,
        &fixture_refs,
        |worker, fixture| {
            let crate_dir = &workers[worker];
            let name = fixture
                .strip_prefix(&fixtures_dir)
                .unwrap_or(fixture)
                .display()
                .to_string();

            if options.print {
                Report {
                    text: print_block(crate_dir, fixture, &name),
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
        },
        // Print each fixture as it finishes and flush, so the run streams live; the
        // runner serializes this, so blocks never interleave. Order is completion order.
        |report: Report| {
            let mut out = std::io::stdout().lock();
            let _ = out.write_all(report.text.as_bytes());
            let _ = out.flush();
            if report.failed {
                failed.fetch_add(1, Ordering::Relaxed);
            }
        },
    );

    let failed = failed.into_inner();
    if failed > 0 {
        eprintln!(
            "\n{failed} fixture(s) with a snapshot mismatch; re-run with `--bless` to update after an intended change"
        );
        std::process::exit(1);
    }
}

/// One fixture's contribution to the run: the text to print for it (already fully
/// rendered) and whether it counts as a failure. Built on a worker thread, then handed to
/// the runner's `done` callback, which prints it live as the fixture finishes.
struct Report {
    text: String,
    failed: bool,
}

/// Run the passes for one fixture and return each pass's label and outcome: the rust pass
/// records the plain-compiler `.rust.stderr` baseline, and the cgp-stderr pass owns the
/// tool's `.cgp.stderr` output.
fn run_passes(
    options: &Options,
    harness_crate: &Path,
    fixture: &Path,
    cgp_root: &Path,
) -> Vec<(&'static str, Outcome)> {
    vec![
        (
            "rust",
            passes::rust_stderr_pass(harness_crate, fixture, cgp_root, options.bless),
        ),
        (
            "cgp",
            passes::cgp_stderr_pass(harness_crate, fixture, cgp_root, options.bless),
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

/// Render a fixture's raw output for interactive inspection — the tool's own stderr.
fn print_block(harness_crate: &Path, fixture: &Path, name: &str) -> String {
    let body = harness::run_fixture(harness_crate, fixture);
    format!("===== {name} =====\n{body}===== end {name} =====\n")
}
