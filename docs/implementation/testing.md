# Testing

`cargo-cgp` is tested at two levels: fast tests over the argument-handling and diagnostic-processing
logic, and a UI snapshot suite — a custom Rust test harness, in the style of Clippy's — that compiles
example CGP programs through the real tool and pins both its rendered output (`.stderr`) and the
diagnostics it captured (`.output.json`). Every test lives in its crate's `tests/` directory; per
[../../AGENTS.md](../../AGENTS.md) the project keeps no inline `#[cfg(test)]` modules, so all tests
are integration tests against a crate's public API.

## The layers of testing

Testing the tool splits along the seam between its two kinds of logic: the ordinary Rust that decides
*how to invoke the compiler*, and the emergent behavior of *actually invoking it*. The first is pure
and cheap to test; the second only appears when a real `cargo` drives a real compiler through the
driver, so it is exercised by running the whole tool against example crates and snapshotting what it
prints. The argument tests guard the former; the UI snapshot suite guards the latter. Each is
described below.

## Argument-handling tests

The argument-handling logic on both sides of the tool is tested directly, because it is pure
input-to-output transformation with corner cases that are easy to get wrong and easy to pin. Each
crate's tests live in its `tests/` directory and run under `cargo test` with no toolchain ceremony;
new argument-handling behavior should arrive with a test here rather than a manual check.

Two crates carry these. The front-end's [`tests/args.rs`](../../crates/cargo-cgp/tests/args.rs) checks
that `strip_subcommand` drops the cargo-inserted `cgp` token for the `cargo cgp check` form, leaves
the direct form alone, keeps a later token that merely equals `cgp`, and yields nothing when only the
program name is present. The driver's [`tests/args.rs`](../../crates/cargo-cgp-driver/tests/args.rs)
checks that `rustc_args` strips the injected `rustc` path in wrapper mode, injects `--sysroot` when
absent, keeps an existing sysroot, appends an injected flag when absent, and lets an explicit
`-Znext-solver` override it. (That test links the driver, so — like the driver binary — it carries
the `#![feature(rustc_private)]` gate.) The harness crate additionally tests its own option parsing
and output normalization, also under `tests/`.

## The UI snapshot suite

The UI suite is modeled on Clippy's: a tree of `.rs` fixtures, each paired with committed snapshots,
checked by compiling the fixture and diffing. The fixtures live under [`tests/ui/`](../../tests/ui),
grouped into subdirectories by the *quality of the output* the tool produces — `hidden-root-cause/`
for errors whose cause is unrecoverable from the output, `usability/` for errors that carry the cause
but bury it, and `ok/` for output that needs no further work — the same categories as the
pending-issue documents in [docs/issues/](../issues/README.md), each fixture exposing the issue its
directory names. Each fixture `<name>.rs` has two siblings: `<name>.stderr`, the tool's rendered
output, and `<name>.output.json`, the diagnostics it captured; a fixture that compiles cleanly has an
empty `.stderr` and an empty (`[]`) `.output.json`. The `usability/` fixtures are further sorted into
kind subdirectories (`checks/`, `wiring/`, `lowering/`, `unsatisfied-dependency/`) mirroring the
upstream catalog's sections; alongside the hand-curated examples they include a verbatim mirror of the
upstream CGP compile-fail suite (one fixture per reproducible error class), giving the tool a snapshot
of its own transformed output for the whole [error catalog](../../../cgp/docs/errors/README.md). The
[usability fixtures README](../../tests/ui/usability/README.md) records the class-by-class findings.

What the suite snapshots is *the output of `cargo-cgp` itself*, not of plain `rustc`. Each fixture is
compiled by running the real `cargo-cgp check` end to end — front-end, driver, and all — so the
snapshot is whatever the tool emits, already shaped by what the driver does. That difference is
visible today: because the driver enables the next-gen trait solver, the
`usability/unsatisfied-dependency/unsatisfied_dependency` snapshot shows the un-hidden `HasField` root
cause, not the default solver's dead-end. As the tool grows (reformatting diagnostics in its
processing stage), these snapshots are exactly what will change, and the diff is the signal that the
change did what was intended. The suite exists so that is caught the moment it lands.

