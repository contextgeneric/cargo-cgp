# cargo-cgp UI tests

This directory holds the UI test fixtures for `cargo-cgp`: small, standalone CGP programs, each
compiled through the tool and compared against committed snapshots of its output. It is modeled on
Clippy's `tests/ui/`. The fixtures are the canonical reproductions of CGP's post-codegen error
classes: `cgp`'s
[error catalog](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/README.md)
links each class to the fixtures here that pin it.

For how the fixtures fit into the project's testing (the unit tests, the harness mechanics, the
toolchain caveat, and the comparison with Clippy), see the
[Testing](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/implementation/testing.md)
implementation document. This README is the quick operational guide.

## Layout

Each fixture `<name>.rs` has three snapshots beside it: `<name>.cgp.stderr`, the tool's rendered
output; `<name>.rust.stderr`, what plain `cargo check` prints for the same program (the
untransformed "before"); and `<name>.expand.rs`, the Rust the fixture's CGP macros generate, as
`cargo cgp expand` shows it. A fixture marked `//@json` also has `<name>.cgp.json`, the compiler
messages of the tool's JSON output. A snapshot depends only on the fixture's content, never on its
directory, so moving a fixture needs no re-bless.

**Fixtures are grouped by the error class they reproduce**, so a fixture moves only when its class
changes, never when the tool's message for it improves. The class directories under [`ui/`](ui)
mirror the error catalog:

- [`ui/checks/`](ui/checks): failures a check trait surfaces, one directory per class
  (`check-trait-failure/`, `higher-order-provider-layer/`, `ordinary-trait-bound/`,
  `unregistered-namespace-path/`, `verbose-cascade/`). The largest, `check-trait-failure/`, keeps
  concept sub-directories such as `fields/`, `providers/`, and `missing-wiring/`.
- [`ui/hidden/unsatisfied-dependency/`](ui/hidden/unsatisfied-dependency): an unmet dependency
  reached by calling a method rather than through a check, whose cause plain `cargo check` hides.
- [`ui/lowering/`](ui/lowering): failures in what a macro lowered the user's input into, among
  them `undeclared-dependency/` (a trait, inner provider, or abstract type a definition uses
  without declaring it) and `wrong-component-half/` (a construct that names or sits on the wrong
  half of a component, such as a consumer trait where its provider trait belongs).
- [`ui/wiring/`](ui/wiring): structural wiring failures, such as conflicting wiring, orphan-rule
  violations, cycles, and unconstrained generics, and `unresolved-wiring-name/` (a wiring key,
  value, or provider that names nothing, or the wrong kind of item).
- [`ui/data/extensible-data/`](ui/data/extensible-data): a builder, extractor, or cast applied in a
  state its value does not allow, or to a type without the data derive it needs.
- [`ui/uncataloged/`](ui/uncataloged): mistakes the catalog does not cover yet, one directory per
  class: `implicit-arguments/` and `macro-defects/` (mistakes a `cgp` macro mishandles).

Three more directories hold fixtures that belong to no error class:

- [`ui/ok/`](ui/ok): correctly wired programs that compile clean.
- [`ui/pass-through/`](ui/pass-through): plain Rust errors in which nothing CGP is at fault, which
  the tool must render exactly as plain `cargo check` does.
- [`ui/robustness/`](ui/robustness): how the tool degrades when the compilation fails under it, such
  as a compiler crash or a fatal error while it holds diagnostics, which must still print every
  error. A panic in the tool's own emitter is pinned by the driver's unit tests instead.

Every fixture name is unique across the tree, so `find tests/ui -name '<name>.rs'` locates any
fixture.

## The fixture header

A fixture opens with a `//!` comment stating what it demonstrates and the mistake behind it, then
declares **directives**: line comments starting with `//@`, which the compiler ignores and the
harness reads. Every error fixture (everything outside `ok/`, `pass-through/`, and `robustness/`)
declares two:

```rust
//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E110
```

- `//@quality:` records how well the tool presents the error today: `acceptable` (the tool leads
  with the cause), `usability` (the cause is present but buried), or `hidden` (the cause is absent
  from the output). Improving a message changes this field, not the fixture's directory.
- `//@codes:` lists the exact set of `[CGP-Exxx]` codes the tool's output carries, or `none`. The
  harness fails a fixture whose output gains or loses a code.

Four more directives are used where they apply:

- `//@aux-build: <crate>`: build an auxiliary crate and depend on it (see below).
- `//@rewrites-falsely: <reason>`: a `pass-through/` fixture whose output still differs from plain
  rustc's, for a fix still to come or a documented limitation; its snapshots pin today's output, and
  a fix that closes the difference removes the marker.
- `//@json`: also run the JSON pass.
- `//@env: <KEY>=<VALUE>`: set an environment variable for the tool's runs. A `robustness/`
  fixture sets `RUSTFLAGS=-Ztreat-err-as-bug=2` this way to make the compiler crash.

An unknown directive, or a known one written with a space after `//`, is rejected rather than
ignored.

## The checks

Beside the three snapshot comparisons, each fixture runs these checks:

- **Exit status:** the tool and plain `cargo check` must agree on whether the fixture compiles.
- **Pass-through:** for a fixture under `pass-through/`, the tool's output must equal plain
  rustc's.
- **Header:** an error fixture must declare its quality, and its `//@codes:` must match the output.

A mismatch prints a line diff for each failing pass, and a missing snapshot fails rather than
passing as empty.

## Cross-crate fixtures

Most fixtures are one standalone crate depending only on `cgp`, but a cross-crate scenario, such as
the orphan rule or cross-crate coherence, exists only *between* crates, so the harness supports
**auxiliary crates**. A fixture opts in with a directive naming a companion crate:

