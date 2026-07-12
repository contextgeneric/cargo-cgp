# AGENTS.md

This file guides LLM agents working in the `cargo-cgp` repository. Read it before any task here,
then read the sub-crate `AGENTS.md` for whichever crate you are about to touch.

## What this project is

`cargo-cgp` is a cargo subcommand whose eventual goal is to make Context-Generic Programming (CGP)
compiler errors readable. A CGP macro expands to ordinary Rust, so many mistakes are caught not by
the macro but by the compiler type-checking the generated code, where a single error can cascade
across generated types the programmer never wrote and the root cause is often buried or suppressed
entirely. `cargo-cgp` will post-process those diagnostics into a compact, root-cause-first form,
the way Clippy layers analysis on top of `rustc`.

The project is early. Only `cargo cgp check` exists. It compiles the workspace through a
`rustc_driver`-based wrapper, and that wrapper already earns its keep in one way: it injects
`-Znext-solver=globally`, turning on the next-generation trait solver, which surfaces the CGP
dependency errors the default solver hides (it descends to the real missing bound — e.g.
`HasField<Symbol!("name")>` — instead of stopping at the provider trait). Beyond that the output
still matches `cargo check`. The larger payoff is still ahead and rests on the same foothold: full
access to the compiler's internals, which later features will use to read and rewrite CGP
diagnostics. When reasoning about behaviour, remember the tool already diverges from plain
`cargo check` by choice of trait solver.

## Orient before any task

CGP is the subject matter, so load the CGP mental model before reasoning about what this tool
should do to an error. **Invoke the `/cgp` skill** whenever a task touches CGP constructs or the
diagnostics they produce; the skill is the ground truth for what each macro expands to and why its
errors look the way they do.

The `cgp` source is a sibling of this repository, at the parent directory — `../cgp`. Treat it as
read-only reference. Agents may read anything under `../cgp` to understand CGP behaviour, and in
particular the **[CGP error catalog](../cgp/docs/errors/README.md)** is the map of every error class
this tool must eventually recognize: which classes hide the root cause, which surface it, and where
the cause sits when it is present. The catalog is backed by the `trybuild` compile-fail fixtures in
`../cgp/crates/tests/cgp-compile-fail-tests`, which are concrete inputs you can run through
`cargo cgp check` to see a class of error in the raw. **Do not create any dependency from `cgp` on
`cargo-cgp`.** The reference direction is one-way: `cargo-cgp` reads `cgp`, never the reverse, and
nothing in `../cgp` should be edited to accommodate this project.

Two more read-only references sit alongside this repository, and you should use them whenever a task
turns on how the compiler actually behaves rather than on how it is documented to behave. The **Rust
compiler source is at [`../external/rust`](../external/rust)** — consult it to confirm the real
signature and behaviour of a `rustc_driver`/`rustc_interface` API before relying on it, since these
internals are unstable and change between nightlies; grep `compiler/rustc_driver_impl/src` for the
entrypoints this project calls. **Clippy's source is at
[`../external/rust-clippy`](../external/rust-clippy)** — it is the closest working example of the
integration this project performs, so read its `src/main.rs` (the `cargo-clippy` front-end) and
`src/driver.rs` (the `clippy-driver` wrapper) when working out how to wire the driver into cargo,
inject the sysroot, or install callbacks. Prefer verifying against these sources over guessing, but
do not edit them and do not create a dependency on them.

This repository keeps its own knowledge base under [`docs/`](docs/README.md), written by and for
agents. Read it before a task and keep it in sync after one. The
[implementation documentation](docs/implementation/README.md) explains how the tool is built and why
— start with [Executable structure](docs/implementation/executable-structure.md) for the
two-executable design and the cargo wrapping, and [The driver](docs/implementation/driver.md) for the
rustc wrapping, the compiler-API access, and the driver-side diagnostic transformations. The knowledge base
is bound by a **synchronization rule** ([docs/AGENTS.md](docs/AGENTS.md)): the source is the single
source of truth, and when you change how the tool is structured or behaves, you revise the matching
document in the same change. A document describing a design the code no longer has is worse than
none.

