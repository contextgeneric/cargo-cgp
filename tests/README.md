# cargo-cgp UI tests

This directory holds the UI test fixtures for `cargo-cgp`: small, standalone CGP source files, each
compiled through the tool and compared against a committed snapshot of its output. It is modeled on
Clippy's `tests/ui/`, pairing each `.rs` fixture with a blessed expected-output file. These fixtures
are the canonical reproductions of CGP's post-codegen error classes: those cases were migrated here
from `cgp`'s former `cgp-compile-fail-tests` suite (since removed), so `cargo-cgp` — the tool that
rewrites those errors — owns the snapshots of what a reader actually sees. `cgp`'s
[error catalog](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/README.md) links each
error class to the fixture here that pins it.

For how the fixtures fit into the project's overall testing approach — the argument tests, the
harness mechanics, the toolchain caveat, and the comparison with Clippy — see the
[Testing](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/implementation/testing.md) implementation document. This README is the quick
operational guide.

## Layout

Fixtures live under [`ui/`](ui), grouped into category directories by the *quality of the output* the
tool produces for them. Each fixture `<name>.rs` has three siblings: `<name>.cgp.stderr`, the tool's
rendered output; `<name>.rust.stderr`, what plain `cargo check` prints for the same fixture — the
untransformed "before" against which the tool's `.cgp.stderr` is the "after"; and `<name>.expand.rs`,
the Rust the fixture's CGP macros generate, as `cargo cgp expand` shows it. A fixture that compiles
cleanly has an empty `.cgp.stderr` and an empty `.rust.stderr`, but still has an `.expand.rs`: the two
`.stderr` files record what the compiler *says* about the code, and `.expand.rs` records the code it
was actually given, which is usually where the answer to "why does it say that?" is. A snapshot depends only on the
fixture's *content*, never on its directory (the harness copies each fixture into a throwaway crate's
`src/main.rs` before compiling), so moving a fixture between categories needs no re-bless.

The categories are:

- [`ui/acceptable/`](ui/acceptable) — errors whose root cause the tool already presents well: a
  coded `[CGP-Exxx]` headline, a plain-language `root cause:` note, a compact dependency tree, and no
  generated-type scaffolding. This is where an error fixture graduates once it clears the usability
  bar. It is split into concept sub-directories — `fields/`, `field-types/`, `types/`, `providers/`,
  `generic/`, `resolution/`, `use-site/`, `use-type/`, `verbosity/`, `duplication/`, `lowering/`, and
  `wiring/{constrained-key,constraints,duplicate-keys,missing-wiring,namespace-paths,orphan}/` — so no
  directory grows crowded.
- [`ui/usability/`](ui/usability) — errors that carry the root cause but bury it in volume, encoding,
  duplication, or misleading framing (a [usability issue](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md)); the cause is
  present, so the work is re-presentation. It is split into issue-class sub-directories —
  `extensible-data/`, `lowering/`, and `wiring/constraints/` — each naming the problem its fixtures
  expose.
- [`ui/ok/`](ui/ok) — the clean-compile baseline: correctly-wired programs that check with empty
  output.
- `ui/hidden-root-cause/` — errors whose root cause cannot be recovered from the output at all, the
  highest-value class to fix (a [hidden root cause](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/hidden-root-cause.md)). It has
  **no fixture today** — both known archetypes are defeated by flags the driver injects, so the
  directory is absent — but it is recreated the moment a genuinely unrecoverable case is found.

A fixture's placement follows the sufficiency-and-presentation test in
[cgp-knowledge-base/cargo-cgp/issues/](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/README.md): if no downstream tool could recover the cause from the
output, it is `hidden-root-cause/`; if a careful reader could but the output buries it, `usability/`;
if the output already leads with the cause, `acceptable/`; if the program compiles clean, `ok/`.

## Two sources of fixtures

