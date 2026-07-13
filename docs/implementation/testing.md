# Testing

`cargo-cgp` is tested at two levels: fast tests over the argument-handling logic, and a UI snapshot
suite — a custom Rust test harness, in the style of Clippy's — that compiles example CGP programs
through the real tool and pins both its rendered output (`.cgp.stderr`) and — for contrast — the
output plain `cargo check` produces for the same fixture (`.rust.stderr`). Every test lives in its
crate's `tests/` directory; per [../../AGENTS.md](../../AGENTS.md) the project keeps no inline
`#[cfg(test)]` modules, so all tests are integration tests against a crate's public API.

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
grouped into three top-level categories by the *quality of the output* the tool now produces:
`acceptable/` for output that meets the bar — the diagnostic the tool renders is a clean,
root-cause-first error a reader can act on; `usability/` for errors that still carry the cause but
bury it, the remaining issues; and `ok/` for a fixture that compiles clean and needs no diagnostic
work. Each fixture `<name>.rs` has two siblings: `<name>.cgp.stderr`, the tool's rendered output;
and `<name>.rust.stderr`, what plain `cargo check` prints for the same fixture — the untransformed
"before" against which the tool's `.cgp.stderr` is the "after". A fixture that compiles cleanly has
an empty `.cgp.stderr` and an empty `.rust.stderr`.

Within each category the fixtures are sorted into kind subdirectories. `acceptable/` is split by the
kind of failure the tool resolves — `fields/` and `field-types/` for missing and mistyped fields,
`providers/` for provider dependency chains, `generic/` for generic components, `resolution/` for the
non-field and boundary cases the resolver still reshapes, `use-site/` for consumer-method call
failures, and `lowering/` and `wiring/` for the remaining classes. `usability/` is split by the kind
of issue that remains — `duplication/`, `use-type/`, `lowering/`, and `wiring/` (itself split into
`constraints/`, `duplicate-keys/`, and `namespace-paths/`). Alongside the hand-curated examples the
tree includes a verbatim mirror of the upstream CGP compile-fail suite (one fixture per reproducible
error class), giving the tool a snapshot of its own transformed output for the whole
[error catalog](../../../cgp/docs/errors/README.md). The
[usability fixtures README](../../tests/ui/usability/README.md) records the class-by-class findings.

The suite pins *both* halves of that transformation, so the tool's contribution is legible on the
page. The `.cgp.stderr` is the output of `cargo-cgp` itself — each fixture compiled by running the
real `cargo-cgp check` end to end, front-end, driver, and all, so the snapshot is whatever the tool
emits, already shaped by the driver's emitter. The `.rust.stderr` beside it is the output of plain
`cargo check` on the same fixture, with no driver and no CGP transforms, and it exists as the
recorded "before". Both snapshots come out of the *same* renderer — the compiler's default human
emitter — because the driver renders the human path itself, so the only difference between them is
the transforms the driver applied; their diff is therefore purely the tool's work, cleaner than a
diff across two different renderers would be. Reading the two side by side shows exactly what the
tool changed — the resugared type names, the renamed wiring traits, the CGP error codes on the
messages it fully rewrites. As the tool reshapes more diagnostics in the driver's emitter, the
`.cgp.stderr` is what will change while the `.rust.stderr` stays fixed, so the widening gap between
them is the visible measure of the tool's work, and the `.cgp.stderr` diff is the signal that a
change did what was intended. The suite exists so that is caught the moment it lands.

### Two passes per fixture

Each fixture is verified by two passes: one records the tool's real output, the other the
plain-compiler baseline it improves on. They do not cross-check each other, and there is no separate
capture or unit pass — because the driver applies every CGP transform in-process and renders the
result, `.cgp.stderr` is simply what `cargo-cgp` prints, with nothing to reconcile it against. Both
are implemented in [`passes`](../../crates/cargo-cgp-ui-tests/src/passes.rs):

