# cargo-cgp UI tests

This directory holds the UI test fixtures for `cargo-cgp`: small, standalone CGP source files, each
compiled through the tool and compared against a committed snapshot of its output. It is modeled on
Clippy's `tests/ui/` and, like the parent project's
[`cgp-compile-fail-tests`](../../cgp/crates/tests/cgp-compile-fail-tests), pairs each `.rs` fixture
with a blessed expected-output file.

For how the fixtures fit into the project's overall testing approach — the unit tests, the harness
mechanics, the toolchain caveat, and the comparison with Clippy — see the
[Testing](../docs/implementation/testing.md) implementation document. This README is the quick
operational guide.

## Layout

Fixtures live under [`ui/`](ui), grouped into subdirectories by the kind of scenario. Each fixture
`<name>.rs` has a sibling `<name>.stderr` holding the expected, normalized output; a fixture that
compiles cleanly has an empty snapshot. The subdirectories mirror the
[CGP error catalog](../../cgp/docs/errors/README.md) classes:

- [`ui/ok/`](ui/ok) — correctly-wired programs that check cleanly (empty snapshots); the baseline.
- [`ui/hidden/`](ui/hidden) — errors whose root cause the compiler hides; the class `cargo-cgp` most
  wants to make readable, so these are the snapshots to watch change.

Add a class directory (`checks/`, `wiring/`, …) as fixtures for it are written.

## Running

Use the scripts from the repository root. The snapshot suite compiles every fixture through
`cargo-cgp` and diffs the output against its `.stderr`:

```sh
scripts/ui-test.sh            # run the whole suite
scripts/ui-test.sh hidden     # only fixtures whose path contains "hidden"
scripts/ui-test.sh --bless    # regenerate the .stderr snapshots (review the diff!)
```

To read the tool's raw output on one fixture, without normalizing or comparing, use the interactive
runner:

```sh
scripts/run-check.sh unsatisfied_dependency
```

The snapshots capture `cargo-cgp`'s own output, so today — while the driver passes diagnostics
through unchanged — a snapshot is the normalized `rustc` message. When the driver starts reformatting
CGP errors, these snapshots are what change; `--bless` is how you record the new output after an
intended change.

## Adding a fixture

Drop a new `<name>.rs` into the matching class directory under `ui/`. Give it a `fn main`, since the
harness compiles it as a binary, and open it with a `//!` comment stating what the scenario
demonstrates and — for an error case — which [CGP error class](../../cgp/docs/errors/README.md) it
reproduces. `cgp` is available to every fixture (the harness compiles each in a throwaway crate that
depends on it), so a fixture may `use cgp::prelude::*;` with no setup. Then run
`scripts/ui-test.sh --bless` to create the snapshot and review it before committing.