```rust
//@aux-build: cgp-test-crate-a
```

The named crate's source lives under
[`crates/cargo-cgp-ui-tests/auxiliary/`](../crates/cargo-cgp-ui-tests/auxiliary). The harness
materializes it against the sibling `cgp` checkout (generating its manifest so its `cgp` path
resolves) and adds it as a path dependency of the throwaway crate before compiling. A fixture may
declare several. The auxiliary crates and the fixtures they back:

- `cgp-test-crate-a` (upstream: a foreign namespace, component, and getter) and `cgp-test-crate-b`
  (downstream: the orphan-safe wirings against it). They back the three orphan-rule failures in
  [`ui/wiring/orphan-rule/`](ui/wiring/orphan-rule), which `cargo-cgp` reshapes into
  `[CGP-E022]`, and the clean [`ui/ok/cross_crate_wiring.rs`](ui/ok/cross_crate_wiring.rs).
- `cgp-test-foreign-pipeline`: a pipeline context and its wiring for
  [`foreign_context_error_route`](ui/hidden/unsatisfied-dependency/foreign_context_error_route.rs),
  whose root cause the tool recovers though the context is defined in another crate.
- `cgp-test-dispatch-traits` (upstream: `#[cgp_auto_dispatch]` traits with no implementor, plus a
  `#[cgp_computer]` and a `#[cgp_producer]`) and `cgp-test-dispatch-shapes` (payloads implementing
  those traits and an enum over them). They back the clean
  [`ui/ok/cross_crate_dispatch_enum.rs`](ui/ok/cross_crate_dispatch_enum.rs) and
  [`ui/ok/cross_crate_dispatch_payloads.rs`](ui/ok/cross_crate_dispatch_payloads.rs).
- `cgp-test-plain-macros`: attribute macros of a crate with no `cgp` dependency, whose expansions
  fail the way a CGP macro's can. They back
  [`ui/pass-through/generated_lifetime_without_cgp.rs`](ui/pass-through/generated_lifetime_without_cgp.rs)
  and
  [`ui/pass-through/attribute_splice_without_cgp.rs`](ui/pass-through/attribute_splice_without_cgp.rs),
  which pin that a crate without CGP keeps the suggestions the tool drops in one with it, and
  [`ui/pass-through/respanned_token_suggestion.rs`](ui/pass-through/respanned_token_suggestion.rs),
  which pins that a CGP crate keeps a suggestion at a token an attribute macro re-spanned, and
  [`ui/pass-through/function_macro_suggestion.rs`](ui/pass-through/function_macro_suggestion.rs),
  which pins that it keeps one at the end of a function-like macro's call.

One class compiles clean under the tool: two namespaces that inherit from each other, pinned by
[`namespace_inheritance_cycle`](ui/wiring/namespace-inheritance-cycle/namespace_inheritance_cycle.rs).
Stable `rustc` rejects it with an `E0275` overflow, but on the pinned nightly both the default
solver and the next-generation solver accept it, so both snapshots are empty. It is a *missing*
error rather than a suppressed cause, and a documented limitation of checking with the next solver
(see
[The driver](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/implementation/driver.md#choosing-the-trait-solver)).
Its exit-status check fails if a later nightly makes the two compilers disagree.

## Running

The suite is a custom Rust test harness in the [`cargo-cgp-ui-tests`](../crates/cargo-cgp-ui-tests)
crate, modeled on Clippy's `compile-test`. Run it with `cargo test`:

```sh
cargo test -p cargo-cgp-ui-tests            # run the whole suite
```

To filter, bless, print, or list, pass an argument to the harness, targeting `--test ui` so the
flag is not also handed to the crate's other tests:

```sh
cargo test -p cargo-cgp-ui-tests --test ui -- wiring/      # only fixtures whose path contains "wiring/"
cargo test -p cargo-cgp-ui-tests --test ui -- --bless      # regenerate the snapshots
cargo test -p cargo-cgp-ui-tests --test ui -- -j 4         # check at most 4 fixtures at once
cargo test -q -p cargo-cgp-ui-tests --test ui -- --print unsatisfied_dependency  # raw output of all three passes
cargo test -q -p cargo-cgp-ui-tests --test ui -- --list | grep ^usability        # the fixtures still to improve
```

Fixtures are checked in parallel across a pool of workers, one throwaway crate each, with
`--jobs`/`-j` setting the count (default: the machine's parallelism, capped at 8). Each fixture's
result prints the moment it finishes, so the run streams in completion order. Snapshots are blessed
under the toolchain the repository pins, so a toolchain bump can require a re-bless.

## Adding a fixture

Put a new `<name>.rs` in the directory of the error class it reproduces (or `ok/` for a clean
compile, `pass-through/` for a plain Rust error, `robustness/` for a failure of the tool itself),
with a name no other fixture uses. Give it a `fn main`, since the harness compiles it as a binary.
Open it with a `//!` comment stating what the scenario demonstrates and the mistake behind it,
linking the
[CGP error class](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/README.md)
it reproduces and, for a problem case, the
[issue](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/README.md)
it exposes. Follow it with the `//@quality:` and `//@codes:` directives. `cgp` is available to every
fixture, so a fixture may `use cgp::prelude::*;` with no setup.

Then run `cargo test -p cargo-cgp-ui-tests --test ui -- --bless`, which writes the snapshots, and
review them before committing. The `.expand.rs` is worth reading as carefully as the `.stderr` pair:
it shows the code the fixture's macros generate, so a surprise there, such as a construct left as a
raw type-level spine, is a finding of its own. When a `cgp` change alters one of these error
classes, update the fixture here and the class document in the error catalog together.
