# cargo-cgp test examples

This package hosts small, standalone CGP source files to run `cargo-cgp` against while developing the
tool. Each file under [`examples/`](examples) is one self-contained scenario — a correctly-wired
program, or a specific CGP mistake and the compiler error it produces — in the spirit of the
[`cgp-compile-fail-tests`](../../cgp/crates/tests/cgp-compile-fail-tests) fixtures, but arranged so
we can feed any one of them through `cargo-cgp check` and watch the diagnostics.

This README covers the package itself. For how it fits into the project's overall testing approach —
the unit tests, the end-to-end verification, and the comparison with Clippy's test harness — see the
[Testing](../docs/implementation/testing.md) implementation document.

## How the shared configuration works

Every example shares this package's single [`Cargo.toml`](Cargo.toml); there is no per-file manifest.
Cargo auto-discovers `examples/*.rs` as example targets, so adding a scenario is just adding a file —
no manifest edit, no entry to register. `cgp` is declared once as a dependency (a path into the
sibling `cgp` checkout at `../../cgp`), so every example may `use cgp::prelude::*;`.

This package is deliberately **excluded from the workspace** (see [`../Cargo.toml`](../Cargo.toml)).
Many examples are CGP mistakes that fail to compile on purpose, and a workspace member would break
`cargo build` and `cargo clippy` at the repository root. Because it is excluded, the package is only
ever compiled on demand — through `cargo-cgp`, which is exactly what we want to exercise.

## Running an example

Use the helper script from the repository root, passing an example name (or the path to its file):

```sh
scripts/run-check.sh greet_ok
scripts/run-check.sh hidden_missing_dependency
```

The script builds `cargo-cgp` and its driver, then runs `cargo cgp check --example <name>` in this
directory. Run it with no argument to list the available examples. You can also invoke the tool by
hand — `cargo check --example <name>` compiles just that file, and prefixing it with a built
`cargo-cgp` runs it through the driver.

## Adding an example

Drop a new `<name>.rs` file into [`examples/`](examples). Give it a `fn main`, since an example is a
binary target, and open it with a `//!` comment stating what the scenario demonstrates and — for an
error case — which [CGP error class](../../cgp/docs/errors/README.md) it reproduces and where the
root cause sits. Keep one scenario per file so it can be checked in isolation.

The current examples are:

- [`greet_ok.rs`](examples/greet_ok.rs) — a correctly-wired program that checks clean; the baseline.
- [`hidden_missing_dependency.rs`](examples/hidden_missing_dependency.rs) — an unmet impl-side
  dependency reached by a direct method call, so the compiler *hides* the root cause. This is the
  kind of error `cargo-cgp` aims to make readable.