### Three passes per fixture

Each fixture is verified by three passes that must all agree, so the tool's real output, the
diagnostics it captures, and its pure processing pipeline cannot drift apart. The passes are
implemented in [`passes`](../../crates/cargo-cgp-ui-tests/src/passes.rs):

- **The stderr pass** runs `cargo-cgp check` directly and compares the tool's rendered stderr to
  `<name>.stderr`. This is the end-to-end check that the whole binary produces the expected output.
- **The JSON pass** runs `cargo-cgp check --message-format=json`, through which the front-end forwards
  cargo's diagnostic stream unchanged; the harness extracts the diagnostics the tool feeds to
  `process_cgp_errors` and compares them to `<name>.output.json`. This pins the *input* to processing.
- **The process pass** parses `<name>.output.json`, runs it through `process_cgp_errors`, renders the
  result with the tool's own renderer, and compares to `<name>.stderr`. This is the pure unit pass —
  no compiler, no cargo — and it shares the `.stderr` target with the stderr pass because rendering
  the processed diagnostics must reproduce what the binary prints.

The stderr and process passes agree only because the cargo summary line (`could not compile … due to
N errors`) is normalized away: it comes from cargo's own stderr, not from a diagnostic, so it is not
present in the captured JSON and the process pass cannot reproduce it. Dropping it keeps `.stderr`
equal to the rendered processing output. The process pass reuses the tool's real capture
(`parse_cargo_output`) and render (`emit_rendered`) code, so the unit path cannot silently diverge
from the binary — which is why the harness now depends on the `cargo-cgp` and
`cargo-cgp-error-processing` libraries rather than only shelling out.

### The harness is a custom Rust test binary

Following Clippy, the suite is a **custom test harness written in Rust**, not a shell script and not
libtest. It lives in its own crate, [`crates/cargo-cgp-ui-tests`](../../crates/cargo-cgp-ui-tests),
whose `ui` test target sets `harness = false` and provides its own `fn main`
([`tests/ui.rs`](../../crates/cargo-cgp-ui-tests/tests/ui.rs)) — the same shape as Clippy's
`tests/compile-test.rs`. The `fn main` is thin; the logic is in the crate's library so it stays small
and testable, split into focused modules: `options` (argument parsing), `paths` (locating the
workspace, fixtures, cgp checkout, and built binaries), `fixtures` (discovery), `harness` (building
the binaries and compiling a fixture in a worker crate), `passes` (the three per-fixture passes),
`runner` (scheduling fixtures across the worker pool), `normalize` (rewriting volatile paths out of
the output), and `snapshot` (compare, bless, diff).

The harness crate is a full workspace member, so `cargo test` runs the whole suite alongside the
argument tests. It depends on the tool's own rustc-free libraries — `cargo-cgp` (for its capture and
render functions) and `cargo-cgp-error-processing` (for `process_cgp_errors`) — so the process pass
runs the same code the binary does; the two cargo-invoking passes still shell out to `cargo`. Running
the full suite therefore builds the driver and expects a sibling `cgp` checkout at `../cgp`, so a
plain `cargo test` needs both. The process pass alone (`--process-only`, below) needs neither.

### How a fixture is compiled

A fixture is a loose `.rs` file, so the harness turns it into a crate the tool can check: it
maintains a throwaway crate that depends on `cgp` by path, copies the fixture in as its `src/main.rs`,
and runs `cargo-cgp check -q --color never` there. Naming the crate `ui` keeps cargo's output stable,
and an empty `[workspace]` table in its manifest stops cargo from folding it into the `cargo-cgp`
workspace above it in `target/`. In a full run the stderr and JSON passes each run the tool once (the
second adds `--message-format=json`), so the fixture is compiled twice; re-copying it before each run
bumps its mtime, which forces cargo to recompile and re-emit diagnostics rather than serve a cached
build with none.

