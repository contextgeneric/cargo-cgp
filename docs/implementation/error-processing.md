# Error processing

`cargo-cgp` turns a compiler's raw diagnostics into readable, root-cause-first CGP errors, and
*processing* is the stage that does the turning: a single stateless function takes the structured
diagnostics rustc produced and returns a smaller, reordered set of CGP diagnostics. This document
records that stage's interface, the types on either side of it, why it must be a stateful analysis
rather than a per-error rewrite, and how it is tested.

**Status: scaffolding built, analysis not yet.** The stage exists as a crate,
[`cargo-cgp-error-processing`](../../crates/cargo-cgp-error-processing), and is wired end to end: the
front-end captures cargo's diagnostics, hands them to [`process_cgp_errors`], and re-emits the
result, so the pipeline runs through the new interface today. But `process_cgp_errors` is still a
**pass-through** — it does no CGP analysis yet, so the output matches rustc's own diagnostics. What
follows describes both the built scaffold and the design the real analysis must grow into; where it
says "the processor lifts the root cause," read that as the specified target, not current behavior.

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

## Why processing is not a one-to-one map

The processor must be free to return a different number of diagnostics than it received, and this is
the single most important property of its design. A CGP mistake rarely produces one error. One
missing field or one unsatisfied dependency cascades across the generated types the programmer never
wrote, so the compiler reports the same root cause at every transitively dependent provider, often
burying or eliding the cause itself along the way — the failure classes the [CGP error
catalog](../../../cgp/docs/errors/README.md) maps in full. The whole point of the stage is to
collapse that cascade: recognize the repetition, present the one root cause first, and drop or
summarize the echoes. Input count and output count therefore differ by design, and usually the
output is smaller.

That requirement rules out the shape the code most wants to fall into — a per-error rewrite. You
cannot decide what to emit for one diagnostic by looking at that diagnostic alone, because whether it
is a root cause or an echo of another one is a fact about the *whole set*. The processor must
therefore work in two phases. It first **ingests** every raw diagnostic into an internal store — a
queryable model of what the compilation reported, indexed so related diagnostics can be found. It
then **queries** that store to synthesize the output, building each CGP diagnostic from a view across
the ingested set rather than from a single input. The store is the mechanism that lets a
many-to-fewer, reordered transform exist at all.

**A future change must not collapse this into a naive loop.** The current placeholder walks the input
once, and the most likely wrong turn is to "extend" it by adding rewrite logic inside that walk —
turning the stage into `rust_errors.iter().map(...).collect()` with per-error branches. That shape
can never deduplicate a cascade or lift a cause above the errors that follow it, because each step is
blind to the others. The two-phase ingest-then-query structure is the design, not an optimization to
be added later; the placeholder is a stand-in for the query phase, not a skeleton to be fleshed out
in place. The same warning is stamped on the function's own doc comment.

## The interface

The stage is one plain function, stateless and free of side effects, in the
[`cargo-cgp-error-processing`](../../crates/cargo-cgp-error-processing) crate:

```rust
pub fn process_cgp_errors(rust_errors: &[cargo_metadata::diagnostic::Diagnostic]) -> Vec<CgpDiagnostic>;
```