When your task involves editing markdown documentation or inline doc comments, **load the
`/dual-reader-prose` skill** and follow its convention for the prose you write.

## Architecture: two binaries, like Clippy

`cargo-cgp` mirrors Clippy's split into a front-end and a driver, and understanding that split is
the key to the whole codebase. The **`cargo-cgp` crate** (`crates/cargo-cgp`) is the front-end: the
cargo subcommand a user invokes, a plain `std` + `anyhow` binary that runs `cargo check` with
`RUSTC_WORKSPACE_WRAPPER` set to the driver. The **`cargo-cgp-driver` crate**
(`crates/cargo-cgp-driver`) is the driver: the `rustc` replacement cargo then calls for each
workspace crate, running the real compiler in-process through `rustc_driver`. They are separate
crates for one concrete reason — only the driver links the compiler's internal libraries, and
keeping that linkage out of the front-end keeps it a small, ordinary binary that builds without
loading LLVM. A third, library-only crate, **`cargo-cgp-error-processing`**
(`crates/cargo-cgp-error-processing`), holds the stateless diagnostic-processing stage the front-end
calls after a build; it links no compiler internals either, so it builds and tests on any toolchain
(see [Error processing](docs/implementation/error-processing.md)).

How the two cooperate — the argument normalization, the `CARGO_CGP_SYSROOT` and
dynamic-library-path contract, wrapper-mode detection, and the front-end capture — is documented in
[Executable structure](docs/implementation/executable-structure.md); the driver's own internals — the
argument preparation, the `rustc_private` compiler-API access, and the three diagnostic
transformations (the `-Znext-solver=globally` and `--verbose` flag injections and the `CgpCallbacks`
emitter that renames CGP wiring notes) — are in [The driver](docs/implementation/driver.md). Read the
relevant one before changing how the executables interact or what the driver does, and keep it in sync
when you do.

## Toolchain and `rustc_private`

The driver links the compiler's unstable internal crates, so it can only be built with a nightly
toolchain that carries the `rustc-dev` component. That toolchain is pinned in
[`rust-toolchain.toml`](rust-toolchain.toml) to an exact dated nightly, because the `rustc_private`
API changes between nightlies; the `rustc-dev` and `llvm-tools` components install automatically on
first build. Bump the pin deliberately, and expect to fix driver code against `rustc_driver` API
changes when you do.

Two `rustc_private` consequences are easy to trip over: the `#![feature(rustc_private)]` gate is
needed on the driver's **binary** crate as well as its library (the binary is what links the compiler
dylib), and the pinned nightly is the compiler the driver *embeds*, so the tool must be run against a
project using that same nightly — for example with `RUSTUP_TOOLCHAIN` set — or it loads a mismatched
`librustc_driver` and fails.
[The driver](docs/implementation/driver.md#accessing-the-rust-compiler-api)
explains both in full.

## Code organization conventions

The code organization follows the same conventions as the `cgp` workspace, especially
`../cgp/crates/macros/cgp-macro-core`. Hold to them when adding code.

Keep each module small and focused on one concept, and give sub-directories a `mod.rs` that contains
nothing but re-exports of its child modules (`mod child; pub use child::*;`). A top-level `lib.rs`
declares its modules with `pub mod`. Keep code loosely coupled by passing values as parameters
rather than reaching for hardcoded constants: the few unavoidable well-known strings (the cargo
subcommand name, the driver executable name, the sysroot environment variable) live in a `config`
module and are passed into the functions that use them, so call sites stay independent of the
literals. Do not put logic in a `bin` target; implement everything as library functions and let the
`bin` file be a lightweight wrapper around the entrypoint (the one exception is the driver binary's
`#![feature(rustc_private)]` gate, which must be on the binary crate for linking).

Keep tests out of `src/`. Do not write inline `#[cfg(test)] mod tests` blocks; every test lives in
the crate's `tests/` directory alongside `src/`, as an integration test exercising the crate's
public API. This keeps source files focused on the code, and it means a function worth testing is
reached through the crate's public surface rather than through module-private access — expose it (or
test it via a public entry point) instead of reaching inside. The custom UI harness is consistent
with this: its `harness = false` target and its plain tests both live under
`crates/cargo-cgp-ui-tests/tests/`.

