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
//! committed JSON and runs only `process_cgp_errors`, needing no compilation. See the
//! [testing document](../../docs/implementation/testing.md).

pub mod fixtures;
pub mod harness;
pub mod normalize;
pub mod options;
pub mod passes;
pub mod paths;
pub mod snapshot;

use std::path::Path;

use crate::options::Options;
use crate::snapshot::Outcome;

/// Run the UI suite. `args` are the harness arguments (everything cargo passes after
/// `--`): `--bless` to regenerate snapshots, `--print` to print each fixture's raw output
/// instead of comparing, `--process-only` to run just the `process_cgp_errors` unit pass,
/// and any bare words as path substring filters.
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

    // The two cargo-invoking passes need the built binaries and the throwaway crate; the
    // process-only pass needs neither, so skip the slow setup when it is all we run.
    let harness_crate = if options.process_only {
        paths::harness_crate_dir()
    } else {
        harness::build_binaries();
        harness::ensure_harness_crate()
    };

    let mut failed = 0usize;
    for fixture in &fixtures {
        let name = fixture
            .strip_prefix(&fixtures_dir)
            .unwrap_or(fixture)
            .display()
            .to_string();

        if options.print {
            print_fixture(&options, &harness_crate, fixture, &name);
            continue;
        }

        let outcomes = run_passes(&options, &harness_crate, fixture, &cgp_root);
        if report(&name, &outcomes) {
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

/// Print one line summarizing a fixture's passes. Returns `true` if any pass mismatched.
fn report(name: &str, outcomes: &[(&str, Outcome)]) -> bool {
    let mismatched: Vec<&str> = outcomes
        .iter()
        .filter(|(_, outcome)| matches!(outcome, Outcome::Mismatch))
        .map(|(label, _)| *label)
        .collect();

    if !mismatched.is_empty() {
        println!("MISMATCH {name}  ({})", mismatched.join(", "));
        true
    } else if outcomes
        .iter()
        .any(|(_, outcome)| matches!(outcome, Outcome::Blessed))
    {
        println!("blessed  {name}");
        false
    } else {
        println!("ok       {name}");
        false
    }
}

/// Print a fixture's raw output for interactive inspection: the tool's own stderr in full
/// mode, or the process pass's rendered output in process-only mode.
fn print_fixture(options: &Options, harness_crate: &Path, fixture: &Path, name: &str) {
    println!("===== {name} =====");
    if options.process_only {
        print!("{}", passes::print_process_output(fixture));
    } else {
        print!("{}", harness::run_fixture(harness_crate, fixture));
    }
    println!("===== end {name} =====");
}
