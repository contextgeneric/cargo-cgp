# Error processing

`cargo-cgp` turns a compiler's raw diagnostics into readable, root-cause-first CGP errors, and
*processing* is the stage that does the turning: a single stateless function takes the structured
diagnostics rustc produced and returns a smaller, reordered set of CGP diagnostics. This document
records that stage's interface, the types on either side of it, why it must be a stateful analysis
rather than a per-error rewrite, and how it is tested — the design agreed before it is built.

**Status: designed, not yet built.** The tool captures no diagnostics today — both current
transformations are flag injections that let rustc's own output flow straight to the terminal (see
[The error pipeline](error-pipeline.md)). This document is the interface and the requirements settled
*before* the first line of the processor is written, so that the first implementation and every one
after it share one shape. Where it says "the processor does X," read "the processor is specified to
do X."

## Where processing sits in the pipeline

Processing is the third of four stages, and it is the only one that needs neither the compiler nor
the filesystem. The [error pipeline](error-pipeline.md) runs: **configure rustc** (inject flags so
the compiler emits better raw diagnostics), **capture** (collect those diagnostics as structured,
serializable data), **process** (this stage — transform the captured diagnostics into CGP
diagnostics), and **render** (format the result as human-readable text or JSON). The first two stages
run inside a compilation and are the pipeline document's subject; the last renders the output this
stage produces. Processing is deliberately isolated between them as a pure function, because that
isolation is what makes it testable and reorderable — the two properties this stage exists to
deliver.

The boundary with capture is worth stating precisely, because it decides where each piece of logic
belongs. Anything that needs the compiler's live state — re-running trait fulfillment through the
`InferCtxt` / `ObligationCtxt` API to reconstruct an obligation chain, or reading an interned
`Symbol` type the printer elided — must happen during capture, inside the driver, and be folded into
the captured data before processing sees it. Processing itself receives only already-serialized
diagnostics and has no way to ask the compiler anything. This keeps the expensive, compiler-coupled
work upstream and leaves processing a self-contained transform over plain data.

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

**A future change must not collapse this into a naive loop.** The most likely wrong turn is to see
the initial placeholder (below), which walks the input once, and "extend" it by adding rewrite logic
inside that walk — turning the stage into `rust_errors.iter().map(...).collect()` with per-error
branches. That shape can never deduplicate a cascade or lift a cause above the errors that follow
it, because each step is blind to the others. The two-phase ingest-then-query structure is the
design, not an optimization to be added later; the placeholder is a stand-in for the query phase, not
a skeleton to be fleshed out in place.

## The interface

The stage is one plain function, stateless and free of side effects:

```rust
pub fn process_cgp_errors(rust_errors: &[RustError]) -> Vec<CgpError>;
```

