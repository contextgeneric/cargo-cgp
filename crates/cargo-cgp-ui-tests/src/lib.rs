//! A custom UI-test harness for `cargo-cgp`, modeled on Clippy's `compile-test`.
//!
//! The suite compiles each fixture under `tests/ui/` through the real `cargo-cgp` and
//! compares the tool's output against a committed `.stderr` snapshot beside the
//! fixture. It is driven by the `harness = false` test in
//! [`tests/ui.rs`](../../tests/ui.rs), which calls [`run`]; the logic lives here so it
//! stays small and out of the `bin`/test entrypoint.
//!
//! The output snapshotted is `cargo-cgp`'s own, produced by running the whole tool end
//! to end (front-end and driver) — so when the driver begins reformatting diagnostics,
//! these snapshots are what change. See the [testing document](../../docs/implementation/testing.md).

pub mod fixtures;
pub mod harness;
pub mod options;
pub mod paths;
pub mod snapshot;

use crate::options::Options;
use crate::snapshot::Outcome;

/// Run the UI suite. `args` are the harness arguments (everything cargo passes after
/// `--`): `--bless` to regenerate snapshots, `--print` to print each fixture's raw
/// output instead of comparing, and any bare words as path substring filters.
///
/// Exits the process: `0` if every fixture matched (or was blessed/printed), `1` on a
/// snapshot mismatch, `2` if no fixture matched the filters.
pub fn run(args: Vec<String>) {
    let options = Options::parse(args);

    harness::build_binaries();
    let harness_crate = harness::ensure_harness_crate();
    let fixtures_dir = paths::fixtures_dir();

    let fixtures = fixtures::collect(&fixtures_dir, &options);
    if fixtures.is_empty() {
        eprintln!("no fixtures matched");
        std::process::exit(2);
    }

    let mut failed = 0usize;
    for fixture in &fixtures {
        let name = fixture
            .strip_prefix(&fixtures_dir)
            .unwrap_or(fixture)
            .display();
        let actual = harness::run_fixture(&harness_crate, fixture);

        if options.print {
            println!("===== {name} =====");
            print!("{actual}");
            println!("===== end {name} =====");
            continue;
        }

        match snapshot::review(fixture, &actual, options.bless) {
            Outcome::Ok => println!("ok       {name}"),
            Outcome::Blessed => println!("blessed  {name}"),
            Outcome::Mismatch => {
                println!("MISMATCH {name}");
                failed += 1;
            }
        }
    }

    if failed > 0 {
        eprintln!(
            "\n{failed} snapshot mismatch(es); re-run with `--bless` to update after an intended change"
        );
        std::process::exit(1);
    }
}