Fixtures are checked in parallel, and the shape of that parallelism is dictated by one cargo
constraint: a `cargo` build holds an exclusive lock on its target directory for the whole build, so
two checks can only overlap if they build in *separate* target directories. The harness therefore
runs a **pool of workers**, each owning its own throwaway crate under
`target/ui-harness/worker-<n>/` (so each gets its own target directory), and hands fixtures to
whichever worker is free (the `runner` module). Reusing a worker's crate across the fixtures it picks
up keeps `cgp` compiled and cached *within* that worker; the price of the isolation is that `cgp` is
built once per worker rather than once overall. The worker count defaults to the machine's
parallelism, capped at both 8 and the fixture count, and is overridable with `--jobs`/`-j` (below) —
the cap keeps a many-core machine from starting so many parallel `cgp` builds that the compilation and
disk cost outweighs the parallelism, and `--jobs` raises it again on a machine that can afford more. Each worker's crate directory carries the worker number, but that
absolute path is normalized to `$DIR`, so a snapshot never depends on which worker produced it. Each
fixture's result is printed the moment it finishes rather than held to the end, so a run streams live;
the order is therefore completion order, not fixture order, which is why every line names its fixture.

The `-q` removes most of the noise: it suppresses cargo's own progress lines (`Checking`,
`Compiling`, `Finished`). What remains and must be normalized away is machine-specific or
non-diagnostic: the absolute paths of the `cgp` checkout and the throwaway crate, the cargo
build-failure summary (`could not compile …`, which the process pass cannot reproduce from the
diagnostics alone), and — defensively — a note pointing at a hash-named temp file when a long type is
elided (the driver's `--verbose` suppresses that elision, so it does not arise for the current
fixtures). The `normalize` module handles the rendered `.stderr`: it rewrites the paths to
`$CGP`/`$DIR` and drops the summary and temp-file lines. A second normalizer, `normalize_json`,
handles `.output.json` with path rewriting only — the JSON is one value, so it must not drop lines.
Normalization applies to the compared/blessed output only; `--print` shows the raw output untouched.
The harness finds the built `cargo-cgp` beside its own test binary in `target/debug`, having first
built both binaries with `cargo build` (the front-end locates the driver as its sibling).

### Running and blessing

The suite is part of `cargo test`; to run only it, select the crate:

```sh
cargo test                                  # everything (the argument tests + the UI suite)
cargo test -p cargo-cgp-ui-tests            # only the suite
```

To pass an argument to the harness, target the `ui` test explicitly with `--test ui`, so the flag is
not also handed to the crate's other (libtest) tests. The harness accepts a path substring to filter
fixtures, `--bless` to rewrite the snapshots, `--print` to show a fixture's raw output instead of
comparing, `--process-only` to run just the fast process pass, and `--jobs N` (`-j N`) to set the
worker count:

```sh
cargo test -p cargo-cgp-ui-tests --test ui -- usability    # only fixtures whose path contains "usability"
cargo test -p cargo-cgp-ui-tests --test ui -- --bless      # rewrite the .stderr and .output.json snapshots
cargo test -p cargo-cgp-ui-tests --test ui -- -j 4                    # check at most 4 fixtures at once
cargo test -p cargo-cgp-ui-tests --test ui -- --process-only          # only the process_cgp_errors unit pass
cargo test -p cargo-cgp-ui-tests --test ui -- --process-only --bless  # re-bless .stderr from the process output
cargo test -q -p cargo-cgp-ui-tests --test ui -- --print unsatisfied_dependency  # print raw output
```

**`--jobs N`** (`-j N`) sets how many fixtures the harness checks at once; it defaults to the
machine's parallelism, capped at 8 and at the number of fixtures. Raise it past the default on a
machine that can afford more concurrent `cgp` builds — one per worker, since the workers cannot share
a target directory (above) — or set `-j 1` to run fully sequentially.

**`--process-only`** skips the two cargo-invoking passes and runs only the process pass over the
committed `.output.json`. It needs no compilation — the whole suite runs in well under a second — so
it is the loop to use while iterating on `process_cgp_errors`: change the processing code and re-run
to see the effect on every fixture at once. With `--bless` it rewrites `.stderr` from the new process
output (leaving `.output.json`, its input, untouched). A full run without `--process-only` is the
reconciling check: the stderr pass blesses `.stderr` from the real binary, the JSON pass blesses
`.output.json`, and the process pass verifies it still reproduces the blessed `.stderr`.