The fixtures come from two origins. The **hand-curated** fixtures are the worked examples and the
regression pins the knowledge base references — the check-trait-failure family (`base_area_*`,
`density_*`, `scaled_area_*`, the consumer-call `unsatisfied_dependency`) and the typed-resolver pins
(`field_type_mismatch*`, `same_name_components`, `generic_area*`, `deep_nesting`, `parallel_branches`,
`dependency_cascade`, `missing_has_field_derive`, `field_via_deref`, `ordinary_bound_unsatisfied`,
`mixed_rust_error`, `empty_field_struct`, and the missing-wiring family `basic_missing_wiring`,
`direct_missing_wiring`, `parallel_missing_wiring`, and the use-site `missing_wiring`), each catalogued under
[Typed root-cause resolution](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/implementation/typed-root-cause-resolution.md#tests). The rest
were **migrated from `cgp`'s former compile-fail suite** — one fixture per post-codegen error class
CGP produces — and are now maintained here directly rather than mirrored from anywhere. A migrated
fixture's `.cgp.stderr` is cargo-cgp's own transformed output, and its `.rust.stderr` is what plain
`cargo check` prints for the same source; its `//!` header names the
[CGP error class](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/README.md) it
reproduces.

**No reproducible class hides its root cause.** Every imported case carries the concrete cause in
cargo-cgp's output, so each is either an `acceptable/` case (the cause is presented well) or a
`usability/` case (the cause is present but buried) — none is a hidden root cause. The sharpest
confirmation is the consumer-call class: it is *hidden* as raw `rustc` (only `E0599` "method exists
but its bounds were not satisfied"), yet under cargo-cgp's next-gen solver the leaf bound is recovered
and the resolver leads with it, so those fixtures sit in
[`ui/acceptable/use-site/`](ui/acceptable/use-site).

## Cross-crate fixtures and the one class with no snapshot

Most fixtures are one standalone crate depending only on `cgp`, but a cross-crate scenario — the
orphan rule, cross-crate coherence — exists only *between* crates, so the harness supports
**auxiliary crates**. A fixture opts in with a header directive naming a companion crate:

```rust
//@aux-build: cgp-test-crate-a
```

The named crate's source lives under
[`crates/cargo-cgp-ui-tests/auxiliary/`](../crates/cargo-cgp-ui-tests/auxiliary); the harness
materializes it against the sibling `cgp` checkout (generating its manifest so its `cgp` path
resolves) and adds it as a path dependency of the throwaway crate before compiling. Two aux crates
carry the cross-crate CGP surface, migrated from `cgp`: `cgp-test-crate-a` (upstream — a foreign
namespace, component, and getter) and `cgp-test-crate-b` (downstream — the orphan-*safe* wirings
against it). They back two kinds of fixture:

- **The three orphan-rule failures** — `default_impl_foreign_component`,
  `default_impl_foreign_prefix_path`, and `reopen_foreign_namespace`, in
  [`ui/acceptable/wiring/orphan/`](ui/acceptable/wiring/orphan) — each `//@aux-build: cgp-test-crate-a`
  so the `E0210` orphan violation can arise against a foreign namespace. cargo-cgp reshapes each into
  a `[CGP-E011]` header naming the foreign namespace and key, with the ownership fix in a `help`.
- **The positive counterpart** — [`ui/ok/cross_crate_wiring.rs`](ui/ok/cross_crate_wiring.rs) builds
  `cgp-test-crate-b` (and transitively `cgp-test-crate-a`) to confirm every orphan-*safe* cross-crate
  impl compiles cleanly.

One upstream class has **no snapshot at all**: `inheritance_cycle`, two namespaces that inherit from
each other. Plain `rustc` rejects it eagerly with an `E0275` overflow, but under cargo-cgp's next-gen
solver it **compiles clean**, so there is no error to reproduce. This is a *missing* error, not a
suppressed cause — the "reverse" of the next-solver compatibility caveat noted in
[The driver](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/implementation/driver.md#choosing-the-trait-solver) — and it is recorded here
rather than committed as a misleading empty snapshot.

## Running

The suite is a custom Rust test harness in the [`cargo-cgp-ui-tests`](../crates/cargo-cgp-ui-tests)
crate (modeled on Clippy's `compile-test`). It checks every fixture through three passes: it runs
`cargo-cgp` and diffs its stderr against `.cgp.stderr`, it runs plain `cargo check` and diffs its
stderr against `.rust.stderr` (the untransformed baseline), and it runs `cargo cgp expand` and diffs
its stdout against `.expand.rs`. Run it with `cargo test`:

```sh
cargo test -p cargo-cgp-ui-tests            # run the whole suite
```

To filter, bless, or print, pass an argument to the harness — target `--test ui` so the flag is not
also handed to the crate's other tests:

```sh
cargo test -p cargo-cgp-ui-tests --test ui -- acceptable  # only fixtures whose path contains "acceptable"
cargo test -p cargo-cgp-ui-tests --test ui -- --bless     # regenerate all three snapshots per fixture
cargo test -p cargo-cgp-ui-tests --test ui -- -j 4        # check at most 4 fixtures at once
cargo test -q -p cargo-cgp-ui-tests --test ui -- --print unsatisfied_dependency  # raw output
```

Fixtures are checked in parallel across a pool of workers — one throwaway crate each, so they never
collide — with `--jobs`/`-j` setting the count (default: the machine's parallelism, capped at 8).
Each fixture's result prints the moment it finishes, so the run streams live in completion order;
every line names its fixture. Snapshots are blessed under the toolchain the repository pins, so a
toolchain bump can require a re-bless.

## Adding a fixture

Drop a new `<name>.rs` into the category sub-directory under `ui/` its output belongs to — the concept
group in `acceptable/` if the tool presents its cause well, the issue-class group in `usability/` if
the cause is buried, or `ok/` for a clean compile. Give it a `fn main`, since the harness compiles it
as a binary, and open it with a `//!` comment stating what the scenario demonstrates, which
[CGP error class](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/README.md) it
reproduces, and — for a problem case — the [issue](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/README.md) it exposes. `cgp` is
available to every fixture, so a fixture may `use cgp::prelude::*;` with no setup; for a cross-crate
scenario, add a `//@aux-build: <crate>` directive (see above). Then run
`cargo test -p cargo-cgp-ui-tests --test ui -- --bless` (which writes all three snapshots) and review
them before committing. The `.expand.rs` is worth reading as carefully as the `.stderr` pair: it shows
the code the fixture's macros generate, so a surprise there — a construct left as a raw type-level
spine, say — is a finding of its own.

## Maintaining the migrated fixtures

These fixtures are owned by `cargo-cgp` — there is no upstream suite to re-copy from (the migration
was one-way, and `cgp`'s `cgp-compile-fail-tests` has been removed). Change a fixture like any other:
edit the `.rs` and re-bless. When `cgp` changes a construct in a way that alters one of these error
classes, the [sync rule](../AGENTS.md#the-sibling-projects) applies in both directions —
update the fixture here and the class doc in `cgp`'s error catalog together. One historical note: two
fixtures were renamed on migration to avoid a collision, since `duplicate_path_key` existed under two
CGP constructs — `namespace_duplicate_path_key` (reshaped into `[CGP-E008]`) and
`delegate_duplicate_path_key` (reshaped into `[CGP-E004]`), both in
[`ui/acceptable/wiring/namespace-paths/`](ui/acceptable/wiring/namespace-paths).
