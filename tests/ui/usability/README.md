# Usability fixtures

These are the fixtures for the [usability](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md) category: CGP compile
errors that carry their root cause but bury it in volume, duplication, encoding, or misleading
framing, so the work is re-presentation rather than recovery. Every fixture here is one whose cause
cargo-cgp's transformed output (`-Znext-solver=globally` + `--verbose`) does contain — a fixture whose
cause were *absent* would belong under
[hidden-root-cause](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/hidden-root-cause.md) instead. A fixture whose presentation
improves enough to clear the bar graduates out of here into [`../acceptable/`](../acceptable); the
whole check-trait-failure family has already done so, followed by the missing-derive coalescing, the
dispatch-chain elision, the method-advice cleanup, the abstract-type mismatch, the consumer
coalescing, most of the wiring-conflict reshaping, and the naming of an aggregate provider's own
`open` table.

The fixtures are grouped by the *kind* of remaining usability problem, one sub-directory per issue
class in [the usability issue document](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md):

- [`lowering/`](lowering) — a macro lowered accepted input into ill-formed Rust, and the error lands
  on the macro attribute without naming the real cause: an unsized generated type (`option_slice`).
  A cyclic `#[use_type]` routing used to sit here too and was resolved upstream instead — the cycle
  is decidable from the import list, so the macro now rejects it and nothing reaches the compiler for
  the tool to reshape.
  It also pins two derive defects: an enum variant whose name collides with an associated type the
  data derives generate (`Value`, `Remainder`, `Fields`, …), and a variant helper attribute that the
  extractor's companion enums copy without the derive that owns it.
- [`extensible-data/`](extensible-data) — a cast, builder, or extractor failure the resolver does
  not reshape at all (`upcast_missing_variant`), so the reader gets an internal `FromVariant` bound
  and the macro-generated extractor state instead of the variant one enum lacks. It also pins the
  post-processing that survives a decline: rustc's "similar impl" hint splits into styled fragments
  at every difference, and the fragments are read as one line so a shredded `Symbol!` still
  resugars. The other fixtures pin the rest of the family as rustc reports it: the record builder
  (a field built twice, taken while absent, or unknown, and a premature finalize), merging records
  with `build_from`, the defaulted and optional builders, extraction and construction by tag, the
  casts, the `BuildAndMergeOutputs` builder dispatcher, and a shape bound for every lifetime. None of
  them names the field or variant at fault.
- [`misleading/`](misleading) — a rewrite that states something false: a wrong context, table, or
  inner provider named, a derive or wiring fix that does not apply, a cascade reported as a cycle,
  or a root cause built from an unresolved type. Each fixture's header names the false line.
- [`fields/`](fields), [`providers/`](providers), [`types/`](types), [`handlers/`](handlers), and
  [`use-site/`](use-site) — failures of those constructs that the tool passes through or reshapes
  without the hint that names the fix, such as a getter wired to the wrong provider or a pipeline
  stage whose types disagree.
- [`paths/`](paths) — an untransformed diagnostic that prints full paths (`cgp::prelude::IsNothing`,
  `std::result::Result`) where plain `cargo check` prints short names, including inside a
  machine-applicable suggestion.
- [`wiring/constraints/`](wiring/constraints) — an unconstrained per-entry generic
  (`unconstrained_generic`), whose `E0207` fires twice with contradictory auto-fixes, and the
  `for`-loop and `#[default_impl]` forms of the same `E0207`.
- [`wiring/checks/`](wiring/checks), [`wiring/conflicts/`](wiring/conflicts),
  [`wiring/cycles/`](wiring/cycles), [`wiring/keys/`](wiring/keys), and
  [`wiring/namespaces/`](wiring/namespaces) — wiring mistakes left uncoded or unhinted: a check
  table defined twice, a hand impl beside wiring, a cycle reached by a call, a key or value that
  does not resolve, and a near-miss namespace path. The other
  structural conflicts have been reshaped and graduated: the duplicate delegate-key family into
  `[CGP-E004]`–`[CGP-E008]`, the duplicate `cgp_namespace!` path into `[CGP-E008]`, the duplicate
  provider name's redundant `IsProviderFor` half suppressed, and the `UseContext` cycle into
  `[CGP-E010]` (all under [`../acceptable/wiring/`](../acceptable/wiring)).

## Origins

Each sub-directory mixes hand-curated fixtures with cases migrated from `cgp`'s former compile-fail
suite (since removed and now maintained here). The full account of the two origins, the cross-crate
cases reproduced through auxiliary crates, and the one class with no snapshot lives in the
[top-level tests README](../../README.md); most of the migrated cases now sit under
[`../acceptable/`](../acceptable), since every reproducible class carries its cause and most are
presented well. When a fixture here is fixed, delete its issue from
[cgp-knowledge-base/cargo-cgp/issues/usability.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md) and move its
`.rs`/`.cgp.stderr`/`.rust.stderr`/`.expand.rs` set into the matching `../acceptable/` concept
sub-directory (no re-bless is needed — a snapshot is independent of the fixture's directory).
