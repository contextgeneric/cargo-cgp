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

Fixtures live under [`ui/`](ui), grouped into category directories by the *quality of the output* the
tool produces for them. Each fixture `<name>.rs` has two siblings: `<name>.cgp.stderr`, the tool's
rendered output, and `<name>.rust.stderr`, what plain `cargo check` prints for the same fixture — the
untransformed "before" against which the tool's `.cgp.stderr` is the "after". A fixture that compiles
cleanly has an empty `.cgp.stderr` and an empty `.rust.stderr`. A snapshot depends only on the
fixture's *content*, never on its directory (the harness copies each fixture into a throwaway crate's
`src/main.rs` before compiling), so moving a fixture between categories needs no re-bless.

The categories are:

- [`ui/acceptable/`](ui/acceptable) — errors whose root cause the tool already presents well: a
  coded `[CGP-Exxx]` headline, a plain-language `root cause:` note, a compact dependency tree, and no
  generated-type scaffolding. This is where an error fixture graduates once it clears the usability
  bar. It is split into concept sub-directories — `fields/`, `field-types/`, `providers/`,
  `generic/`, `resolution/`, `use-site/`, `wiring/`, and `lowering/` — so no directory grows crowded.
- [`ui/usability/`](ui/usability) — errors that carry the root cause but bury it in volume, encoding,
  duplication, or misleading framing (a [usability issue](../docs/issues/usability.md)); the cause is
  present, so the work is re-presentation. It is split into issue-class sub-directories —
  `duplication/`, `use-type/`, `lowering/`, and `wiring/{duplicate-keys,namespace-paths,constraints}/`
  — each naming the problem its fixtures expose.
- [`ui/ok/`](ui/ok) — the clean-compile baseline: correctly-wired programs that check with empty
  output.
- `ui/hidden-root-cause/` — errors whose root cause cannot be recovered from the output at all, the
  highest-value class to fix (a [hidden root cause](../docs/issues/hidden-root-cause.md)). It has
  **no fixture today** — both known archetypes are defeated by flags the driver injects, so the
  directory is absent — but it is recreated the moment a genuinely unrecoverable case is found.

A fixture's placement follows the sufficiency-and-presentation test in
[docs/issues/](../docs/issues/README.md): if no downstream tool could recover the cause from the
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
[Typed root-cause resolution](../docs/implementation/typed-root-cause-resolution.md#tests). The rest
are a **verbatim mirror** of the upstream CGP compile-fail suite (the `acceptable/` fixtures under
[`cgp-compile-fail-tests`](../../cgp/crates/tests/cgp-compile-fail-tests/tests)), imported so
cargo-cgp has a snapshot of its own transformed output for every error class a single-crate harness
can reproduce. An imported `.rs` is an unchanged copy of its upstream counterpart (header included, so
its `//!` comment refers into the `cgp` checkout); its `.cgp.stderr` is cargo-cgp's output — not the
upstream `trybuild` snapshot — and its `.rust.stderr` is what plain `cargo check` prints for the same
copy.

**No reproducible class hides its root cause.** Every imported case carries the concrete cause in
cargo-cgp's output, so each is either an `acceptable/` case (the cause is presented well) or a
`usability/` case (the cause is present but buried) — none is a hidden root cause. The sharpest
confirmation is the consumer-call class: it is *hidden* as raw `rustc` (only `E0599` "method exists
but its bounds were not satisfied"), yet under cargo-cgp's next-gen solver the leaf bound is recovered
and the resolver leads with it, so those fixtures sit in
[`ui/acceptable/use-site/`](ui/acceptable/use-site).

## Four upstream fixtures are intentionally not imported

The harness compiles each fixture as one standalone crate depending only on `cgp`, and that boundary
makes four upstream fixtures impossible to reproduce faithfully, so they are left out rather than
committed with a misleading snapshot:

- **The three cross-crate orphan-rule fixtures** — `default_impl_foreign_component`,
  `default_impl_foreign_prefix_path`, and `reopen_foreign_namespace` — each `use cgp_test_crate_a`, a
  sibling crate in the `cgp` workspace that supplies a *foreign* namespace and component so the
  orphan-rule violation (`E0210`/`E0117`) can arise. The harness cannot provide that crate, so the
  fixtures would fail with a bogus `E0432 unresolved import` and the intended error is never reached.
- **`inheritance_cycle`** — two namespaces that inherit from each other. Upstream, plain `rustc`
  rejects it eagerly with an `E0275` overflow; under cargo-cgp's next-gen solver it **compiles clean**,
  so there is no error to snapshot. This is a *missing* error, not a suppressed cause — the "reverse"
  of the next-solver compatibility caveat noted in
  [The driver](../docs/implementation/driver.md#choosing-the-trait-solver).

## Running

The suite is a custom Rust test harness in the [`cargo-cgp-ui-tests`](../crates/cargo-cgp-ui-tests)
crate (modeled on Clippy's `compile-test`). It checks every fixture through two passes: it runs
`cargo-cgp` and diffs its stderr against `.cgp.stderr`, and it runs plain `cargo check` and diffs its
stderr against `.rust.stderr`, the untransformed baseline. Run it with `cargo test`:

```sh
cargo test -p cargo-cgp-ui-tests            # run the whole suite
```

To filter, bless, or print, pass an argument to the harness — target `--test ui` so the flag is not
also handed to the crate's other tests:

```sh
cargo test -p cargo-cgp-ui-tests --test ui -- acceptable  # only fixtures whose path contains "acceptable"
cargo test -p cargo-cgp-ui-tests --test ui -- --bless     # regenerate the .cgp.stderr and .rust.stderr snapshots
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
[CGP error class](../../cgp/docs/errors/README.md) it reproduces, and — for a problem case — the
[issue](../docs/issues/README.md) it exposes. `cgp` is available to every fixture, so a fixture may
`use cgp::prelude::*;` with no setup. Then run `cargo test -p cargo-cgp-ui-tests --test ui -- --bless`
(which writes both snapshots) and review them before committing.

## Keeping the imported mirror in sync

The imported `.rs` files are verbatim copies, so refreshing an existing one is a re-copy from upstream
over its current location (find it by name under `acceptable/` or `usability/`) followed by a re-bless
(`cargo test -p cargo-cgp-ui-tests --test ui -- --bless`). When upstream adds a fixture, add it under
the `acceptable/` or `usability/` sub-directory that matches the quality of the output cargo-cgp
produces for it — unless it is one of the cross-crate or next-solver-divergent cases above, which are
recorded there rather than imported. The one edit made on import is disambiguating a name collision:
`duplicate_path_key` exists under two upstream constructs, so the copies are
`namespace_duplicate_path_key` (a single raw `E0119`, still under
[`ui/usability/wiring/namespace-paths/`](ui/usability/wiring/namespace-paths)) and
`delegate_duplicate_path_key` (reshaped into `[CGP-E004]`, graduated to
[`ui/acceptable/wiring/namespace-paths/`](ui/acceptable/wiring/namespace-paths)).
