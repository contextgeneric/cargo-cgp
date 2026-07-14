# Error processing

`cargo-cgp` turns a compiler's raw diagnostics into readable, root-cause-first CGP errors, and the
string-level logic that does the turning lives in one rustc-free crate,
[`cargo-cgp-error-processing`](../../crates/cargo-cgp-error-processing). This document records what
that crate holds, why it is kept apart from the driver, how the driver drives it, and how it is
tested.

The crate is **driven entirely by the driver**; the front-end no longer touches diagnostics. It
exists as a separate crate for one reason: it links no compiler internals, so it builds and its
tests run on any toolchain, without the driver's `rustc_private` linkage. The driver depends on it
and calls into it from its emitter; the crate never depends on the driver or the front-end. Keeping
the logic here is what lets a plain unit test exercise a transform over a hand-built string, with no
compiler, no cargo, and no `cargo-cgp` process in the loop.

## What the crate holds

The crate has four tenants, all driven by the driver's emitter. Grouping them here, apart from the
driver, is what keeps them unit-testable.

- **Post-processing** ([`postprocess`](../../crates/cargo-cgp-error-processing/src/postprocess)) is
  the set of fallback text transforms the driver applies to a diagnostic's messages so raw CGP
  constructs do not look confusing. Each is a pure `&str -> Option<String>` — `Some` when it changed
  the text — and
  [`postprocess_message`](../../crates/cargo-cgp-error-processing/src/postprocess/chain.rs) chains
  them.
- **The wiring rewrite** ([`rewrite`](../../crates/cargo-cgp-error-processing/src/rewrite)) is the
  string transform that renames CGP wiring messages, over the
  [`ComponentNameMap`](../../crates/cargo-cgp-error-processing/src/rewrite/names.rs) the driver fills
  in from the compiler. It is documented where it is used, in
  [The driver](driver.md#naming-the-traits-behind-a-component-marker).
- **The diagnosis model and its wording**
  ([`diagnosis`](../../crates/cargo-cgp-error-processing/src/diagnosis)) is the rustc-free root-cause
  model — the `Resolved` failure the driver's typed resolver produces, in owned `String` form — and
  the wording that turns it into diagnostic text. `plan_resolved` composes the rewritten header, the
  derive `help`s, and the per-cause `root cause:` notes into a `DiagnosisPlan`, which the emitter only
  maps onto rustc's `DiagInner`; keeping every piece rustc-free is what makes the whole
  diagnosis-to-text layer unit-testable without a `TyCtxt`. It is documented in
  [Typed root-cause resolution](typed-root-cause-resolution.md).
- **The dependency-tree renderer**
  ([`tree`](../../crates/cargo-cgp-error-processing/src/tree.rs)) is the `DependencyTree` type and its
  `cargo tree`-style renderer, over the tiny `termtree` crate, that the driver's resolver builds and
  the diagnosis wording renders to show a check failure's transitive dependency chain. It is
  documented in [Typed root-cause resolution](typed-root-cause-resolution.md).

A fifth module, [`code`](../../crates/cargo-cgp-error-processing/src/code.rs), holds the `CGP-E`
error-code constants the rewrite and the diagnosis wording stamp on classified main messages
(catalogued in [error-code.md](../error-code.md)). This document covers the post-processing transforms
in full and points at the other tenants' own documents.

## Where post-processing sits in the pipeline

Post-processing is the driver's final diagnostic pass, and it runs on **every** diagnostic the
driver emits, after whatever transform came before it (see [The error pipeline](error-pipeline.md)).
The driver's emitter first tries the [typed root-cause resolver](typed-root-cause-resolution.md); if
that declines, it renames the CGP wiring messages it recognizes; then, either way, the diagnostic
passes through the post-processing transforms.

The pass does two jobs depending on what came before it, and both keep raw CGP spellings out of the
output. For a diagnostic the tool left **un-rewritten**, post-processing is the whole cleanup — it
strips the `cgp::` prefixes, resugars the `Symbol!` and `Path!` spines, and rewords an unmet
`HasField` bound, so a diagnostic the tool does not classify still reads cleanly. For a **rewritten**
one, only the prefix strip and the `Symbol!`/`Path!` resugaring bite: they tidy the compiler-formatted
CGP type names a rewrite embeds — a provider like `RedirectLookup<…, PathCons<Symbol<…>>>` in a coded
header, folded to `RedirectLookup<…, Path!(@…)>`, say — while the missing-field reword finds nothing
to match, because the resolver's tree never carries the `` `HasField<…>` is not implemented `` clause
the reword keys on.

## How the driver applies the transforms

The driver applies post-processing over a `DiagInner` before its inner emitter renders it, so the
rendered output reflects the change. Each transform is a text function, and the driver runs
[`postprocess_message`] over every plain-string message and span label of the diagnostic and its
children — the same reach the compiler's renderer has, so the caret label, the notes, and the header
are all covered. Editing the structured `DiagInner` rather than a rendered blob means the transform
never touches the source snippets the emitter pulls from the `SourceMap`, so it cannot corrupt a line
of the user's own code the way a whole-text rewrite could.

One transform needs a fact about the whole diagnostic rather than one message, and the driver
computes it once up front. The missing-field reword must know whether the context implements
`HasField` for *any* field, because that decides between two wordings whose fixes differ, and the
"similar impl" landmark that tells it can sit in a different child than the clause. So the driver
scans every message and label with
[`context_has_hasfield_impls`](../../crates/cargo-cgp-error-processing/src/postprocess/missing_field.rs)
first, then passes the answer into each per-message rewrite.

## The post-processing transforms

Post-processing is a chain of transforms applied in order, so the output of one feeds the next, and
the order matters. Prefix stripping runs first so the later transforms match the bare CGP names
(`Symbol`, `Chars`, …) rather than their fully-qualified forms; `Symbol!` resugaring runs before
`Path!` resugaring (which reads the already-resugared `Symbol!("…")` segments) and before the field
rewrite (which matches the resugared `HasField<Symbol!("…")>` form). Four transforms exist today:

- **[`strip_cgp_prefixes`](../../crates/cargo-cgp-error-processing/src/postprocess/strip_prefixes.rs)**
  removes the CGP module paths rustc prints in front of CGP type names — `cgp::prelude::Chars` becomes
  `Chars`. The prefixes it strips are a constant list, `CGP_PREFIXES` (`cgp::prelude::`,
  `cgp::macro_prelude::`, `cgp::cgp_core::`, `cgp::cgp_extra::`), kept as a list precisely so more
  re-export paths can be added as they turn up.
- **[`resugar_symbol`](../../crates/cargo-cgp-error-processing/src/postprocess/resugar_symbol.rs)**
  reverses a `Symbol!` expansion back to its surface form: `Symbol<2, Chars<'x', Chars<'y', Nil>>>`
  becomes `Symbol!("xy")`. It parses the spine and rewrites it **only on an exact structural match** —
  the declared length must equal the decoded string's byte length (the length `Symbol!` bakes in is
  `str::len()`, not the character count), the spine must be `Chars`/`Nil` all the way down, and each
  `Chars` head must be a single plain character literal. A `Symbol<…>` that does not match exactly is
  left untouched, because another type could share the name; this caution is essential to every
  resugaring transform, not just this one.