After an *intended* change to what the tool emits, `--bless` regenerates the snapshots — the analogue
of Clippy's `cargo bless` — and the diff is reviewed before committing.

### Toolchain and determinism

A snapshot is only reproducible against the toolchain it was blessed with, because it contains the
compiler's diagnostic text. The harness builds and runs under the toolchain the repository pins in
[`rust-toolchain.toml`](../../rust-toolchain.toml) (overridable with `RUSTUP_TOOLCHAIN`), and
snapshots must be blessed under that same toolchain. A deliberate toolchain bump can therefore change
the diagnostic wording and require a re-bless, exactly as it does for Clippy — a `.stderr` diff after
a toolchain change is expected, not a regression. A passing `usability/unsatisfied-dependency/unsatisfied_dependency`
snapshot is also the standing proof that the driver genuinely stands in as the compiler, since its
un-hidden root cause could only be produced by compiling the fixture through the tool.

## Comparison with Clippy

The suite now matches Clippy's *approach* closely: a custom Rust test harness with `harness = false`
and its own `fn main`, driving a tree of `tests/ui/*.rs` fixtures against committed snapshots with a
bless step. The mental model transfers directly, and
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

A third, smaller difference is that the suite pins *two* snapshots per fixture and adds the process
pass — a pure unit check of `process_cgp_errors` that Clippy has no analogue for, because Clippy has
no post-compilation processing stage to unit-test. It is what makes the `--process-only` fast loop
possible.

One gap against Clippy is unrelated to the harness: there is no dogfood test that runs `cargo-cgp` on
this repository's own crates. It becomes worthwhile once the tool does more than pass through.

## Further reading

These are the snapshot harnesses this suite's design draws on; both compile each fixture and diff a
committed snapshot with a bless step, the workflow reproduced here.

- [`ui_test`](https://github.com/oli-obk/ui_test) — the UI-test library Clippy's harness is built on.
- [`trybuild`](https://docs.rs/trybuild) — the compile-fail snapshot harness the parent `cgp` project
  uses for its post-codegen failures.

## Tests

The automated tests are the argument-handling tests, the harness crate's option-parsing and
normalization tests, and the UI snapshot suite — all under each crate's `tests/` directory. There is
no dogfood test yet (see above).

- [`crates/cargo-cgp/tests/args.rs`](../../crates/cargo-cgp/tests/args.rs) — `strip_subcommand` across
  the `cargo cgp check` form, the direct form, a later matching token, and the program-name-only case.
- [`crates/cargo-cgp-driver/tests/args.rs`](../../crates/cargo-cgp-driver/tests/args.rs) — `rustc_args`
  across wrapper-mode stripping, sysroot injection, an existing sysroot, injected-flag appending, and
  an explicit `-Znext-solver` override.
- [`crates/cargo-cgp-ui-tests/tests/options.rs`](../../crates/cargo-cgp-ui-tests/tests/options.rs),
  [`crates/cargo-cgp-ui-tests/tests/normalize.rs`](../../crates/cargo-cgp-ui-tests/tests/normalize.rs)
  — harness option/filter parsing (including `--process-only`) and both output normalizers.
- [`tests/ui/`](../../tests/ui) — the UI snapshot fixtures, each `<name>.rs` paired with a blessed
  `<name>.stderr` and `<name>.output.json`, run by the harness's three passes.

## Source

- [`crates/cargo-cgp-ui-tests/`](../../crates/cargo-cgp-ui-tests) — the custom UI-test harness:
  `tests/ui.rs` (the `harness = false` entrypoint) and the `src/` modules (`options`, `paths`,
  `fixtures`, `harness`, `passes`, `normalize`, `snapshot`).
- [`tests/ui/`](../../tests/ui) — the fixture tree, one scenario per `.rs` file with its `.stderr` and
  `.output.json` snapshots, grouped into the `hidden-root-cause/` / `usability/` / `ok/` category
  subdirectories.
- [`crates/cargo-cgp/src/args.rs`](../../crates/cargo-cgp/src/args.rs),
  [`crates/cargo-cgp-driver/src/args.rs`](../../crates/cargo-cgp-driver/src/args.rs) — the modules the
  argument tests cover.