Keep inline docs brief and current as you write. Add a one-line `///` to any public item that lacks
one, prefer naming the *why* or a corner case over restating the signature, and delete a comment
that only repeats the code.

### Module map

The front-end (`crates/cargo-cgp/src`) is organized around dispatch and the `check` command.
`run.rs` is the entrypoint that normalizes arguments and dispatches on the subcommand; `args.rs`
strips the cargo-inserted `cgp` token so the same entrypoint serves both `cargo cgp check` and a
direct `cargo-cgp check`; `config.rs` holds the shared well-known names. The `check/` directory
holds the command itself: `command.rs` builds and runs the wrapped `cargo check` (with
`--message-format=json`), captures its output, and re-emits the processed diagnostics; `diagnostics.rs`
parses cargo's JSON stream and re-renders the processed result; `driver_path.rs` locates the sibling
driver executable; and `sysroot.rs` discovers the toolchain sysroot.

The driver (`crates/cargo-cgp-driver/src`) is smaller. `run.rs` is the entrypoint that runs the
compiler through `rustc_driver`; `args.rs` turns the wrapper's process arguments into a rustc
argument vector (dropping the injected `rustc` path and injecting `--sysroot`); `callbacks.rs` holds
the `Callbacks` implementation, whose `config` hook installs a diagnostic-rewriting emitter;
`emitter.rs` is that emitter, which renames CGP wiring messages using the live compiler and, when it
can, transforms a wiring failure into its root-cause dependency tree (rewriting a main message that
is an identified CGP class into its `[CGP-Exxx]`-coded form — the Rust code kept — and swapping the
sub-notes for one `root cause:` note per leaf);
`resolve.rs` is that typed resolver, recovering the failing obligation either from a `check_components!`
entry or from the use site of a broken consumer-method call (`E0599`), then descending the wiring to
each terminal leaf (a `HasField` field or an ordinary bound) (see
[Typed root-cause resolution](docs/implementation/typed-root-cause-resolution.md));
`component_map.rs` builds the component-marker → consumer/provider trait-name map by querying the
trait solver; `config.rs` holds the shared names. The compiler-free string rewrite and the lazily-built
`ComponentNameMap` it uses live in the `cargo-cgp-error-processing` crate (the driver's one ordinary
dependency), so they are unit-tested without the driver's `rustc_private` linkage.

The processing library (`crates/cargo-cgp-error-processing/src`) is the smallest and holds no
compiler linkage. `process.rs` is the stateless `process_cgp_errors` entrypoint, which wraps each
diagnostic and runs the per-diagnostic preprocessing pipeline in `preprocess/` (stripping CGP path
prefixes, resugaring `Symbol!`, rewriting unmet `HasField` bounds into missing-field messages);
`diagnostic.rs` defines the `CgpDiagnostic` output type. Because this crate is rustc-free, it is also
the home of two driver-driven helpers, hosted here so they can be unit-tested without the driver's
compiler linkage: `rewrite.rs` — the compiler-free string rewrite that renames CGP wiring messages and
the lazily-built `ComponentNameMap` it uses — and `tree.rs` — the `DependencyTree` type and its
`cargo tree`-style renderer (over the `termtree` crate) that the driver's typed resolver uses to show a
check failure's transitive dependency chain. Both are driven by the *driver*, not by
`process_cgp_errors`; `code.rs` holds the `CGP-E` error-code constants they stamp on classified main
messages (catalogued in docs/error-code.md). Its tests in `tests/` drive the preprocessors, `process_cgp_errors`, the rewrite,
and the tree renderer over committed fixtures and hand-built inputs, so they run on any toolchain. The
cross-diagnostic aggregation sub-stage (collapsing cascades) is still to come.

## Commands

The commands mirror the `cgp` workspace. Run them from the repository root.

