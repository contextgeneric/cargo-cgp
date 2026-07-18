# Usability fixtures

These are the fixtures for the [usability](../../../docs/issues/usability.md) category: CGP compile
errors that carry their root cause but bury it in volume, duplication, encoding, or misleading
framing, so the work is re-presentation rather than recovery. Every fixture here is one whose cause
cargo-cgp's transformed output (`-Znext-solver=globally` + `--verbose`) does contain — a fixture whose
cause were *absent* would belong under
[hidden-root-cause](../../../docs/issues/hidden-root-cause.md) instead. A fixture whose presentation
improves enough to clear the bar graduates out of here into [`../acceptable/`](../acceptable); the
whole check-trait-failure family has already done so.

The fixtures are grouped by the *kind* of remaining usability problem, one sub-directory per issue
class in [the usability issue document](../../../docs/issues/usability.md):

- [`duplication/`](duplication) — one mistake reported as many errors: a single cause that fans out
  across several top-level error blocks (`density_3`, `dependency_cascade`), or a single missing
  `#[derive(HasField)]` reported field by field (`base_area_2`).
- [`lowering/`](lowering) — a macro lowered accepted input into ill-formed Rust, and the error lands
  on the macro attribute without naming the real cause: an unsized generated type (`option_slice`) or
  a cyclic `#[use_type]` routing (`use_type_cyclic_context`).
- [`use-site/`](use-site) — a use-site failure the resolver cannot anchor:
  `generic_consumer_unwritten_arg` calls a *local* generic consumer whose dispatch parameter rides in
  a plain variable argument — an argument the call does not type syntactically, so no anchor can read
  it — and its `E0599` keeps rustc's misleading method-syntax advice ahead of the buried cause. (The
  namespace-joined `E0599`, the `#[use_type]` shapes, the `Code`-dispatched `E0277`, and the
  written-value-argument case that used to live here now resolve and have moved to
  [`../acceptable/`](../acceptable).)
- [`verbosity/`](verbosity) — output that is *correct but overwhelming*: `deep_dispatch_chain` is a
  resolved `Code`-dispatched failure whose dependency chain restates the full program type at every
  `Handler` node and renders every redirect hop, so a DSL-sized program yields a tree of dozens of
  near-identical lines around one short root cause.
- [`wiring/`](wiring) — the structural coherence conflicts (`E0119`/`E0207`/`E0275`) that still pass
  through with only light post-processing. The duplicate delegate-key `E0119` is now reshaped into a
  coded `[CGP-E004]`–`[CGP-E008]` headline and has moved to [`../acceptable/wiring`](../acceptable/wiring); what remains
  here is `duplicate-keys/` (a duplicate provider *name*, whose `Greeter`/`IsProviderFor` pair still
  fans out — not a `DelegateComponent` conflict), `namespace-paths/` (a duplicate `cgp_namespace!`
  `@`-path, a single `E0119` on the user's own namespace trait), and `constraints/` (an unconstrained
  per-entry generic and a `UseContext` wiring cycle).

## Origins and the imported mirror

Each sub-directory mixes hand-curated fixtures with verbatim copies of the upstream CGP compile-fail
suite. The full account of the two origins, the four upstream fixtures deliberately not imported, and
the re-sync workflow lives in the [top-level tests README](../../README.md); most of the imported
mirror now sits under [`../acceptable/`](../acceptable), since every reproducible class carries its
cause and most are presented well. When a fixture here is fixed, delete its issue from
[docs/issues/usability.md](../../../docs/issues/usability.md) and move its
`.rs`/`.cgp.stderr`/`.rust.stderr` triple into the matching `../acceptable/` concept sub-directory (no
re-bless is needed — a snapshot is independent of the fixture's directory).
