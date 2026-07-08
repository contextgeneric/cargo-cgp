# Testing

`cargo-cgp` is tested at two levels: fast unit tests over the argument-handling logic, and a UI
snapshot suite that compiles example CGP programs through the real tool and pins its output against
committed `.stderr` files.

## The layers of testing

Testing the tool splits along the seam between its two kinds of logic: the ordinary Rust that decides
*how to invoke the compiler*, and the emergent behavior of *actually invoking it*. The first is pure
and cheap to unit-test; the second only appears when a real `cargo` drives a real compiler through
the driver, so it is exercised by running the whole tool against example crates and snapshotting what
it prints. Unit tests guard the former; the UI snapshot suite guards the latter. Each is described
below.

## Unit tests

The argument-handling logic on both sides of the tool is covered by unit tests, because it is pure
input-to-output transformation with corner cases that are easy to get wrong and easy to pin. These
run under `cargo test` with no toolchain ceremony, and new argument-handling behavior should arrive
with a test here rather than a manual check.

Two modules carry them. The front-end's [`args.rs`](../../crates/cargo-cgp/src/args.rs) tests that
`strip_subcommand` drops the cargo-inserted `cgp` token for the `cargo cgp check` form, leaves the
direct form alone, keeps a later token that merely equals `cgp`, and yields nothing when only the
program name is present. The driver's [`args.rs`](../../crates/cargo-cgp-driver/src/args.rs) tests
that `rustc_args` strips the injected `rustc` path in wrapper mode, injects `--sysroot` when absent,
keeps an existing sysroot, and leaves a non-wrapper invocation alone. Between them they pin the two
transforms that decide which arguments reach `cargo` and which reach `rustc`.

## The UI snapshot suite

The UI suite is modeled on Clippy's: a tree of `.rs` fixtures, each paired with a committed `.stderr`
snapshot of the expected output, checked by compiling the fixture and diffing. The fixtures live
under [`tests/ui/`](../../tests/ui), grouped into subdirectories by the kind of scenario — `ok/` for
correctly-wired programs, `hidden/` for the hidden-cause error class, and further class directories
mirroring the [CGP error catalog](../../../cgp/docs/errors/README.md) as they are added. Each fixture
`<name>.rs` has a sibling `<name>.stderr`; a fixture that compiles cleanly has an empty snapshot.

What makes this suite worth having even while the tool's output equals `rustc`'s is *what* it
snapshots: the output of `cargo-cgp` itself, not of plain `rustc`. Each fixture is compiled by
running the real `cargo-cgp check` end to end — front-end, driver, and all — so the snapshot is
whatever the tool emits. Today the driver is a passthrough, so a snapshot is the normalized `rustc`
diagnostic; when the driver begins reformatting CGP errors, these snapshots are exactly what will
change, and the diff is the signal that the reformatting did what was intended. The suite exists now
so that change is caught the moment it lands.

### How a fixture is compiled

The fixtures are loose `.rs` files, not cargo targets, so something has to turn one into a crate the
tool can check. The shared script library [`scripts/lib.sh`](../../scripts/lib.sh) does this: it
maintains a throwaway crate (under the git-ignored `target/ui-harness/`) that depends on `cgp` by
path, copies the chosen fixture in as its `src/main.rs`, and runs `cargo-cgp check` there. Reusing
one crate keeps `cgp` compiled and cached across fixtures; naming it `ui` keeps cargo's
`could not compile \`ui\`` line stable; and an empty `[workspace]` table in its manifest stops cargo
from folding it into the `cargo-cgp` workspace above it in `target/`.

The tool's raw output carries cargo's own progress lines — `Checking`, `Compiling`, `Finished`, and
the like — which name volatile paths, versions, and timings. `normalize_output` in the same library
strips those status lines, leaving the compiler diagnostics and the final `could not compile`
summary. That normalized text is what a snapshot holds, so a snapshot changes only when the
*diagnostics* change, not when a path or a build time does.

### Running and blessing

Two scripts drive the suite, both building on `lib.sh` so the tool is exercised the same way in each.
[`scripts/ui-test.sh`](../../scripts/ui-test.sh) is the suite itself:

```sh
scripts/ui-test.sh            # compile every fixture, diff against its .stderr
scripts/ui-test.sh hidden     # only fixtures whose path contains "hidden"
scripts/ui-test.sh --bless    # rewrite the .stderr snapshots from current output
```

A run compiles each fixture, normalizes, and compares; a mismatch prints a unified diff and the suite
exits non-zero. After an *intended* change to what the tool emits, `--bless` regenerates the
snapshots — the analogue of Clippy's `cargo bless` — and the diff is reviewed before committing.
[`scripts/run-check.sh`](../../scripts/run-check.sh) is the interactive counterpart: it runs one
fixture through the same path but prints the tool's raw output, unmodified and uncompared, for
reading exactly what the tool produced.