- **Build:** `cargo build` builds both binaries into `target/debug`. They must stay in the same
  directory, since the front-end locates the driver as its sibling.
- **Format:** `cargo +nightly fmt --all` (check with `-- --check`). The `.rustfmt.toml` uses the
  same unstable `group_imports`/`imports_granularity` settings as `cgp`, so formatting needs
  nightly.
- **Lint:** `cargo clippy --all-targets -- -D warnings`.
- **Test:** `cargo test` runs everything — the argument-handling tests in both tool crates, the
  processing library's fixture tests (which run on any toolchain, needing no compiler), and the UI
  snapshot suite (below), which builds the driver and expects a sibling `cgp` checkout at `../cgp`.
  Every test lives in its crate's `tests/` directory (no inline `#[cfg(test)]`); prefer adding
  coverage there over ad-hoc checks.

### UI snapshot tests and running the tool

The UI suite is a custom Rust test harness modeled on Clippy's `compile-test`: the
[`cargo-cgp-ui-tests`](crates/cargo-cgp-ui-tests) crate has a `harness = false` test with its own
`fn main` that checks each fixture under [`tests/ui/`](tests/README.md) through four passes — three
that must agree, plus a plain-compiler baseline. The agreeing three run `cargo-cgp` and diff its
stderr against `<name>.cgp.stderr`, capture the diagnostics it feeds to processing and diff them
against `<name>.output.json`, and parse that JSON through `process_cgp_errors` and diff the rendered
result back against `<name>.cgp.stderr`. The fourth runs plain `cargo check` and diffs its stderr
against `<name>.rust.stderr`, recording the untransformed "before" so the diff against `.cgp.stderr`
shows what the tool changes. The crate is a full workspace member, so `cargo test` runs the whole
suite alongside the argument tests; a full run builds the driver and expects a sibling `cgp` checkout
at `../cgp`. Work with the suite directly through it:

```sh
cargo test -p cargo-cgp-ui-tests                                  # just the snapshot suite
cargo test -p cargo-cgp-ui-tests --test ui -- --bless             # regenerate .cgp.stderr, .rust.stderr, and .output.json
cargo test -p cargo-cgp-ui-tests --test ui -- --process-only      # only the process_cgp_errors unit pass (fast, no compile)
cargo test -p cargo-cgp-ui-tests --test ui -- -j 4                # check at most 4 fixtures at once
cargo test -q -p cargo-cgp-ui-tests --test ui -- --print greet    # print raw output for a fixture
```

Passing an argument to the harness needs `--test ui`, so the flag is not also handed to the crate's
other (libtest) tests. The harness checks fixtures in parallel across a pool of workers, each with its
own throwaway crate (so they never share a `src/main.rs` or a cargo target lock); `--jobs`/`-j` sets
the worker count, which otherwise defaults to the machine's parallelism capped at 8. `--process-only` is the fast
loop for iterating on the processing
implementation: it skips the three cargo-invoking passes and runs only `process_cgp_errors` over the
committed `.output.json`, so the whole suite finishes in well under a second; pair it with `--bless`
to re-bless `.cgp.stderr` from the new process output. The snapshots capture `cargo-cgp`'s own output end
to end, so they are what changes once the tool reformats diagnostics; a passing suite is also the
standing end-to-end proof that the driver runs as the compiler. Add a scenario by dropping a
`<name>.rs` file (with a `fn main`) into the matching `tests/ui/<class>/` directory and running
`cargo test -p cargo-cgp-ui-tests --test ui -- --bless` (a full run, which writes all three snapshot
files). Snapshots are blessed under the pinned toolchain, so a toolchain bump can require a re-bless. The full testing picture — the
harness structure, why it drives the whole tool rather than the driver directly, and the comparison
with Clippy — is documented in [Testing](docs/implementation/testing.md); read it before adding
tests, and keep it in sync when the test setup changes.

## Ask when in doubt

When something should be settled before the next step — an ambiguous intended behaviour, a design
choice with more than one defensible answer, or a change that would couple this project to `cgp` —
surface the question rather than guessing.
