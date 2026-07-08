# Testing

`cargo-cgp` is tested at two levels today — fast unit tests over the argument-handling logic, and a
set of runnable example fixtures exercised end to end by hand — with an automated end-to-end harness
still to come.

## The layers of testing

Testing the tool splits along the seam between its two kinds of logic: the ordinary Rust that
decides *how to invoke the compiler*, and the emergent behavior of *actually invoking it*. The first
is pure and cheap to unit-test; the second only shows up when a real `cargo` drives a real compiler
through the driver, so it is exercised against example crates instead. Unit tests guard the former,
the example fixtures under [`tests/`](../../tests/README.md) feed the latter, and a short manual
checklist confirms the whole pipeline holds together. Each layer is described below.

## Unit tests

The argument-handling logic on both sides of the tool is covered by unit tests, because it is pure
input-to-output transformation with corner cases that are easy to get wrong and easy to pin. These
are the standing automated tests: they run under `cargo test` (and `cargo nextest run`) with no
toolchain ceremony, and new argument-handling behavior should arrive with a test here rather than a
manual check.

Two modules carry them. The front-end's [`args.rs`](../../crates/cargo-cgp/src/args.rs) tests that
`strip_subcommand` drops the cargo-inserted `cgp` token for the `cargo cgp check` form, leaves the
direct `cargo-cgp check` form alone, keeps a later token that merely happens to equal `cgp`, and
yields nothing when only the program name is present. The driver's
[`args.rs`](../../crates/cargo-cgp-driver/src/args.rs) tests that `rustc_args` strips the injected
`rustc` path in wrapper mode, injects `--sysroot` when it is absent, keeps an existing sysroot
untouched, and leaves a non-wrapper invocation alone. Between them they pin the two transforms most
likely to break silently — the ones that decide which arguments reach `cargo` and which reach
`rustc`.

## Example fixtures

Standalone CGP source files under [`tests/examples/`](../../tests/examples) are the material for
exercising the tool on real compilations. Each file is one self-contained scenario — a
correctly-wired program, or a specific CGP mistake and the compiler error it produces — in the
spirit of the [`cgp-compile-fail-tests`](../../../cgp/crates/tests/cgp-compile-fail-tests) fixtures,
but arranged so any one of them can be fed through `cargo-cgp check` to watch the diagnostics.

The fixtures share one manifest, so adding a scenario costs nothing but a file. Cargo auto-discovers
`examples/*.rs` as example targets, `cgp` is declared once as a path dependency, and the package is
deliberately excluded from the workspace so its intentionally-failing examples never break the root
build — the mechanics are covered in [Executable structure](executable-structure.md) and the
package's own [README](../../tests/README.md). The relevant point for testing is that a fixture is
checked in isolation with `cargo check --example <name>`, and run through the tool with the helper
script below. The two starter fixtures are a clean baseline (`greet_ok`) and a hidden-cause wiring
error (`hidden_missing_dependency`); the latter is the kind of diagnostic `cargo-cgp` exists to make
readable, so it doubles as the first target for the tool's future work.

## Running a fixture through the tool

The helper [`scripts/run-check.sh`](../../scripts/run-check.sh) is the standing way to run the tool
on a fixture. It builds both binaries — the front-end and the driver, which must sit together — and
then runs `cargo cgp check --example <name>` inside the `tests/` package:

```sh
scripts/run-check.sh greet_ok                   # checks clean, exits 0
scripts/run-check.sh hidden_missing_dependency  # surfaces the CGP error, exits non-zero
```

It accepts either an example name or a path to the file, lists the available examples when given no
argument, and forwards any trailing arguments to `cargo check` (so `-v` works). Because it runs
`cargo` inside the repository, rustup selects the pinned toolchain from `rust-toolchain.toml`;
override it for a one-off with `RUSTUP_TOOLCHAIN=<toolchain>` in the environment.

## End-to-end verification

A green `cargo build` and green unit tests do not prove the tool works, because the part that matters
— the driver actually standing in as the compiler — is exactly the part no unit test covers. Until
an automated harness exists, verify it by hand against the fixtures (or any throwaway crate) and
confirm three things:

- **Valid code checks clean.** `scripts/run-check.sh greet_ok` finishes with exit `0`.
- **Broken code surfaces the compiler's errors** and exits non-zero.
  `scripts/run-check.sh hidden_missing_dependency` prints the `E0599`/`E0277` diagnostic and exits
  `101`.
- **The driver is genuinely in the loop.** Run a fixture with a trailing `-v`, forcing a recompile if
  the target is cached (`touch` the example first), and confirm the `cargo-cgp-driver` executable
  appears in cargo's verbose rustc invocation. This also proves forwarded arguments reach
  `cargo check`.

One toolchain constraint underlies all of this: the driver *embeds* the compiler of the nightly it
was built with, so the crate being checked must be compiled with that same nightly, or the driver
loads a mismatched `librustc_driver` and fails. The script sidesteps the problem by building and
running under the one toolchain the repository pins; a manual invocation must select it too. See
[Executable structure](executable-structure.md#accessing-the-rust-compiler-api) for why.

## Comparison with Clippy

Clippy tests the same integration far more heavily than `cargo-cgp` does today, and its setup is the
model to grow toward. Its core is a **UI-test harness**: `tests/compile-test.rs` drives
[`ui_test`](https://github.com/oli-obk/ui_test) over the `.rs` fixtures in `tests/ui/`, compiling
each and diffing the compiler's output against a committed `.stderr` snapshot, which is regenerated
with `cargo bless` (via `cargo dev`) when the expected output changes. Clippy also runs a **dogfood**
test (`tests/dogfood.rs`) that lints its own source, plus config, formatting, and integration tests.
The parent `cgp` project tests its own post-codegen failures the same snapshot way, with
[`trybuild`](https://docs.rs/trybuild) compiling each fixture and pinning its `.stderr`.

`cargo-cgp` has none of that automation yet, and the reason is specific: an output-snapshot harness
pins *the tool's own output*, and `cargo-cgp`'s driver is still a passthrough that emits exactly what
`rustc` emits, so there is nothing distinctive to snapshot. Its fixtures are therefore run by hand,
not asserted against blessed output. This is the main testing gap, and it closes naturally once the
driver begins reformatting diagnostics: at that point the `tests/examples` fixtures — already
arranged one scenario per file — become the inputs to a `ui_test`- or `trybuild`-style harness that
blesses the tool's reformatted output, and an automated end-to-end test can replace the manual
checklist above. A dogfood-style test that runs `cargo-cgp` on this repository's own crates is a
second natural addition.

## Further reading

These are the harnesses the future automated suite will most likely build on; both compile each
fixture as its own crate and diff a committed snapshot, which is the shape this project's fixtures
are already arranged for.

- [`trybuild`](https://docs.rs/trybuild) — the compile-fail snapshot harness the parent `cgp`
  project uses for its post-codegen failures.
- [`ui_test`](https://github.com/oli-obk/ui_test) — the UI-test harness Clippy uses over `tests/ui`.

## Tests

The automated tests are the argument-handling unit tests; the example fixtures are manual inputs, not
assertions, and there is no automated end-to-end or output-snapshot test yet (see the gap above).

- [`crates/cargo-cgp/src/args.rs`](../../crates/cargo-cgp/src/args.rs) — `strip_subcommand` across the
  `cargo cgp check` form, the direct form, a later matching token, and the program-name-only case.
- [`crates/cargo-cgp-driver/src/args.rs`](../../crates/cargo-cgp-driver/src/args.rs) — `rustc_args`
  across wrapper-mode stripping, sysroot injection, an existing sysroot, and a non-wrapper
  invocation.

## Source

- [`crates/cargo-cgp/src/args.rs`](../../crates/cargo-cgp/src/args.rs),
  [`crates/cargo-cgp-driver/src/args.rs`](../../crates/cargo-cgp-driver/src/args.rs) — the modules the
  unit tests cover.
- [`tests/`](../../tests/README.md) — the example-fixture package (shared manifest, workspace
  exclusion, cgp path dependency).
- [`tests/examples/`](../../tests/examples) — the fixtures themselves, one scenario per file.
- [`scripts/run-check.sh`](../../scripts/run-check.sh) — builds the binaries and runs the tool on a
  chosen fixture.
- [`Cargo.toml`](../../Cargo.toml) — the workspace `exclude` that keeps the fixtures out of the root
  build.