- **The cgp-stderr pass** runs `cargo-cgp check` directly and compares the tool's rendered stderr to
  `<name>.cgp.stderr`. This is the end-to-end check that the whole binary produces the expected output.
- **The rust-stderr pass** runs plain `cargo check` — no `cargo-cgp`, no driver — and compares its
  rendered stderr to `<name>.rust.stderr`. Nothing cross-checks it, because it is the untransformed
  compiler output, not a tool result; it exists to record the "before" the cgp-stderr pass improves on.

### The harness is a custom Rust test binary

Following Clippy, the suite is a **custom test harness written in Rust**, not a shell script and not
libtest. It lives in its own crate, [`crates/cargo-cgp-ui-tests`](../../crates/cargo-cgp-ui-tests),
whose `ui` test target sets `harness = false` and provides its own `fn main`
([`tests/ui.rs`](../../crates/cargo-cgp-ui-tests/tests/ui.rs)) — the same shape as Clippy's
`tests/compile-test.rs`. The `fn main` is thin; the logic is in the crate's library so it stays small
and testable, split into focused modules: `options` (argument parsing), `paths` (locating the
workspace, fixtures, cgp checkout, and built binaries), `fixtures` (discovery), `harness` (building
the binaries and compiling a fixture in a worker crate, through `cargo-cgp` or plain `cargo`),
`passes` (the two per-fixture passes), `runner` (scheduling fixtures across the worker pool),
`normalize` (rewriting volatile paths and dropping content-free lines out of the output), and
`snapshot` (compare, bless, diff).

The harness crate is a full workspace member, so `cargo test` runs the whole suite alongside the
argument tests. It shells out to `cargo` and `cargo-cgp` and carries no non-std dependencies of its
own — the driver does every diagnostic transform in-process, so the harness only launches processes
and diffs their output, with no need to link the tool's libraries. Running the full suite builds the
front-end and its driver and expects a sibling `cgp` checkout at `../cgp` (which each throwaway crate
depends on by path), so a plain `cargo test` needs both present.

### How a fixture is compiled

A fixture is a loose `.rs` file, so the harness turns it into a crate the tool can check: it
maintains a throwaway crate that depends on `cgp` by path, copies the fixture in as its `src/main.rs`,
and runs `cargo-cgp check -q --color never` there. Naming the crate `ui` keeps cargo's output stable,
and an empty `[workspace]` table in its manifest stops cargo from folding it into the `cargo-cgp`
workspace above it in `target/`. In a full run the cgp-stderr pass runs the tool once and the
rust-stderr pass runs plain `cargo check` once, so the fixture is compiled twice; re-copying it
before each run bumps its mtime, which forces cargo to recompile and re-emit diagnostics rather than
serve a cached build with none.

The rust-stderr pass builds in a *separate* target directory from the `cargo-cgp` pass —
`target-rust/` beside the worker crate's default `target/`. The reason is cargo's fingerprinting:
`cargo-cgp` sets `RUSTC_WORKSPACE_WRAPPER` and plain `cargo` does not, and that variable is part of
the fingerprint, so sharing one target directory would rebuild `cgp` on every alternation between the
wrapped and unwrapped runs. Two directories keep each variant's `cgp` build cached, at the cost of
compiling `cgp` a second time per worker.

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
build-failure summary (`could not compile …`, which is cargo's own output rather than part of any
diagnostic), and a note pointing at a hash-named temp file when a long type is elided. The
driver's `--verbose` suppresses that elision, so the temp-file note never reaches a `.cgp.stderr`; but
the rust-stderr pass runs plain `cargo check` *without* `--verbose`, so a long CGP type can be elided
there and the note does arise in `.rust.stderr` — which is exactly why dropping it earns its keep. The
single `normalize` module handles the rendered stderr of both passes: it rewrites the paths to
`$CGP`/`$DIR` and drops the summary and temp-file lines, so what is compared depends only on the
diagnostic content. Normalization applies to the compared/blessed output only; `--print` shows the raw output untouched.
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
comparing, and `--jobs N` (`-j N`) to set the worker count:

