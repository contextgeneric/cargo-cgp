# Testing

`cargo-cgp` is tested at two levels: fast unit tests over the argument-handling logic, and a UI
snapshot suite — a custom Rust test harness, in the style of Clippy's — that compiles example CGP
programs through the real tool and pins its output against committed `.stderr` files.

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
keeps an existing sysroot, and leaves a non-wrapper invocation alone. The harness crate additionally
unit-tests its own option parsing.

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
whatever the tool emits. Today the driver is a passthrough, so a snapshot is the `rustc` diagnostic;
when the driver begins reformatting CGP errors, these snapshots are exactly what will change, and the
diff is the signal that the reformatting did what was intended. The suite exists now so that change
is caught the moment it lands.

### The harness is a custom Rust test binary

Following Clippy, the suite is a **custom test harness written in Rust**, not a shell script and not
libtest. It lives in its own crate, [`crates/cargo-cgp-ui-tests`](../../crates/cargo-cgp-ui-tests),
whose `ui` test target sets `harness = false` and provides its own `fn main`
([`tests/ui.rs`](../../crates/cargo-cgp-ui-tests/tests/ui.rs)) — the same shape as Clippy's
`tests/compile-test.rs`. The `fn main` is thin; the logic is in the crate's library so it stays small
and testable, split into focused modules: `options` (argument parsing), `paths` (locating the
workspace, fixtures, cgp checkout, and built binaries), `fixtures` (discovery), `harness` (building
the binaries and compiling a fixture), and `snapshot` (compare, bless, diff).

The harness crate is a full workspace member, so `cargo test` runs the whole suite alongside the
unit tests. The crate itself depends on nothing but `std` — it shells out to `cargo` at run time —
but running the suite therefore builds the driver and expects a sibling `cgp` checkout at `../cgp`,
so a plain `cargo test` now needs both.

### How a fixture is compiled

A fixture is a loose `.rs` file, so the harness turns it into a crate the tool can check: it
maintains a throwaway crate (under the git-ignored `target/ui-harness/`) that depends on `cgp` by
path, copies the fixture in as its `src/main.rs`, and runs `cargo-cgp check -q --color never` there.
Reusing one crate keeps `cgp` compiled and cached across fixtures; naming it `ui` keeps cargo's
output stable; and an empty `[workspace]` table in its manifest stops cargo from folding it into the
`cargo-cgp` workspace above it in `target/`.

The `-q` is what keeps snapshots clean without post-processing: it suppresses cargo's own progress
lines (`Checking`, `Compiling`, `Finished`), leaving the compiler diagnostics and the final
`could not compile` summary — all deterministic, so the captured output can be snapshotted verbatim.
The harness finds the built `cargo-cgp` beside its own test binary in `target/debug`, having first
built both binaries with `cargo build` (the front-end locates the driver as its sibling).

### Running and blessing

The suite is part of `cargo test`; to run only it, select the crate:

```sh
cargo test                                  # everything (unit tests + the UI suite)
cargo test -p cargo-cgp-ui-tests            # only the suite
```

To pass an argument to the harness, target the `ui` test explicitly with `--test ui`, so the flag is
not also handed to the crate's libtest unit tests. The harness accepts a path substring to filter
fixtures, `--bless` to rewrite the snapshots, and `--print` to show a fixture's raw output instead of
comparing:

```sh
cargo test -p cargo-cgp-ui-tests --test ui -- hidden      # only fixtures whose path contains "hidden"
cargo test -p cargo-cgp-ui-tests --test ui -- --bless     # rewrite the .stderr snapshots
cargo test -q -p cargo-cgp-ui-tests --test ui -- --print unsatisfied_dependency  # print raw output
```

After an *intended* change to what the tool emits, `--bless` regenerates the snapshots — the analogue
of Clippy's `cargo bless` — and the diff is reviewed before committing.

### Toolchain and determinism

