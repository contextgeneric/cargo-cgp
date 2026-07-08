# cargo-cgp UI tests

This directory holds the UI test fixtures for `cargo-cgp`: small, standalone CGP source files, each
compiled through the tool and compared against a committed snapshot of its output. It is modeled on
Clippy's `tests/ui/` and, like the parent project's
[`cgp-compile-fail-tests`](../../cgp/crates/tests/cgp-compile-fail-tests), pairs each `.rs` fixture
with a blessed expected-output file.

For how the fixtures fit into the project's overall testing approach — the argument tests, the
harness mechanics, the toolchain caveat, and the comparison with Clippy — see the
[Testing](../docs/implementation/testing.md) implementation document. This README is the quick
operational guide.

## Layout

Fixtures live under [`ui/`](ui), grouped into subdirectories by the *quality of the output* the tool
produces for them. Each fixture `<name>.rs` has a sibling `<name>.stderr` holding the expected
output; a fixture that compiles cleanly has an empty snapshot. The directories mirror the
pending-issue categories in [docs/issues/](../docs/issues/README.md), so a fixture's directory names
the kind of problem it exposes:

- [`ui/hidden-root-cause/`](ui/hidden-root-cause) — errors whose root cause cannot be recovered from
  the output at all, no matter how it is reformatted (a
  [hidden root cause](../docs/issues/hidden-root-cause.md)); the highest-value class to fix, so these
  snapshots are the ones to watch change.
- [`ui/usability/`](ui/usability) — errors that carry the root cause but bury it in volume, encoding,
  or misleading framing (a [usability issue](../docs/issues/usability.md)); the cause is present, so
  the work is re-presentation.
- [`ui/ok/`](ui/ok) — output that needs no further work: correctly-wired programs that check cleanly
  today, and, as issues are fixed, the reformatted errors that graduate here. This is the passing
  baseline.

A fixture's placement follows the sufficiency test in [docs/issues/](../docs/issues/README.md): if no
downstream tool could recover the cause from the output, it is `hidden-root-cause/`; if a careful
reader could, it is `usability/`; if the output is already good, it is `ok/`.

## Running

The suite is a custom Rust test harness in the [`cargo-cgp-ui-tests`](../crates/cargo-cgp-ui-tests)
crate (modeled on Clippy's `compile-test`), which compiles every fixture through `cargo-cgp` and
diffs the output against its `.stderr`. Run it with `cargo test`:

```sh
cargo test -p cargo-cgp-ui-tests            # run the whole suite
```

To filter, bless, or print, pass an argument to the harness — target `--test ui` so the flag is not
also handed to the crate's other tests:

```sh
cargo test -p cargo-cgp-ui-tests --test ui -- usability  # only fixtures whose path contains "usability"
cargo test -p cargo-cgp-ui-tests --test ui -- --bless   # regenerate the .stderr snapshots
cargo test -q -p cargo-cgp-ui-tests --test ui -- --print unsatisfied_dependency  # raw output
```

The snapshots capture `cargo-cgp`'s own output. Because the driver runs the workspace crate through
the next-gen trait solver, that output already differs from a plain `cargo check` — the
`usability/unsatisfied_dependency` snapshot shows the un-hidden `HasField` root cause that a plain
`cargo check` would suppress. As the driver grows to reformat CGP errors, these snapshots are what
change; `--bless` is how you record the new output after an intended change. Snapshots are blessed
under the toolchain the repository pins, so a toolchain bump can require a re-bless.

## Adding a fixture

Drop a new `<name>.rs` into the category directory under `ui/` its output belongs to (`ok/`,
`usability/`, or `hidden-root-cause/`, per the sufficiency test above). Give it a `fn main`, since the
harness compiles it as a binary, and open it with a `//!` comment stating what the scenario
demonstrates, which [CGP error class](../../cgp/docs/errors/README.md) it reproduces, and — for a
problem case — the [issue](../docs/issues/README.md) it exposes. `cgp` is available to every fixture (the harness compiles each in a throwaway crate that
depends on it), so a fixture may `use cgp::prelude::*;` with no setup. Then run
`cargo test -p cargo-cgp-ui-tests --test ui -- --bless` to create the snapshot and review it before
committing.