```sh
cargo test -p cargo-cgp-ui-tests --test ui -- usability    # only fixtures whose path contains "usability"
cargo test -p cargo-cgp-ui-tests --test ui -- --bless      # rewrite the .cgp.stderr and .rust.stderr snapshots
cargo test -p cargo-cgp-ui-tests --test ui -- -j 4                    # check at most 4 fixtures at once
cargo test -q -p cargo-cgp-ui-tests --test ui -- --print unsatisfied_dependency  # print raw output
```

**`--jobs N`** (`-j N`) sets how many fixtures the harness checks at once; it defaults to the
machine's parallelism, capped at 8 and at the number of fixtures. Raise it past the default on a
machine that can afford more concurrent `cgp` builds — one per worker, since the workers cannot share
a target directory (above) — or set `-j 1` to run fully sequentially.

After an *intended* change to what the tool emits, `--bless` regenerates both snapshots — the
analogue of Clippy's `cargo bless` — writing `.cgp.stderr` from the real `cargo-cgp` run and
`.rust.stderr` from plain `cargo check`, and the diff is reviewed before committing. Because
`.rust.stderr` records plain `cargo check`, it changes only on a toolchain bump, not when the tool's
own behavior changes.

### Toolchain and determinism

A snapshot is only reproducible against the toolchain it was blessed with, because it contains the
compiler's diagnostic text. The harness builds and runs under the toolchain the repository pins in
[`rust-toolchain.toml`](../../rust-toolchain.toml) (overridable with `RUSTUP_TOOLCHAIN`), and
snapshots must be blessed under that same toolchain. A deliberate toolchain bump can therefore change
the diagnostic wording and require a re-bless, exactly as it does for Clippy — a `.cgp.stderr` or
`.rust.stderr` diff after a toolchain change is expected, not a regression. A passing
`acceptable/use-site/unsatisfied_dependency`
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

A third, smaller difference is that the suite pins *two* snapshots per fixture where Clippy pins one.
The extra one is the `.rust.stderr` baseline: it records plain `cargo check`, which Clippy never
needs because it only *adds* lints to rustc's output rather than rewriting it, so it has no "before"
worth pinning. `cargo-cgp` rewrites, so the before/after pairing is what makes the rewrite legible.
There is no unit-test pass against Clippy's either, and none is possible: the driver applies its
transforms in-process while rendering, so there is no separately renderable stage to check apart from
the end-to-end run.

One gap against Clippy is unrelated to the harness: there is no dogfood test that runs `cargo-cgp` on
this repository's own crates. It becomes worthwhile as the tool's diagnostic transforms grow, when
running them against real crates would catch regressions the curated fixtures miss.

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
  — harness option/filter parsing and the output normalizer.
- [`tests/ui/`](../../tests/ui) — the UI snapshot fixtures, each `<name>.rs` paired with a blessed
  `<name>.cgp.stderr` (the tool's output) and `<name>.rust.stderr` (the plain-`cargo check`
  baseline), run by the harness's two passes.

## Source

- [`crates/cargo-cgp-ui-tests/`](../../crates/cargo-cgp-ui-tests) — the custom UI-test harness:
  `tests/ui.rs` (the `harness = false` entrypoint) and the `src/` modules (`options`, `paths`,
  `fixtures`, `harness`, `passes`, `runner`, `normalize`, `snapshot`).
- [`tests/ui/`](../../tests/ui) — the fixture tree, one scenario per `.rs` file with its `.cgp.stderr`
  and `.rust.stderr` snapshots, grouped into the `acceptable/` / `usability/` / `ok/` category
  subdirectories.
- [`crates/cargo-cgp/src/args.rs`](../../crates/cargo-cgp/src/args.rs),
  [`crates/cargo-cgp-driver/src/args.rs`](../../crates/cargo-cgp-driver/src/args.rs) — the modules the
  argument tests cover.