- **[`resugar_path`](../../crates/cargo-cgp-error-processing/src/postprocess/resugar_path.rs)**
  reverses a `Path!` expansion back to its surface form:
  `PathCons<Symbol!("app"), PathCons<GreeterComponent, Nil>>` becomes `Path!(@app.GreeterComponent)`.
  It walks the right-nested `PathCons`/`Nil` spine and rewrites it **only on an exact,
  round-trippable match**, mirroring how `Path!` classifies each segment forward: a `Symbol!("name")`
  head becomes the bare segment `name` only when `name` is a lowercase, non-primitive identifier
  `Path!` would encode as a `Symbol`, and a named-type head is kept only when it is a plain identifier
  `Path!` would leave as a type — capitalized or a primitive. A spine that is not `PathCons`/`Nil` all
  the way down, or that carries a segment that would not round-trip, is left untouched. One spine that
  is *not* `Nil`-terminated is still rewritten: an **open-ended path**, whose spine ends in a generic
  "rest of path" parameter that rustc prints as the placeholder `_` (as in the conflicting-wiring
  `E0119` blocks over a duplicated `@`-path key). Such a tail resugars to a trailing `.*` wildcard
  segment, so `PathCons<Symbol!("foo"), PathCons<Symbol!("bar"), _>>` becomes `Path!(@foo.bar.*)`. The
  `.*` is not real `Path!` syntax and would not parse back, but it reads far better than the raw spine;
  only a bare `_` tail triggers it, since `_` is never a concrete segment, and any other non-`Nil` tail
  still declines. It runs after `resugar_symbol`, so its symbol segments are already in `Symbol!("…")`
  form.