A snapshot is only reproducible against the toolchain it was blessed with, because it contains the
compiler's diagnostic text. The harness builds and runs under the toolchain the repository pins in
[`rust-toolchain.toml`](../../rust-toolchain.toml) (overridable with `RUSTUP_TOOLCHAIN`), and
snapshots must be blessed under that same toolchain. A deliberate toolchain bump can therefore change
the diagnostic wording and require a re-bless, exactly as it does for Clippy — a `.stderr` diff after
a toolchain change is expected, not a regression. A passing `hidden/` snapshot is also the standing
proof that the driver genuinely stands in as the compiler, since it could only be produced by
compiling the fixture through the tool.

## Comparison with Clippy

The suite now matches Clippy's *approach* closely: a custom Rust test harness with `harness = false`
and its own `fn main`, driving a tree of `tests/ui/*.rs` fixtures against committed `.stderr`
snapshots with a bless step. The mental model transfers directly, and
[`external/rust-clippy/tests/compile-test.rs`](../../../external/rust-clippy/tests/compile-test.rs)
is the reference to read alongside this crate. Two deliberate differences remain.

First, the harness is **hand-rolled rather than built on the [`ui_test`](https://github.com/oli-obk/ui_test)
library** Clippy uses. `ui_test` invokes a compiler directly on each fixture and, via its
`DependencyBuilder`, computes the `--extern`/`-L` flags needed to make a dependency like `cgp`
available. The hand-rolled harness sidesteps that machinery — and the version-coupling of a large
test dependency — by driving the whole `cargo-cgp` tool through `cargo`, which resolves `cgp` for us.
The cost is that compilation goes through cargo (its progress noise, quieted with `-q`) instead of
straight to the compiler.

Second, and following from that, the harness **drives the whole tool, where Clippy's `ui_test`
drives `clippy-driver` directly**. Driving the front-end is a stronger end-to-end test — it exercises
`cargo-cgp` as a user invokes it — and it is what makes the cargo-resolves-`cgp` shortcut possible.
If the suite grows enough to want per-diagnostic control (inline `//~` annotations, rustfix, and the
like), adopting `ui_test` pointed at `cargo-cgp-driver` is the natural next step.

One gap against Clippy is unrelated to the harness: there is no dogfood test that runs `cargo-cgp` on
this repository's own crates. It becomes worthwhile once the tool does more than pass through.

## Further reading

These are the snapshot harnesses this suite's design draws on; both compile each fixture and diff a
committed snapshot with a bless step, the workflow reproduced here.

- [`ui_test`](https://github.com/oli-obk/ui_test) — the UI-test library Clippy's harness is built on.
- [`trybuild`](https://docs.rs/trybuild) — the compile-fail snapshot harness the parent `cgp` project
  uses for its post-codegen failures.

## Tests

The automated tests are the argument-handling unit tests, the harness crate's option-parsing unit
tests, and the UI snapshot suite. There is no dogfood test yet (see above).

- [`crates/cargo-cgp/src/args.rs`](../../crates/cargo-cgp/src/args.rs) — `strip_subcommand` across the
  `cargo cgp check` form, the direct form, a later matching token, and the program-name-only case.
- [`crates/cargo-cgp-driver/src/args.rs`](../../crates/cargo-cgp-driver/src/args.rs) — `rustc_args`
  across wrapper-mode stripping, sysroot injection, an existing sysroot, and a non-wrapper
  invocation.
- [`crates/cargo-cgp-ui-tests/src/options.rs`](../../crates/cargo-cgp-ui-tests/src/options.rs) —
  harness option and filter parsing.
- [`tests/ui/`](../../tests/ui) — the UI snapshot fixtures, each `<name>.rs` paired with a blessed
  `<name>.stderr`, run by the harness.

## Source

- [`crates/cargo-cgp-ui-tests/`](../../crates/cargo-cgp-ui-tests) — the custom UI-test harness:
  `tests/ui.rs` (the `harness = false` entrypoint) and the `src/` modules (`options`, `paths`,
  `fixtures`, `harness`, `snapshot`).
- [`tests/ui/`](../../tests/ui) — the fixture tree, one scenario per `.rs` file with its `.stderr`
  snapshot, grouped into class subdirectories.
- [`crates/cargo-cgp/src/args.rs`](../../crates/cargo-cgp/src/args.rs),
  [`crates/cargo-cgp-driver/src/args.rs`](../../crates/cargo-cgp-driver/src/args.rs) — the modules the
  argument unit tests cover.