### Toolchain and determinism

A snapshot is only reproducible against the toolchain it was blessed with, because it contains the
compiler's diagnostic text. The scripts build and run under the toolchain the repository pins in
[`rust-toolchain.toml`](../../rust-toolchain.toml) (overridable with `RUSTUP_TOOLCHAIN`), and
snapshots must be blessed under that same toolchain. A deliberate toolchain bump can therefore change
the diagnostic wording and require a re-bless, exactly as it does for Clippy — a `.stderr` diff after
a toolchain change is expected, not a regression.

The suite also serves as the end-to-end check that the driver genuinely stands in as the compiler: a
passing `hidden/` snapshot could only be produced by compiling the fixture through the tool. To see
the driver in the invocation directly, run a fixture with `run-check.sh` and read the output, or pass
`-v` through to cargo.

## Comparison with Clippy

The suite deliberately mirrors Clippy's UI-test *workflow* — a `tests/ui/` tree of `.rs` fixtures,
each with a committed `.stderr`, regenerated with a bless step — so the mental model transfers
directly. The parent `cgp` project pins its post-codegen failures the same snapshot way, with
[`trybuild`](https://docs.rs/trybuild). Where `cargo-cgp` diverges is in the harness and what it
drives, and each divergence has a reason.

Clippy's harness is a `cargo test` (`tests/compile-test.rs`) built on
[`ui_test`](https://github.com/oli-obk/ui_test), which invokes `rustc` directly on each fixture and
diffs the output. `cargo-cgp`'s harness is a shell script that runs the whole `cargo-cgp` binary,
going through `cargo` rather than calling `rustc` directly. The reason is that the thing under test
*is* a cargo subcommand and rustc wrapper: the behavior worth snapshotting only exists when the tool
drives a real `cargo`, so the harness drives the tool, not the compiler. Going through cargo brings
progress-line noise that `ui_test` never sees, which is why the harness normalizes it away.

Two gaps remain against Clippy, both deliberate for now. There is no dogfood test that runs
`cargo-cgp` on this repository's own crates; it becomes worthwhile once the tool does more than pass
through. And the suite is script-driven rather than a `cargo test` target, because bootstrapping the
driver binary and the pinned toolchain inside a `cargo test` is more machinery than a
build-and-run script; if the suite grows, promoting it to a `cargo test` (or adopting `ui_test`
pointed at the driver) is the natural next step.

## Further reading

These are the snapshot harnesses the suite's design draws on; both compile each fixture and diff a
committed snapshot with a bless step, the workflow reproduced here.

- [`ui_test`](https://github.com/oli-obk/ui_test) — the UI-test harness Clippy uses over `tests/ui`.
- [`trybuild`](https://docs.rs/trybuild) — the compile-fail snapshot harness the parent `cgp` project
  uses for its post-codegen failures.

## Tests

The automated tests are the argument-handling unit tests and the UI snapshot suite. There is no
dogfood or `cargo test`-integrated end-to-end test yet (see the gaps above).

- [`crates/cargo-cgp/src/args.rs`](../../crates/cargo-cgp/src/args.rs) — `strip_subcommand` across the
  `cargo cgp check` form, the direct form, a later matching token, and the program-name-only case.
- [`crates/cargo-cgp-driver/src/args.rs`](../../crates/cargo-cgp-driver/src/args.rs) — `rustc_args`
  across wrapper-mode stripping, sysroot injection, an existing sysroot, and a non-wrapper
  invocation.
- [`tests/ui/`](../../tests/ui) — the UI snapshot fixtures, each `<name>.rs` paired with a blessed
  `<name>.stderr`, run by `scripts/ui-test.sh`.

## Source

- [`crates/cargo-cgp/src/args.rs`](../../crates/cargo-cgp/src/args.rs),
  [`crates/cargo-cgp-driver/src/args.rs`](../../crates/cargo-cgp-driver/src/args.rs) — the modules the
  unit tests cover.
- [`tests/ui/`](../../tests/ui) — the fixture tree, one scenario per `.rs` file with its `.stderr`
  snapshot, grouped into class subdirectories.
- [`scripts/lib.sh`](../../scripts/lib.sh) — shared helpers: builds the binaries, maintains the
  throwaway crate, runs a fixture through `cargo-cgp`, and normalizes the output.
- [`scripts/ui-test.sh`](../../scripts/ui-test.sh) — the snapshot suite (compare, filter, `--bless`).
- [`scripts/run-check.sh`](../../scripts/run-check.sh) — runs one fixture and prints raw output.
