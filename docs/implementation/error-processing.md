# Error processing

`cargo-cgp` turns a compiler's raw diagnostics into readable, root-cause-first CGP errors, and
*processing* is the stage that does the turning: a single stateless function takes the structured
diagnostics rustc produced and returns a smaller, reordered set of CGP diagnostics. This document
records that stage's interface, the types on either side of it, why it must be a stateful analysis
rather than a per-error rewrite, and how it is tested.

**Status: preprocessing built, aggregation not yet.** The stage exists as a crate,
[`cargo-cgp-error-processing`](../../crates/cargo-cgp-error-processing), wired end to end: the
front-end captures cargo's diagnostics, hands them to [`process_cgp_errors`], and re-emits the result.
Processing has two sub-stages — **preprocessing**, which transforms each diagnostic on its own, and
**aggregation**, which works across the whole set to collapse cascades. Preprocessing is implemented
(it strips CGP path prefixes and resugars `Symbol!`, so the output is already more readable than
rustc's); aggregation does not exist yet, so the output still has one entry per input. Where this
document describes lifting a root cause or dropping an echo, read that as the specified target of the
aggregation sub-stage, not current behavior.

## Where processing sits in the pipeline

Processing is the third of four stages, and it is the only one that needs neither the compiler nor
the filesystem. The [error pipeline](error-pipeline.md) runs: **configure rustc** (inject flags so
the compiler emits better raw diagnostics), **capture** (collect those diagnostics as structured,
serializable data), **process** (this stage — transform the captured diagnostics into CGP
diagnostics), and **render** (format the result as human-readable text or JSON). The configure stage
runs inside the compilation, in the driver; capture, process, and render all live in the front-end
today, which parses cargo's JSON output and re-emits it. Processing is deliberately isolated as a
pure function between capture and render, because that isolation is what makes it testable and
reorderable — the two properties this stage exists to deliver.

The boundary with capture is worth stating precisely, because it decides where each piece of logic
belongs. Anything that needs the compiler's live state — re-running trait fulfillment through the
`InferCtxt` / `ObligationCtxt` API to reconstruct an obligation chain, or reading an interned
`Symbol` type the printer elided — must happen during capture, and be folded into the captured data
before processing sees it. Processing itself receives only already-serialized diagnostics and has no
way to ask the compiler anything. This keeps the expensive, compiler-coupled work upstream and leaves
processing a self-contained transform over plain data.

## Two sub-stages: preprocess, then aggregate

Processing splits into two sub-stages that differ in what they are allowed to see, and keeping them
distinct is the single most important property of the design. **Preprocessing** transforms each
diagnostic on its own — cleaning up its type names, resugaring its encodings — and never looks at any
other diagnostic. **Aggregation** works across the whole set: it detects that one CGP mistake has
cascaded into many diagnostics, lifts the single root cause to the top, and drops or summarizes the
echoes. Preprocessing keeps the diagnostic count the same; aggregation is where it shrinks.

Because preprocessing is per-diagnostic, it is legitimately a `map` over the input, and that is
exactly how it is implemented (see [The preprocessing pipeline](#the-preprocessing-pipeline)).
Aggregation cannot be a map, and this is the distinction to hold onto: you cannot decide what to emit
for one diagnostic by looking at that diagnostic alone, because whether it is a root cause or an echo
of another is a fact about the *whole set*. Aggregation must therefore **ingest** every preprocessed
diagnostic into a queryable store and then **query** that store to synthesize the output, building
each result from a view across the set. The store is what lets a many-to-fewer, reordered transform
exist at all.

**A future change must not fold aggregation into the preprocessing map.** The processing entrypoint
today is a `map` because only preprocessing exists, and the tempting wrong turn is to "extend" it by
adding cascade-collapsing logic inside that `map` — which can never work, since each step is blind to
the others. Aggregation is a *second phase* that runs after the preprocessing map completes and sees
all its results at once; it is not more branches inside the per-diagnostic loop. The same warning is
stamped on the entrypoint's own doc comment.

## The interface

The stage is one plain function, stateless and free of side effects, in the
[`cargo-cgp-error-processing`](../../crates/cargo-cgp-error-processing) crate:

```rust
pub fn process_cgp_errors(rust_errors: Vec<cargo_metadata::diagnostic::Diagnostic>) -> Vec<CgpDiagnostic>;
```

It takes the diagnostics by value: each is wrapped into a `CgpDiagnostic` (a move, not a clone) and
run through the preprocessing pipeline. Statelessness is a hard requirement, not a stylistic
preference, because it is what lets the stage be tested without the rest of the tool. A pure function
over serializable input and serializable output can be driven from a unit test that reads a fixture
file and compares a snapshot, with no compiler, no cargo, and no `cargo-cgp` process in the loop (see
[Testing with snapshots](#testing-with-snapshots)). Any state the future aggregation sub-stage needs —
the internal store — lives *inside* one call and is gone when it returns; nothing persists between
calls and nothing is read from the environment.

The function lives in its own crate precisely so that neither building nor testing it pulls in
`rustc_private`. The driver crate links the compiler's unstable internals and can only be built on the
pinned nightly; keeping the processor in a separate, ordinary crate means the stage compiles and its
tests run on any toolchain. The front-end depends on this crate and calls into it; the crate never
depends on the front-end or the driver.

## The input type: `cargo_metadata::Diagnostic`

The input is [`cargo_metadata::diagnostic::Diagnostic`](../../../external/cargo_metadata/src/diagnostic.rs),
the public, `serde`-deserializable mirror of the exact JSON shape rustc emits under
`--message-format=json`: its `message`, `code`, `level`, `spans`, `children`, and `rendered` fields
line up field-for-field with the compiler's own wire format. Being deserializable is what lets a
fixture be read back from a text file, and being the established public mirror means the tool tracks a
stable type rather than a private compiler internal. `cargo_metadata` is a workspace dependency of
both this crate and the front-end.

The compiler's own diagnostic types were rejected for this role, and the reasoning is recorded in the
function's doc comment as well, because it is the first thing a later agent will reconsider. The
in-memory `rustc_errors::DiagInner` is `rustc_private`, is not serializable, carries untranslated
messages and unresolved `MultiSpan`s that need a `SourceMap` to become file-and-line, and has no
rendered form — none of which survives a trip to a fixture file. The structs rustc's `JsonEmitter`
serializes *are* the right shape, but they are module-private inside `rustc_errors`, derive
`Serialize` only (no `Deserialize`), and are not exported, so even a `rustc_private` build cannot name
or read them back. `cargo_metadata::Diagnostic` exists precisely to be the reusable public
counterpart, which is why it is the input type.

`DiagInner` is nonetheless the noted fallback. It carries richer, un-rendered structure — interned
messages, the raw argument map — that a future analysis might need and that `cargo_metadata` cannot
express. Reaching for it would mean capturing diagnostics live inside the driver through a custom
`Emitter`, moving `process_cgp_errors` into the `rustc_private` world and costing it its standalone
testability. We take the `cargo_metadata` route first for that reason, and reconsider `DiagInner` only
if it proves unable to carry enough.

## The output type: `CgpDiagnostic`

[`CgpDiagnostic`](../../crates/cargo-cgp-error-processing/src/diagnostic.rs) is a structural superset
of the diagnostic. It always carries the underlying rustc diagnostic in a `pub diagnostic:
cargo_metadata::diagnostic::Diagnostic` field, and grows CGP-specific structure alongside it. The
first such field exists today: `pub has_cgp_error: bool`, which a preprocessor sets to `true` once it
recognizes a CGP construct in the diagnostic (a CGP path prefix, a `Symbol!` spine, …), and which
defaults to `false`. It is the flag a later aggregation sub-stage will use to tell a CGP diagnostic
worth analyzing from a plain Rust one to leave alone. A diagnostic is created with
`CgpDiagnostic::wrap`, which wraps a raw diagnostic with `has_cgp_error` at its `false` default before
preprocessing runs.

The second CGP-specific field is `pub details: Vec<CgpDiagnosticDetail>`, the structured facts a
preprocessor extracts alongside rewriting the text. Where `has_cgp_error` is a yes/no flag, a detail
records *what* was understood as typed data — a [`CgpDiagnosticDetail`](../../crates/cargo-cgp-error-processing/src/diagnostic.rs)
enum, today with `MissingField { field_name, context }` and `MissingDeriveHasField { field_name,
context }` variants. The point of extracting the fact, not just rewriting the prose, is that the later
aggregation sub-stage (and eventual JSON output) can act on the structured detail — group diagnostics
by context, dedupe by field — without re-parsing rendered text.

The superset shape is what lets one output type serve two kinds of result. A **passed-through**
diagnostic — a non-CGP error, or a CGP error not yet handled — keeps `has_cgp_error` false and
`details` empty. A **recognized** CGP diagnostic has `has_cgp_error` set and its `details` populated,
and will carry more as analysis grows: a classified [catalog](../../../cgp/docs/errors/README.md)
class, decoded encodings, the link from a root cause to the cascade it explains. Rendering never has
to special-case the two, because both are the same type and both always carry the base diagnostic to
fall back on. (An enum split between a passthrough variant and a recognized variant was the considered
alternative; the superset struct was chosen because it keeps the base diagnostic unconditionally
present.)

`CgpDiagnostic` stays structured data, never a pre-rendered string, for the same reason rustc keeps
its diagnostics structured until the emitter runs. The output feeds the render stage, which must be
able to produce more than one form from it — human-readable text today, and the JSON that
`--message-format=json` consumers expect later. A diagnostic frozen into a text blob at the end of
processing could only ever be printed one way; keeping it structured defers the formatting choice to
the stage that owns it. `CgpDiagnostic::rendered` exposes the base diagnostic's rustc-rendered text so
the front-end can print it, and it is that `rendered` field the preprocessors rewrite.

## The preprocessing pipeline

Preprocessing is a chain of preprocessor functions, each with the shape
`fn(CgpDiagnostic) -> CgpDiagnostic`, applied in order so the output of one feeds the next. The chain
is a single list — [`preprocess::PREPROCESSORS`](../../crates/cargo-cgp-error-processing/src/preprocess/pipeline.rs) —
folded over the diagnostic, so a new preprocessor is added by adding it to the list. Each transforms
the diagnostic's human-readable text (its `message` and, crucially, its `rendered` field, since that
is what the tool prints) and sets `has_cgp_error` when it recognizes a CGP construct. Order matters:
prefix stripping runs first so the later stages match the bare CGP names rather than their
fully-qualified forms. Three preprocessors exist today:

- **[`strip_cgp_prefixes`](../../crates/cargo-cgp-error-processing/src/preprocess/strip_prefixes.rs)**
  removes the CGP module paths rustc prints in front of CGP type names — `cgp::prelude::Chars` becomes
  `Chars`. The prefixes it strips are a constant list, `CGP_PREFIXES` (`cgp::prelude::`,
  `cgp::macro_prelude::`, `cgp::cgp_core::`, `cgp::cgp_extra::`), kept as a list precisely so more
  re-export paths can be added as they turn up. A prefix is a reliable sign the diagnostic involves
  CGP, so removing one also sets `has_cgp_error`.
- **[`resugar_symbol`](../../crates/cargo-cgp-error-processing/src/preprocess/resugar_symbol.rs)**
  reverses a `Symbol!` expansion back to its surface form: `Symbol<2, Chars<'x', Chars<'y', Nil>>>`
  becomes `Symbol!("xy")`. It parses the spine and rewrites it **only on an exact structural match** —
  the declared length must equal the decoded string's byte length (the length `Symbol!` bakes in is
  `str::len()`, not the character count), the spine must be `Chars`/`Nil` all the way down, and each
  `Chars` head must be a single plain character literal. A `Symbol<…>` that does not match exactly is
  left untouched, because another type could share the name; this caution is essential to every
  resugaring preprocessor, not just this one. A successful resugar also sets `has_cgp_error`.
- **[`extract_missing_fields`](../../crates/cargo-cgp-error-processing/src/preprocess/missing_field.rs)**
  turns an unmet `HasField` bound into a field-oriented message and extracts a `CgpDiagnosticDetail`.
  It matches (after the two stages above)
  `` the trait `HasField<Symbol!("name")>` is not implemented for `Context` `` and distinguishes two
  cases whose fixes differ — a distinction available *within the one diagnostic*, so no cross-diagnostic
  aggregation is needed. When the context implements `HasField` for some other field, it is a single
  missing field (`` missing field `name` in `Context` ``, detail `MissingField`); when it implements
  `HasField` for no field at all, the whole derive is missing
  (`` `#[derive(HasField)]` is required to access field `name` in `Context` ``, detail
  `MissingDeriveHasField`). The tell is rustc's "similar impl" landmark, which the CGP
  [check-trait-failure catalog entry](../../../cgp/docs/errors/checks/check-trait-failure.md) documents:
  its presence — either inline (`but trait `HasField<…>` is implemented for it`, one other field) or as
  a separate `` `Context` implements trait `HasField<…>` `` note (several other fields) — means a single
  missing field; its absence means the missing derive. Either way it sets `has_cgp_error` and records
  the detail. It does *not* yet handle the sibling form rustc emits for a direct method call rather than
  a `check_components!` assertion (an `E0599` carrying CGP's own `#[diagnostic::on_unimplemented]` text
  instead of the `` `HasField<…>` is not implemented `` clause); that is a future preprocessor.

  **The empty-derived-struct case is fine, not a defect.** The single-vs-derive classification is
  exact except for one degenerate input, and that input needs no fix. A context that derives
  `HasField` but declares **no fields at all** gets the missing-derive message even though the derive
  is present — but that is correct, because `#[derive(HasField)]` emits one impl per field, so on a
  fieldless struct it emits *nothing*, identical to no derive at all. A fieldless derive leaves no
  trace in the generated program, so it is genuinely impossible to tell whether it was written; the
  two are the same program wherever `HasField` is concerned. `extract_missing_fields` reports
  `MissingDeriveHasField`, which accurately states what is observable — the context implements
  `HasField` for no field, and a fieldless derive is exactly that. There is nothing to recover (not
  from one diagnostic, not from the whole set, not from the expansion), because the two situations do
  not differ. The case is pinned by the
  [`checks/empty_field_struct`](../../tests/ui/usability/checks/empty_field_struct.rs) fixture so the
  behavior stays visible.

A non-CGP diagnostic runs through the pipeline untouched: no prefix matches, no `Symbol` spine parses,
no `HasField` clause matches, `has_cgp_error` stays false, and the diagnostic passes through unchanged.

One deduplication happens near this stage but is *not* part of it — it is a render-fidelity step in
the front-end. rustc's human emitter suppresses exact-duplicate diagnostics from its terminal output
(while still counting them), and capturing via `--message-format=json` loses that suppression because
the JSON stream repeats them. The front-end restores it when it re-emits rendered text (see [The error
pipeline](error-pipeline.md)). That is distinct from the cascade-collapsing deduplication aggregation
will do: this one drops byte-identical repeats to match rustc's own rendering; aggregation must
recognize *related* diagnostics that share a root cause and are not identical at all.

## Testing with snapshots

Because the processor is a pure function over serializable data, it is tested without running the
tool. A fixture is a text file holding serialized diagnostics — captured once from a real compilation
and committed — and a test reads it, calls `process_cgp_errors`, and asserts over the returned
`CgpDiagnostic` set. Nothing compiles, no driver runs, and the test is fast and deterministic, so a
whole catalog of error classes can be exercised as ordinary library tests. This is what the
statelessness requirement buys. Two test files cover the stage: `preprocess.rs` drives each
preprocessor over crafted diagnostics — a stripped prefix, an exactly-matched `Symbol!`, a wrong
length or foreign type left alone — and checks the rewritten text and the `has_cgp_error` flag; and
`passthrough.rs` confirms a non-CGP diagnostic comes through the pipeline untouched with
`has_cgp_error` false.

The [UI snapshot suite](testing.md) tests this stage a second way, over real captured diagnostics.
Each fixture is pinned by both a `.stderr` snapshot and a `.output.json` snapshot of the diagnostics
the tool captured, and one of the suite's three passes — the *process pass* — parses that
`.output.json`, runs it through `process_cgp_errors`, renders the result, and checks it reproduces the
`.stderr` the real binary produced. So `process_cgp_errors` is exercised over every fixture's actual
diagnostics, not just the hand-picked fixture in this crate's own tests, and the process pass runs
without invoking the compiler (`--process-only`), giving a sub-second loop for iterating on the
processing code. The two levels guard different seams: this crate's tests pin the transform on a
curated input, while the UI process pass proves it stays consistent with what the binary captures and
renders across the whole catalog.

## Planned processing work

The work still ahead falls into the two sub-stages. **More preprocessors** extend the per-diagnostic
pipeline — each a new `fn(CgpDiagnostic) -> CgpDiagnostic` added to `PREPROCESSORS`, each applying the
same exact-match caution `resugar_symbol` sets the precedent for:

- **Decode the remaining type-level encodings.** `Symbol!` is done; `Cons<A, Cons<B, Nil>>` is
  `Product![A, B]`, `Either<…>`/`Void` spines are `Sum![…]`, and so on. Rewriting these back to their
  surface form removes the rest of the visual noise a CGP error carries.
- **Recognize more error classes.** `extract_missing_fields` handles the missing-field and
  missing-derive classes and records a `CgpDiagnosticDetail`; the same shape extends to the other
  [catalog](../../../cgp/docs/errors/README.md) classes, each rewriting its message and adding a detail
  variant. The nearest next step is the sibling of the missing-field class reached through a direct
  method call — the `E0599` form carrying CGP's `#[diagnostic::on_unimplemented]` text rather than the
  `` `HasField<…>` is not implemented `` clause `extract_missing_fields` matches.

**The aggregation sub-stage** does not exist yet and is the larger piece:

- **Lift the root cause and collapse the cascade.** Detect the one deep mistake reported at every
  transitively dependent provider, present it first, and summarize or drop the repeats. This is what
  needs the ingest-then-query store, since finding the repetition is a query across the whole set —
  and it is what makes the output count finally differ from the input.

One related transformation is deliberately in neither sub-stage: recovering an obligation chain the
solver renders tersely by re-running fulfillment through the compiler's `InferCtxt`. That needs live
compiler state, so it belongs to the capture stage and enriches the diagnostic data before processing
runs; it is recorded in [The error pipeline](error-pipeline.md).

## Comparison with Clippy

Clippy offers no prior art for this stage, and the absence is itself informative. Clippy defines no
diagnostic type of its own: it reuses rustc's `Diag`/`DiagCtxt` machinery directly and emits through
the compiler's own emitters, so its human and JSON output is rustc's, unmodified
([`clippy_utils/src/diagnostics.rs`](../../../external/rust-clippy/clippy_utils/src/diagnostics.rs)).
It has no superset type and no passthrough concept because it never rewrites a diagnostic — it only
*adds* new ones (lints) alongside rustc's. `cargo-cgp` is doing the opposite: it *rewrites and
reorders* the diagnostics rustc already produced, which is exactly why it needs a superset output type
and a stateful, many-to-fewer transform that Clippy has no equivalent of. The `cargo_metadata`-based
design is the right precedent to follow precisely because Clippy is not.

## Tests

- [`crates/cargo-cgp-error-processing/tests/preprocess.rs`](../../crates/cargo-cgp-error-processing/tests/preprocess.rs) —
  drives each preprocessor over crafted diagnostics, asserting the rewritten text, the `has_cgp_error`
  flag, and the extracted `details`: the exact-match cases `resugar_symbol` must skip, and the
  single-field (inline and separate-note landmark) versus missing-derive branches of
  `extract_missing_fields`.
- [`crates/cargo-cgp-error-processing/tests/passthrough.rs`](../../crates/cargo-cgp-error-processing/tests/passthrough.rs) —
  drives `process_cgp_errors` over a committed serialized fixture
  ([`tests/fixtures/sample_diagnostics.json`](../../crates/cargo-cgp-error-processing/tests/fixtures/sample_diagnostics.json))
  of plain-Rust diagnostics, asserting they pass through the pipeline untouched with `has_cgp_error`
  false.
- The [UI snapshot suite](testing.md) exercises `process_cgp_errors` over every fixture's real
  captured diagnostics: its *process pass* parses each `<name>.output.json`, runs the function,
  renders the result, and checks it reproduces the binary's `<name>.stderr`. The `--process-only` mode
  runs just this pass, with no compilation, as the fast iteration loop.

## Source

- [`crates/cargo-cgp-error-processing/src/process.rs`](../../crates/cargo-cgp-error-processing/src/process.rs) —
  `process_cgp_errors`: wraps each diagnostic and runs the preprocessing pipeline, carrying the
  "aggregation is a separate phase, not more branches in the map" warning and the `DiagInner` fallback
  note.
- [`crates/cargo-cgp-error-processing/src/diagnostic.rs`](../../crates/cargo-cgp-error-processing/src/diagnostic.rs) —
  the `CgpDiagnostic` superset type (with `has_cgp_error`) and its `wrap`/`rendered` helpers.
- [`crates/cargo-cgp-error-processing/src/preprocess/`](../../crates/cargo-cgp-error-processing/src/preprocess) —
  the preprocessing pipeline: `pipeline.rs` (the `PREPROCESSORS` list and fold), `strip_prefixes.rs`
  (`strip_cgp_prefixes` and the `CGP_PREFIXES` constant), `resugar_symbol.rs` (the exact-match
  `Symbol!` parser), `missing_field.rs` (`extract_missing_fields` and the single-field-vs-missing-derive
  classification), and `text.rs` (applying a transform across a diagnostic's text fields).
- [`crates/cargo-cgp/src/check/diagnostics.rs`](../../crates/cargo-cgp/src/check/diagnostics.rs) — the
  front-end's capture and render around this stage: parsing cargo's JSON stream into diagnostics, and
  re-emitting the processed result (with the render-fidelity deduplication).

## Further reading

- [The error pipeline](error-pipeline.md) — the surrounding stages: how rustc is configured to emit
  better diagnostics, how they are captured, and how the processed result is rendered.
- [rustc diagnostic internals](rustc-diagnostic-internals.md) — where the compiler elides the
  information this stage would otherwise have to reconstruct, and the `InferCtxt` enrichment path that
  runs during capture.
- [CGP error catalog](../../../cgp/docs/errors/README.md) — the error classes the processor must
  learn to recognize, and where each class hides or surfaces its root cause.
