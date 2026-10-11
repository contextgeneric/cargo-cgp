//! A custom UI-test harness for `cargo-cgp`, modeled on Clippy's `compile-test`.
//!
//! Each fixture under `tests/ui/` is checked through three passes (see [`passes`]): one runs
//! the whole tool end to end (front-end and driver) and pins its rendered `.cgp.stderr`, so
//! when the tool reformats diagnostics that snapshot is what changes; one runs plain
//! `cargo check` to record the untransformed `.rust.stderr` baseline the tool improves on; and
//! one runs `cargo cgp expand` to pin the `.expand.rs` code the fixture's CGP macros generate.
//! Two checks compare the passes with each other (the exit status, and for `pass-through/`
//! fixtures the output itself), and a fixture's `//@json` directive adds a JSON pass (see
//! [`directives`]).
//! It is driven by the `harness = false` test in [`tests/ui.rs`](../../tests/ui.rs), which
//! calls [`run`]; the logic lives here so it stays small and out of the `bin`/test
//! entrypoint. Fixtures are checked in parallel across a pool of workers (see [`runner`]).
//! See cgp-knowledge-base/cargo-cgp/implementation/testing.md.

pub mod aux;
pub mod directives;
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
    let sysroot = paths::sysroot();
    let jobs = options
        .jobs
        .unwrap_or_else(|| runner::default_jobs(fixtures.len()))
        .clamp(1, fixtures.len());

    // Each worker checks fixtures in its own throwaway crate, so every pass needs those
    // crates (and the built binaries) first.
    harness::build_binaries();
    let workers: Vec<PathBuf> = harness::ensure_worker_crates(jobs);
    // Auxiliary crates a fixture can depend on via a `//@aux-build:` directive are
    // materialized once, up front, and shared read-only across workers.
    let aux_crates = aux::materialize_all();

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

            // Point the worker crate at whatever auxiliary crates this fixture declares
            // (usually none) before any pass copies the fixture in and compiles it.
            let fixture_aux = aux::required(fixture, &aux_crates);
            harness::write_manifest(crate_dir, &fixture_aux);

            if options.print {
                Report {
                    text: print_block(crate_dir, fixture, &name),
                    failed: false,
                }
            } else {
                let paths = passes::Paths {
                    harness_crate: crate_dir,
                    cgp_root: &cgp_root,
                    sysroot: &sysroot,
                };
                let outcomes = run_passes(&options, fixture, &name, &paths);
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

/// Run the passes and checks for one fixture and return each one's label and outcome: the rust
/// pass records the plain-compiler `.rust.stderr` baseline, the cgp pass owns the tool's
/// `.cgp.stderr`, the expand pass owns the `.expand.rs` generated code, the status check compares
/// the two compilers' exit statuses, and, where they apply, the pass-through check and the JSON
/// pass.
fn run_passes(
    options: &Options,
    fixture: &Path,
    name: &str,
    paths: &passes::Paths,
) -> Vec<(&'static str, Outcome)> {
    let directives = directives::read(fixture);
    let rust = harness::run_fixture_rust(paths.harness_crate, fixture);
    let cgp = harness::run_fixture(paths.harness_crate, fixture);

    let mut outcomes = vec![
        (
            "rust",
            passes::rust_stderr_pass(&rust, fixture, paths, options.bless),
        ),
        (
            "cgp",
            passes::cgp_stderr_pass(&cgp, fixture, paths, options.bless),
        ),
        ("expand", passes::expand_pass(fixture, paths, options.bless)),
        ("status", passes::status_check(rust.success, cgp.success)),
    ];

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
            passes::pass_through_check(
                &baseline(&rust.stderr),
                &baseline(&cgp.stderr),
                directives.rewrites_falsely.as_deref(),
            ),
        ));
    }

    if directives.json {
        outcomes.push(("json", passes::json_pass(fixture, paths, options.bless)));
    }

    outcomes
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

/// Render a fixture's raw output for interactive inspection: the tool's stderr, plain
/// `cargo check`'s stderr, and the expansion, each unnormalized and labelled with its exit status.
fn print_block(harness_crate: &Path, fixture: &Path, name: &str) -> String {
    let status = |ok: bool| if ok { "success" } else { "failure" };
    let cgp = harness::run_fixture(harness_crate, fixture);
    let rust = harness::run_fixture_rust(harness_crate, fixture);
    let expand = harness::run_fixture_expand(harness_crate, fixture);
    format!(
        "===== {name}: cargo cgp check ({}) =====\n{}\
         ===== {name}: cargo check ({}) =====\n{}\
         ===== {name}: cargo cgp expand =====\n{}\
         ===== end {name} =====\n",
        status(cgp.success),
        cgp.stderr,
        status(rust.success),
        rust.stderr,
        expand,
    )
}
