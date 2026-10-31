//! One run of the UI suite: its setup, the parallel fixture loop, and the summary.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::blocks::{list_block, print_block, report_block};
use crate::options::Options;
use crate::passes::run_passes;
use crate::snapshot::Outcome;
use crate::{aux, directives, fixtures, harness, passes, paths, runner};

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

    if options.list {
        print!("{}", list_block(&fixtures_dir, &fixtures));
        return;
    }

    let setup = prepare(&options, fixtures.len());

    if !options.print {
        eprintln!(
            "checking {} fixture(s) across {} worker(s)",
            fixtures.len(),
            setup.jobs
        );
    }

    let failed = check_fixtures(&options, &fixtures_dir, &fixtures, &setup);
    exit_on_failures(failed);
}

/// What every fixture's check needs, prepared once before the workers start.
pub struct Setup {
    /// The number of workers.
    pub jobs: usize,
    /// The sibling `cgp` checkout.
    pub cgp_root: PathBuf,
    /// The toolchain sysroot.
    pub sysroot: PathBuf,
    /// Each worker's throwaway crate.
    pub workers: Vec<PathBuf>,
    /// The materialized auxiliary crates, by name.
    pub aux_crates: Vec<(String, PathBuf)>,
}

/// Resolve the paths and the worker count, build the binaries, and create the crates the workers use.
pub fn prepare(options: &Options, fixture_count: usize) -> Setup {
    let cgp_root = paths::cgp_root();
    let sysroot = paths::sysroot();
    let jobs = options
        .jobs
        .unwrap_or_else(|| runner::default_jobs(fixture_count))
        .clamp(1, fixture_count);

    // Each worker checks fixtures in its own throwaway crate, so every pass needs those
    // crates (and the built binaries) first.
    harness::build_binaries();
    let workers: Vec<PathBuf> = harness::ensure_worker_crates(jobs);
    // Auxiliary crates a fixture can depend on via a `//@aux-build:` directive are
    // materialized once, up front, and shared read-only across workers.
    let aux_crates = aux::materialize_all();

    Setup {
        jobs,
        cgp_root,
        sysroot,
        workers,
        aux_crates,
    }
}

/// Check every fixture across the workers, printing each as it finishes, and return how many failed.
pub fn check_fixtures(
    options: &Options,
    fixtures_dir: &Path,
    fixtures: &[PathBuf],
    setup: &Setup,
) -> usize {
    let fixture_refs: Vec<&Path> = fixtures.iter().map(PathBuf::as_path).collect();
    let failed = AtomicUsize::new(0);
    runner::run(
        setup.jobs,
        &fixture_refs,
        |worker, fixture| check_fixture(options, fixtures_dir, setup, worker, fixture),
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
    failed.into_inner()
}

/// Check one fixture in `worker`'s crate, or print its raw output under `--print`.
pub fn check_fixture(
    options: &Options,
    fixtures_dir: &Path,
    setup: &Setup,
    worker: usize,
    fixture: &Path,
) -> Report {
    let crate_dir = &setup.workers[worker];
    let name = fixture
        .strip_prefix(fixtures_dir)
        .unwrap_or(fixture)
        .display()
        .to_string();

    let directives = directives::read(fixture);

    // Point the worker crate at whatever auxiliary crates this fixture declares
    // (usually none) before any pass copies the fixture in and compiles it.
    let fixture_aux = aux::required(&directives.aux_build, &setup.aux_crates);
    harness::write_manifest(crate_dir, &fixture_aux);

    if options.print {
        Report {
            text: print_block(crate_dir, fixture, &name, &directives.env),
            failed: false,
        }
    } else {
        let paths = passes::Paths {
            harness_crate: crate_dir,
            cgp_root: &setup.cgp_root,
            sysroot: &setup.sysroot,
        };
        let outcomes = run_passes(options, fixture, &name, &directives, &paths);
        let failed = outcomes
            .iter()
            .any(|(_, outcome)| matches!(outcome, Outcome::Mismatch(_)));
        Report {
            text: report_block(&name, &outcomes),
            failed,
        }
    }
}

/// Exit with `1` and say how to bless when any fixture failed.
pub fn exit_on_failures(failed: usize) {
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
pub struct Report {
    /// The rendered text to print.
    pub text: String,
    /// Whether the fixture counts as a failure.
    pub failed: bool,
}