- **[`rewrite_missing_fields`](../../crates/cargo-cgp-error-processing/src/postprocess/missing_field.rs)**
  turns an unmet `HasField` bound into a field-oriented message. It matches (after the two transforms
  above) `` the trait `HasField<Symbol!("name")>` is not implemented for `Context` `` and distinguishes
  two cases whose fixes differ. When the context implements `HasField` for some other field, it is a
  single missing field (`` missing field `name` on `Context` ``); when it implements `HasField` for no
  field at all, the whole derive is missing
  (`` `#[derive(HasField)]` is required to access field `name` on `Context` ``). Neither rewrite
  carries a CGP error code: the clause it rewrites sits in a sub-message, and codes classify only a
  rewritten *main* message (see [error-code.md](../error-code.md)). The tell that separates the two
  cases is rustc's "similar impl" landmark, which the CGP
  [check-trait-failure catalog entry](../../../cgp/docs/errors/checks/check-trait-failure.md) documents:
  its presence — either inline (`but trait `HasField<…>` is implemented for it`, one other field) or as
  a separate `` `Context` implements trait `HasField<…>` `` note (several other fields) — means a single
  missing field; its absence means the missing derive.

  **The empty-derived-struct case is fine, not a defect.** The single-vs-derive classification is
  exact except for one degenerate input, and that input needs no fix. A context that derives
  `HasField` but declares **no fields at all** gets the missing-derive message even though the derive
  is present — but that is correct, because `#[derive(HasField)]` emits one impl per field, so on a
  fieldless struct it emits *nothing*, identical to no derive at all. A fieldless derive leaves no
  trace in the generated program, so it is genuinely impossible to tell whether it was written; the
  two are the same program wherever `HasField` is concerned.

In practice the missing-field reword rarely fires today, because the [typed root-cause
resolver](typed-root-cause-resolution.md) recovers most missing-field failures from the compiler and
replaces them with a dependency tree before this fallback is reached. The transform remains for the
diagnostics the resolver declines, where the text clause is all there is to work with.

## Testing with pure inputs

Because each transform is a pure function over a string, it is tested without running the tool. A
test drives one transform over the case under test and asserts on the returned `Option<String>`;
nothing compiles, no driver runs, and the test is fast and deterministic, so a whole catalog of
cases can be exercised as ordinary library tests. This is what the rustc-free design buys.
[`tests/postprocess.rs`](../../crates/cargo-cgp-error-processing/tests/postprocess.rs) drives each
transform — a stripped prefix, an exactly-matched `Symbol!`, a wrong length or foreign type left
alone, the single-field (inline and separate-note landmark) versus missing-derive branches, and the
`postprocess_message` chain end to end. The diagnosis wording is tested the same way, only over a
hand-built `Resolved` rather than a string:
[`tests/diagnosis.rs`](../../crates/cargo-cgp-error-processing/tests/diagnosis.rs) drives
`plan_resolved` through the missing-field, missing-derive, `Deref`-target, field-type-mismatch,
use-site, kept-ordinary-bound, and provider-header cases, asserting the header, `help`s, and notes it
returns with no compiler in the loop.

The [UI snapshot suite](testing.md) exercises the transforms a second way, over real diagnostics:
every fixture's `.cgp.stderr` is what the driver rendered after applying them, so a change to a
transform that affects an emitted diagnostic shows up as a snapshot diff. The two levels guard
different seams — this crate's tests pin a transform on a curated input, while the UI suite proves
the transforms stay consistent with what the driver emits across the whole catalog.

## Planned work

The transforms still ahead extend the per-diagnostic cleanup, each a new function added to the
post-processing chain, each applying the same exact-match caution `resugar_symbol` sets the precedent
for. Decoding the remaining type-level encodings is the nearest: `Symbol!` and `Path!` are done;
`Cons<A, Cons<B, Nil>>` is `Product![A, B]`, `Either<…>`/`Void` spines are `Sum![…]`, and so on —
rewriting these back to their surface form removes the rest of the visual noise a CGP error carries.
Recognizing more error classes is the other direction, each rewriting its message the way the
missing-field transform does.

One larger transformation is deliberately out of scope for this crate: collapsing a *cascade* — the
one deep mistake reported at every transitively dependent provider — into a single root cause. That
is a cross-diagnostic transform, and it can only be decided by looking at the whole diagnostic set,
which this crate's per-string transforms never see. It would have to live in the driver's emitter,
which would need to buffer the compilation's diagnostics before emitting them; it does not exist yet.

## Comparison with Clippy

Clippy offers no prior art for this crate, and the absence is itself informative. Clippy defines no
diagnostic type of its own and never rewrites a diagnostic: it reuses rustc's `Diag`/`DiagCtxt`
machinery and emits its lints through the compiler's own emitters, so its output is rustc's,
unmodified ([`clippy_utils/src/diagnostics.rs`](../../../external/rust-clippy/clippy_utils/src/diagnostics.rs)).
`cargo-cgp` does the opposite — it *rewrites* the diagnostics rustc already produced — which is why it
needs a body of string transforms Clippy has no equivalent of. Keeping those transforms in a
rustc-free crate is the design that makes them testable without the compiler, precisely because
Clippy's "just use the compiler's emitter" approach is not open to a tool that rewrites.

## Tests