Statelessness is a hard requirement, not a stylistic preference, because it is what lets the stage be
tested without the rest of the tool. A pure function over serializable input and serializable output
can be driven from a unit test that reads a fixture file and compares a snapshot, with no compiler,
no cargo, and no `cargo-cgp` process in the loop (see [Testing with snapshots](#testing-with-snapshots)).
Any state the analysis needs — the internal store — lives *inside* one call and is gone when it
returns; nothing persists between calls and nothing is read from the environment.

The function belongs in a compiler-independent library, so that neither building nor testing it pulls
in `rustc_private`. The driver crate links the compiler's unstable internals and can only be built on
the pinned nightly; keeping the processor out of it means the stage compiles and its tests run on any
toolchain. The capturing side — whichever of the two mechanisms below the tool adopts — depends on
this library and calls into it; the library never depends on the capturing side.

## The input type: `RustError`

`RustError` is the structured form of one rustc diagnostic, and the practical choice for it is
[`cargo_metadata::Diagnostic`](../../../external/cargo_metadata/src/diagnostic.rs). That type is the
public, `serde`-deserializable mirror of the exact JSON shape rustc emits under
`--message-format=json`: its `message`, `code`, `level`, `spans`, `children`, and `rendered` fields
line up field-for-field with the compiler's own wire format. Being deserializable is what lets a
fixture be read back from a text file, and being the established public mirror means the tool tracks a
stable type rather than a private compiler internal. It is not yet a dependency of `cargo-cgp` (it
sits alongside the repository only as a read-only reference), so adopting it as `RustError` means
adding it — or a local type modeled on it — to the workspace.

The compiler's own diagnostic types are the wrong choice, and knowing why saves a later agent the
investigation. The in-memory `rustc_errors::DiagInner` is `rustc_private`, is not serializable,
carries untranslated messages and unresolved `MultiSpan`s that need a `SourceMap` to become
file-and-line, and has no rendered form — none of which survives a trip to a fixture file. The
structs rustc's `JsonEmitter` serializes *are* the right shape, but they are module-private inside
`rustc_errors`, derive `Serialize` only (no `Deserialize`), and are not exported, so even a
`rustc_private` build cannot name or read them back. `cargo_metadata::Diagnostic` exists precisely to
be the reusable public counterpart, which is why it is the input type.

How the diagnostics reach this type — the capture stage — is a separate decision that processing does
not constrain, and the pipeline document owns it; two mechanisms are open. The simpler one runs
`cargo check --message-format=json` and parses the stream with `cargo_metadata::Message::parse_stream`
in the plain front-end, yielding fully-rendered `Diagnostic`s with no `rustc_private` at all. The
richer one installs a custom `Emitter` in the driver (via `Config.psess_created`) to intercept each
`DiagInner` live, which is heavier — it must reconstruct rendering and resolve spans through the
`SourceMap` — but is the only path that can attach the compiler-state enrichment described above.
Whichever is chosen, it produces `RustError` values and hands them to `process_cgp_errors`.

## The output type: `CgpError`

`CgpError` is a structural superset of `RustError`: it carries everything a diagnostic carries, plus
optional CGP-specific structure. The superset shape is what lets one output type serve two kinds of
result. A **passed-through** diagnostic — a non-CGP error, or a CGP error the processor does not yet
handle — is a `CgpError` whose base diagnostic fields hold the original and whose CGP-specific fields
are empty. A **synthesized** CGP diagnostic fills those extra fields with what the analysis
recovered: a classified [catalog](../../../cgp/docs/errors/README.md) class, decoded type-level
encodings, the link from a root cause to the cascade it explains. Rendering never has to special-case
the two, because both are the same type and both always carry the base diagnostic to fall back on.
The concrete type is expected to be a struct — a working name is `CgpDiagnostic` — that owns (or
wraps) the diagnostic data and adds the extra fields; an enum split between a passthrough variant and
a synthesized variant is the alternative, and the choice is left to the implementation.

`CgpError` must stay structured data, never a pre-rendered string, for the same reason rustc keeps
its diagnostics structured until the emitter runs. The output feeds the render stage, which must be
able to produce more than one form from it — human-readable text today, and the JSON that
`--message-format=json` consumers expect later. A diagnostic frozen into a text blob at the end of
processing could only ever be printed one way; keeping it structured defers the formatting choice to
the stage that owns it.

## The initial passthrough placeholder

The first implementation does no CGP analysis at all: it treats every input as a non-CGP error and
returns each one as a passed-through `CgpError`. Because `CgpError` is a superset of `RustError`, this
is well-typed and correct — it reproduces the current behavior (rustc's diagnostics, unchanged)
through the new interface, so the pipeline can be wired end to end before any transformation logic
exists. It is a scaffold that proves the types and the plumbing, not the stage doing its job.

The placeholder carries a warning in its own doc comment, and the warning is the reason this section
exists: the placeholder happens to walk the input one-to-one, and that shape must not become the
design. A later agent extending it must build the ingest-then-query structure described above, not
add rewrite branches inside the passthrough walk. The distinction is subtle precisely because the
placeholder looks like the start of a per-error loop, so the comment says plainly that it is a
stand-in for the query phase and that the real processor reads the whole set before it emits
anything.

## Testing with snapshots

Because the processor is a pure function over serializable data, it is tested as a snapshot unit test
that never runs the tool. A fixture is a text file holding serialized `RustError` values — captured
once from a real compilation and committed — and a test reads it, calls `process_cgp_errors`, and
compares the returned `CgpError` set against a committed snapshot. Nothing compiles, no driver runs,
and the test is fast and deterministic, so a whole catalog of error classes can be exercised as
ordinary library tests. This is what the statelessness requirement buys.

These snapshots complement the existing UI suite rather than replace it; the two guard different
seams. The [UI snapshot tests](testing.md) drive the whole `cargo-cgp` executable against a compiled
fixture and pin its end-to-end output, so they prove capture, processing, and rendering work together
against the real compiler. The processing snapshots pin the transform in isolation, so a change in
how a cascade is collapsed shows up as a small, readable diff over structured data rather than buried
in a wall of rendered compiler output. A regression in the analysis is caught by the processing
snapshots; a regression in the wiring around it is caught by the UI suite.

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
  class from its shape and record it on the `CgpError`, so a consumer can tell the user which kind of
  CGP mistake they are looking at.

One related transformation is deliberately *not* here: recovering an obligation chain the solver
renders tersely by re-running fulfillment through the compiler's `InferCtxt`. That needs live
compiler state, so it belongs to the capture stage and enriches the `RustError` data before this
stage runs; it is recorded in [The error pipeline](error-pipeline.md).

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

No tests exist yet, because the stage is not yet built. When it is, this section lists each snapshot
that pins a behavior described above:

- The processing snapshot suite — the fixture-and-snapshot tests that drive `process_cgp_errors`
  directly, one per error class, described under [Testing with snapshots](#testing-with-snapshots).
  Its intended home is the processor library's `tests/` directory, alongside the other crate tests
  per the [repository conventions](../../AGENTS.md#code-organization-conventions).

## Source

The stage is not yet implemented; this section will link its modules once it is. The intended home is
a compiler-independent library crate (or a module in the plain front-end) holding `process_cgp_errors`,
the `RustError` input type, and the `CgpError` output type, kept out of the `rustc_private`-linked
driver so it builds and tests on any toolchain.

## Further reading

- [The error pipeline](error-pipeline.md) — the surrounding stages: how rustc is configured to emit
  better diagnostics, how they are captured, and how the processed result is rendered.
- [rustc diagnostic internals](rustc-diagnostic-internals.md) — where the compiler elides the
  information this stage would otherwise have to reconstruct, and the `InferCtxt` enrichment path that
  runs during capture.
- [CGP error catalog](../../../cgp/docs/errors/README.md) — the error classes the processor must
  learn to recognize, and where each class hides or surfaces its root cause.