Statelessness is a hard requirement, not a stylistic preference, because it is what lets the stage be
tested without the rest of the tool. A pure function over serializable input and serializable output
can be driven from a unit test that reads a fixture file and compares a snapshot, with no compiler,
no cargo, and no `cargo-cgp` process in the loop (see [Testing with snapshots](#testing-with-snapshots)).
Any state the analysis needs — the internal store — lives *inside* one call and is gone when it
returns; nothing persists between calls and nothing is read from the environment.

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
cargo_metadata::diagnostic::Diagnostic` field, and will grow optional CGP-specific structure
alongside it. The superset shape is what lets one output type serve two kinds of result. A
**passed-through** diagnostic — a non-CGP error, or a CGP error the processor does not yet handle — is
a `CgpDiagnostic` that carries the original and leaves the extra fields empty; the placeholder builds
every output this way, through `CgpDiagnostic::passthrough`. A **synthesized** CGP diagnostic will
fill those extra fields with what the analysis recovered: a classified
[catalog](../../../cgp/docs/errors/README.md) class, decoded type-level encodings, the link from a
root cause to the cascade it explains. Rendering never has to special-case the two, because both are
the same type and both always carry the base diagnostic to fall back on. (An enum split between a
passthrough variant and a synthesized variant was the considered alternative; the superset struct was
chosen because it keeps the base diagnostic unconditionally present.)

`CgpDiagnostic` stays structured data, never a pre-rendered string, for the same reason rustc keeps
its diagnostics structured until the emitter runs. The output feeds the render stage, which must be
able to produce more than one form from it — human-readable text today, and the JSON that
`--message-format=json` consumers expect later. A diagnostic frozen into a text blob at the end of
processing could only ever be printed one way; keeping it structured defers the formatting choice to
the stage that owns it. For today's pass-through render, `CgpDiagnostic::rendered` exposes the base
diagnostic's rustc-rendered text so the front-end can reproduce rustc's own output.

## The initial passthrough placeholder

`process_cgp_errors` currently does no CGP analysis: it treats every input as a non-CGP error and
returns each one as a passed-through `CgpDiagnostic`. Because `CgpDiagnostic` is a superset of the
diagnostic, this is well-typed and correct — it reproduces rustc's diagnostics, unchanged, through the
new interface, so the pipeline is wired end to end before any transformation logic exists. It is a
scaffold that proves the types and the plumbing, not the stage doing its job.

The placeholder carries a warning in its own doc comment, and the warning is the reason this section
exists: the placeholder happens to walk the input one-to-one, and that shape must not become the
design. A later agent replacing it must build the ingest-then-query structure described above, not add
rewrite branches inside the passthrough walk. The distinction is subtle precisely because the
placeholder looks like the start of a per-error loop, so the comment says plainly that it is a
stand-in for the query phase and that the real processor reads the whole set before it emits anything.

One deduplication already happens, but *not* here — it is a render-fidelity step in the front-end, not
processing. rustc's human emitter suppresses exact-duplicate diagnostics from its terminal output
(while still counting them), and capturing via `--message-format=json` loses that suppression because
the JSON stream repeats them. The front-end restores it when it re-emits rendered text (see [The error
pipeline](error-pipeline.md)). That is distinct from the cascade-collapsing deduplication above:
this one drops byte-identical repeats to match rustc's own rendering; the processor's job is to
recognize *related* diagnostics that share a root cause and are not identical at all.

## Testing with snapshots

Because the processor is a pure function over serializable data, it is tested without running the
tool. A fixture is a text file holding serialized diagnostics — captured once from a real compilation
and committed — and a test reads it, calls `process_cgp_errors`, and asserts over the returned
`CgpDiagnostic` set. Nothing compiles, no driver runs, and the test is fast and deterministic, so a
whole catalog of error classes can be exercised as ordinary library tests. This is what the
statelessness requirement buys, and it is where the fuller per-class snapshot suite will grow as the
analysis lands. Today's tests assert the pass-through invariant: every diagnostic is preserved, in
order, with its rendered text intact.

These tests complement the existing UI suite rather than replace it; the two guard different seams.
The [UI snapshot tests](testing.md) drive the whole `cargo-cgp` executable against a compiled fixture
and pin its end-to-end output, so they prove capture, processing, and rendering work together against
the real compiler — and it is those snapshots that confirmed the pass-through re-render reproduces
rustc's output. The processing tests pin the transform in isolation, so a change in how a cascade is
collapsed shows up as a small, readable diff over structured data rather than buried in a wall of
rendered compiler output.

## Planned processing work

Once the placeholder is replaced by a real ingest-then-query core, the transformations below are the
work this stage takes on, listed roughly in order of expected value. Each operates on the captured
diagnostics as data — none needs the compiler, which is what keeps them in this stage rather than in
capture:

- **Decode CGP's type-level encodings.** A type printed as `Symbol<4, Chars<'n', Chars<'a', Chars<'m',
  Chars<'e', Nil>>>>>` is the field name `"name"`; `Cons<A, Cons<B, Nil>>` is `Product![A, B]`.
  Rewriting these spines back to their surface form removes most of the visual noise a CGP error
  carries.
- **Lift the root cause and collapse the cascade.** Detect the one deep mistake reported at every
  transitively dependent provider, present it first, and summarize or drop the repeats. This is the
  transformation that most needs the store, since finding the repetition is a query across the whole
  ingested set.
- **Map diagnostics to catalog classes.** Recognize an error's [catalog](../../../cgp/docs/errors/README.md)
  class from its shape and record it on the `CgpDiagnostic`, so a consumer can tell the user which
  kind of CGP mistake they are looking at.

One related transformation is deliberately *not* here: recovering an obligation chain the solver
renders tersely by re-running fulfillment through the compiler's `InferCtxt`. That needs live
compiler state, so it belongs to the capture stage and enriches the diagnostic data before this stage
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

- [`crates/cargo-cgp-error-processing/tests/passthrough.rs`](../../crates/cargo-cgp-error-processing/tests/passthrough.rs) —
  drives `process_cgp_errors` over a committed serialized fixture
  ([`tests/fixtures/sample_diagnostics.json`](../../crates/cargo-cgp-error-processing/tests/fixtures/sample_diagnostics.json))
  and asserts the pass-through invariant: every diagnostic preserved, in order, with its rendered text
  intact. This is the seed of the per-class snapshot suite described under
  [Testing with snapshots](#testing-with-snapshots).
- The [UI snapshot suite](testing.md) is the end-to-end guard that the front-end's capture-and-render
  around this stage reproduces rustc's output.

## Source

- [`crates/cargo-cgp-error-processing/src/process.rs`](../../crates/cargo-cgp-error-processing/src/process.rs) —
  `process_cgp_errors`, the pass-through placeholder, carrying the "do not grow into a map" warning and
  the `DiagInner` fallback note.
- [`crates/cargo-cgp-error-processing/src/diagnostic.rs`](../../crates/cargo-cgp-error-processing/src/diagnostic.rs) —
  the `CgpDiagnostic` superset type and its `passthrough`/`rendered` helpers.
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
