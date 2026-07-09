# Usability fixtures

These are the fixtures for the [usability](../../../docs/issues/usability.md) category: CGP
compile errors that carry their root cause but bury it in volume, encoding, or misleading framing, so
the work is re-presentation rather than recovery. Every fixture here is one whose cause cargo-cgp's
transformed output (`-Znext-solver=globally` + `--verbose`) does contain — a fixture whose cause were
*absent* would belong under [hidden-root-cause](../../../docs/issues/hidden-root-cause.md) instead.

The fixtures are grouped by the *kind* of quality issue, following the CGP error catalog's own
top-level sections so a reader lands on the same class the catalog documents:

- [`checks/`](checks) — a dependency or bound forced through `check_components!`, where `IsProviderFor`
  surfaces the concrete unmet bound (a missing field, a missing derive, an unmet abstract-type
  capability, an ordinary bound, a higher-order layer, an unregistered namespace path). The cause is
  present; the burden is volume and encoding.
- [`wiring/`](wiring) — structural failures with a definite error code (`E0119`, `E0428`, `E0207`,
  `E0275`): keys or names wired twice, overlapping or overriding namespace forwarding, an
  unconstrained per-entry generic, a `UseContext` wiring cycle. The cause is the named code and its
  spans; the burden is mapping the code back to the wiring mistake.
- [`lowering/`](lowering) — the macro lowered accepted input into ill-formed Rust: an unsized
  generated type (`Option<&[T]>`), a `#[use_type]` import of an associated type that does not exist
  (`E0576`), or a cyclic `#[use_type]` routing that resolves to nothing (`E0425`).
- [`unsatisfied-dependency/`](unsatisfied-dependency) — the catalog's *hidden* class, an impl-side
  dependency reached by a direct consumer-method call (`E0599`). Raw `rustc` suppresses the dependency
  here; cargo-cgp's next-gen solver recovers the missing `HasField`/ordinary bound into the same
  diagnostic, which is why these live under `usability/` rather than as hidden causes.

## Two sources of fixtures

Each kind directory mixes fixtures from two origins. The **hand-curated** fixtures (`base_area_*`,
`density_*`, `scaled_area_*`, `unsatisfied_dependency`) are the worked examples the
[usability issue document](../../../docs/issues/usability.md) walks through in prose; they all
exercise the check-trait-failure family (so they sit in `checks/`) apart from `unsatisfied_dependency`
(the consumer-call form, in `unsatisfied-dependency/`). The remaining fixtures are a **verbatim mirror**
of the upstream CGP compile-fail suite — the `acceptable/` fixtures under
[`cgp-compile-fail-tests`](../../../../cgp/crates/tests/cgp-compile-fail-tests/tests), the concrete
reproductions behind the [CGP error catalog](../../../../cgp/docs/errors/README.md) — imported so
cargo-cgp has a snapshot of its own transformed output for every error class a single-crate harness
can reproduce. An imported `.rs` is an unchanged copy of its upstream counterpart (header included, so
its `//!` comment refers into the `cgp` checkout); its `.stderr` is cargo-cgp's output, not the
upstream `trybuild` snapshot.

## What the import found

**No reproducible class hides its root cause.** Every imported case carries the concrete cause in
cargo-cgp's output — the missing `HasField<Symbol!…>` bound, the unmet ordinary bound (`f64: Eq`), the
conflicting-impl spans, the unconstrained parameter, the ill-formed generated type — so all of them are
usability cases, none a hidden root cause. The sharpest confirmation is `unsatisfied-dependency/`: that
class is *hidden* as raw `rustc` (only `E0599` "method exists but its bounds were not satisfied"), yet
under cargo-cgp's next-gen solver the leaf `HasField`/ordinary bound is recovered into the diagnostic.

## Four upstream fixtures are intentionally not imported

The single-crate harness compiles each fixture as one crate depending only on `cgp`, and that boundary
makes four upstream fixtures impossible to reproduce faithfully, so they are left out rather than
committed with a misleading snapshot:

- **The three cross-crate orphan-rule fixtures** — `default_impl_foreign_component`,
  `default_impl_foreign_prefix_path`, and `reopen_foreign_namespace` — each `use cgp_test_crate_a`, a
  sibling crate in the `cgp` workspace that supplies a *foreign* namespace and component so the
  orphan-rule violation (`E0210`/`E0117`) can arise. The harness cannot provide that crate, so the
  fixtures would fail with a bogus `E0432 unresolved import` and the intended error is never reached.
  Reproducing the orphan-rule class here would require teaching the harness to supply a foreign crate.
- **`inheritance_cycle`** — two namespaces that inherit from each other. Upstream, plain `rustc`
  rejects it eagerly with an `E0275` overflow; under cargo-cgp's next-gen solver it **compiles clean**,
  so there is no error to snapshot. This is a *missing* error, not a suppressed cause — the "reverse"
  of the next-solver compatibility caveat noted in
  [The error pipeline](../../../docs/implementation/error-pipeline.md#caveats).

## Keeping the imported mirror in sync

The imported `.rs` files are verbatim copies, so refreshing the mirror is a re-copy of the upstream
`acceptable/` fixtures into the matching kind directory followed by a re-bless of the snapshots
(`cargo test -p cargo-cgp-ui-tests --test ui -- --bless`). When upstream adds a fixture, add it here
under the kind that matches its error class unless it is cross-crate or a next-solver divergence, in
which case record it in the list above instead. The one edit made on import is disambiguating a name
collision: `duplicate_path_key` exists under two upstream constructs, so the copies are
`namespace_duplicate_path_key` and `delegate_duplicate_path_key` in [`wiring/`](wiring).