- [`crates/cargo-cgp-error-processing/tests/postprocess.rs`](../../crates/cargo-cgp-error-processing/tests/postprocess.rs) —
  drives each post-processing transform over crafted inputs: the stripped/left-alone prefix cases, the
  exact-match cases `resugar_symbol` must skip, the `PathCons` → `Path!` resugaring (symbol, type, and
  primitive segments, the open `_` tail folded to a `.*` wildcard, and the non-round-trippable cases
  `resugar_path` must skip), the single-field (inline and separate-note landmark) versus missing-derive
  branches of `rewrite_missing_fields`, and the `postprocess_message` chain.
- [`crates/cargo-cgp-error-processing/tests/rewrite.rs`](../../crates/cargo-cgp-error-processing/tests/rewrite.rs) —
  the wiring-message rewrite over a hand-built name map (see [The driver](driver.md#tests)).
- [`crates/cargo-cgp-error-processing/tests/diagnosis.rs`](../../crates/cargo-cgp-error-processing/tests/diagnosis.rs) —
  `plan_resolved` and the wording over hand-built `Resolved` values: the missing-field, missing-derive,
  and `Deref`-target field cases, a field-type mismatch, a use-site method failure, a kept ordinary
  bound (header dropped, lead-less note), a provider header via the text rewrite, and the pluralized
  consumer header.
- [`crates/cargo-cgp-error-processing/tests/tree.rs`](../../crates/cargo-cgp-error-processing/tests/tree.rs) —
  the dependency-tree renderer (see [Typed root-cause resolution](typed-root-cause-resolution.md#tests)).
- The [UI snapshot suite](testing.md) exercises the transforms over every fixture's real diagnostics:
  each `.cgp.stderr` is what the driver rendered after applying them.

## Source

- [`crates/cargo-cgp-error-processing/src/postprocess/`](../../crates/cargo-cgp-error-processing/src/postprocess) —
  the post-processing transforms: `mod.rs` (re-exports), `chain.rs` (the `postprocess_message` chain),
  `strip_prefixes.rs` (`strip_cgp_prefixes` and the `CGP_PREFIXES` constant), `resugar_symbol.rs` (the
  exact-match `Symbol!` parser), `resugar_path.rs` (the `PathCons` → `Path!` resugarer), and
  `missing_field.rs` (`rewrite_missing_fields`, `context_has_hasfield_impls`, and the
  single-field-vs-missing-derive classification).
- [`crates/cargo-cgp-error-processing/src/rewrite/`](../../crates/cargo-cgp-error-processing/src/rewrite) —
  the wiring-message rewrite and `ComponentNameMap` the driver drives: `mod.rs` (re-exports),
  `message.rs` (`rewrite_message` and the note/header forms, including the code-stamping
  `rewrite_trait_bound`), `names.rs` (`ComponentNameMap`/`ComponentTraitNames`), `parse.rs`
  (`parse_trait_bound`), and `text.rs` (the segment/generics splitters). See
  [The driver](driver.md#naming-the-traits-behind-a-component-marker).
- [`crates/cargo-cgp-error-processing/src/diagnosis/`](../../crates/cargo-cgp-error-processing/src/diagnosis) —
  the rustc-free root-cause model and its wording: `leaf.rs` (`Leaf`/`FieldIssue`), `resolved.rs`
  (`Cause`/`Resolved`), `wording.rs` (the `Resolved`→`String` builders — `consumer_header`,
  `field_mismatch_header`, `cause_note`, `derive_help_messages`), and `plan.rs` (`DiagKind`,
  `DiagnosisPlan`, and `plan_resolved` with its `categorized_header`). See
  [Typed root-cause resolution](typed-root-cause-resolution.md).
- [`crates/cargo-cgp-error-processing/src/tree.rs`](../../crates/cargo-cgp-error-processing/src/tree.rs) —
  the `DependencyTree` type and its `cargo tree`-style renderer.
- [`crates/cargo-cgp-error-processing/src/code.rs`](../../crates/cargo-cgp-error-processing/src/code.rs) —
  the `CGP-E` error-code constants, catalogued in [error-code.md](../error-code.md).
- [`crates/cargo-cgp-driver/src/emitter/`](../../crates/cargo-cgp-driver/src/emitter) — the
  driver-side caller: applies the transforms over a `DiagInner`'s messages and span labels after its
  own rewrite.

## Further reading

- [The error pipeline](error-pipeline.md) — the surrounding stages: how rustc is configured to emit
  better diagnostics, how each diagnostic is transformed, and how the result is rendered.
- [The driver](driver.md) — the emitter that drives these transforms, and the wiring rewrite in full.
- [CGP error catalog](../../../cgp/docs/errors/README.md) — the error classes the transforms must
  learn to recognize, and where each class hides or surfaces its root cause.
