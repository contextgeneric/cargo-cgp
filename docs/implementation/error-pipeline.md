# The error pipeline

`cargo-cgp` exists to turn rustc's raw diagnostics for CGP code into readable, root-cause-first
errors, and it does so through a pipeline of stages; this document is the map of that pipeline — the
four stages, where each runs, and how they cooperate. It is the overview that ties the detailed
sub-documents together: the driver-side transformations that shape the raw diagnostics (the
trait-solver and verbosity flag levers, and the emitter that renames CGP wiring notes) are the subject
of [The driver](driver.md), and the stateless stage that reshapes the captured diagnostics is
[Error processing](error-processing.md).

## The stages

The pipeline has four stages, and separating them is what lets each be reasoned about and tested on
its own. **Configure rustc** injects flags so the compiler emits better raw diagnostics than it
otherwise would. **Capture** collects those diagnostics as structured, serializable data. **Process**
transforms the captured diagnostics into CGP diagnostics — deduplicating a cascade and lifting the
root cause to the top — as a stateless function. **Render** formats the result as human-readable text
or, later, JSON.

These stages split across the two executables. The **configure** stage runs in the driver, inside
each compilation, because injecting rustc flags is coupled to the compiler; the driver also rewrites
some diagnostics in place through a custom emitter during the same compilation, and both of those
driver-side transformations are detailed in [The driver](driver.md). The **capture**, **process**,
and **render** stages all run in the plain front-end, which invokes `cargo check
--message-format=json`, parses the diagnostics back out, transforms them, and re-emits them. The
process stage is a self-contained pure function documented separately in
[Error processing](error-processing.md). All four stages exist today; the process stage now runs its
per-diagnostic preprocessing pipeline (stripping CGP path prefixes, resugaring `Symbol!`, rewriting
unmet `HasField` bounds into missing-field messages), so it too changes what a user sees. Its
cross-diagnostic aggregation sub-stage — collapsing cascades — is still to come.

## Why a transformation layer exists

A CGP macro expands to ordinary Rust, so most CGP mistakes are caught not by the macro but by the
compiler type-checking the generated code, where the diagnostic is shaped by CGP's machinery in ways
that make it hard to read: a single mistake can cascade across generated types the programmer never
wrote, and the real cause is often buried or hidden entirely. The [CGP error
catalog](../../../cgp/docs/errors/README.md) maps those classes — which hide the root cause, which
surface it, and where the cause sits. `cargo-cgp`'s job is to take rustc's output for those classes
and re-present it with the cause first; this document maps the stages that do so, and
[The driver](driver.md) is the running account of the compilation-side transformations themselves.

## Where transformation happens

Transformation happens in both executables, and the split turns on what each rewrite needs. In the
**driver**, which runs the real compiler in-process through `rustc_driver` (the front-end wires it
into cargo — see [Executable structure](executable-structure.md)), two kinds of transformation run:
flag injections that change how the compiler *produces* diagnostics, and a custom emitter that
*rewrites* diagnostics needing facts only the live compiler holds. Both are detailed in
[The driver](driver.md). In the **front-end**, the capture, process, and render stages run over
cargo's JSON output — it invokes `cargo check --message-format=json`, so the diagnostics arrive as a
structured stream it can parse, transform, and re-emit with no `rustc_private` required.

The dividing line between the two rewriting sites is the `TyCtxt`: a text-only rewrite that any
consumer of the JSON could do belongs in the front-end's processing stage, while a rewrite that must
consult the compiler — like naming the traits behind a component marker — must run in the driver's
emitter, because the front-end has no compiler to ask.

## The driver's transformations

The driver applies three transformations during the compilation, each recorded in full in
[The driver](driver.md). Two are argument levers: `-Znext-solver=globally`
([choosing the trait solver](driver.md#choosing-the-trait-solver)) turns on the next-generation
solver, which computes the missing leaf bound the default solver never even reaches, and `--verbose`
([un-eliding the diagnostic](driver.md#un-eliding-the-diagnostic)) stops rustc from compressing the
deep `Symbol` / `Cons` types a CGP error carries. The third is a diagnostic rewrite: a custom emitter
that [names the traits behind a component marker](driver.md#naming-the-traits-behind-a-component-marker),
turning the marker-based header and wiring notes into ones that name the consumer and provider traits.
The first two shape what the compiler *produces*; the third edits what it has already *built*, using
the `TyCtxt`, which is why it lives in the driver rather than the front-end's processing stage.

## The capture and render stages

Capture collects rustc's diagnostics as the structured, serializable data the [processing
stage](error-processing.md) consumes, and it is implemented in the front-end. The front-end appends
`--message-format=json` to its `cargo check` invocation, captures cargo's stdout, and parses the
stream with `cargo_metadata::Message::parse_stream`, yielding
[`cargo_metadata::Diagnostic`](../../../external/cargo_metadata/src/diagnostic.rs) values — no
`rustc_private` required. The append is skipped if the caller already chose a message format, so a
user asking for JSON output still gets it. This path lives in
[`check::diagnostics`](../../crates/cargo-cgp/src/check/diagnostics.rs) and
[`check::command`](../../crates/cargo-cgp/src/check/command.rs).

Render turns the processed diagnostics back into the output a user sees, and today it reproduces
rustc's own pretty text. Each `cargo_metadata::Diagnostic` carries a `rendered` field holding the
exact text rustc would print, so the front-end prints that field for every processed diagnostic, then
replays cargo's own captured output (progress, the "could not compile" summary) after it — preserving
the "diagnostics then summary" order rustc's streaming output produced. One fidelity detail lives
here: rustc's *human* emitter suppresses exact-duplicate diagnostics from the terminal (while still
counting them), but the JSON stream repeats them, so the render step drops byte-identical repeats to
match. Because the diagnostics are processed as a set, capture buffers the whole build rather than
streaming it, so cargo's progress is shown at the end rather than live — the cost of a stage that must
see every diagnostic before it can reorder them.

Capture in the front-end is not the only place diagnostics are collected. The driver also intercepts
each `DiagInner` as the compiler builds it, through the custom emitter that powers the
[trait-renaming transform](driver.md#naming-the-traits-behind-a-component-marker). That same seam has
since grown richer compiler-state enrichment: the [typed root-cause resolver](typed-root-cause-resolution.md)
re-runs a failing check obligation through the `InferCtxt` / `ObligationCtxt` API to recover the
missing `HasField` and *replace* the diagnostic, rather than reading its text. That kind of work must
happen in the driver, because the front-end's processing stage is stateless and cannot ask the
compiler anything; the [driver deep dive](driver.md) covers the emitter seam and the resolver document
covers the replacement.

## Comparison with Clippy

Clippy is also a diagnostic tool built on this integration, but its pipeline has a different shape
because its aim is different: Clippy *adds* diagnostics, whereas `cargo-cgp` *rewrites and clarifies*
the ones rustc already produced. Clippy therefore needs no equivalent of `cargo-cgp`'s capture and
process stages — it emits its lints through the compiler's own machinery during the compilation and is
done, with nothing to collect from cargo's JSON afterward and nothing to reorder. `cargo-cgp`, by
contrast, must capture the whole build, reshape it, and re-emit it, which is the reason the pipeline
has a front-end half at all. How the two tools diverge in the *driver* — Clippy registering lints
where `cargo-cgp` installs a rewriting emitter, and the flag levers `cargo-cgp` adds — is compared in
the [driver deep dive](driver.md#comparison-with-clippy).

## Tests

The tests that pin the driver's three transformations are listed in the
[driver deep dive](driver.md#tests). This document's own stages — capture and render in the front-end
— are exercised end to end by the UI snapshot suite rather than by dedicated unit tests: every
fixture's `.cgp.stderr` is what the front-end captured, processed, and rendered. The
[Testing](testing.md) document describes that suite and its three passes.

## Source

- [`crates/cargo-cgp/src/check/command.rs`](../../crates/cargo-cgp/src/check/command.rs) — the
  front-end's capture and render: appends `--message-format=json`, captures cargo's output, runs the
  diagnostics through processing, and re-emits.
- [`crates/cargo-cgp/src/check/diagnostics.rs`](../../crates/cargo-cgp/src/check/diagnostics.rs) —
  parses cargo's JSON stream into diagnostics and re-renders the processed result, with the
  render-fidelity deduplication.
- The driver-side configure and rewrite stages are in
  [`crates/cargo-cgp-driver/src`](../../crates/cargo-cgp-driver/src); see the
  [driver deep dive](driver.md#source) for the per-module list.

## Further reading

- [The driver](driver.md) — the driver-side transformations this pipeline's configure and rewrite
  steps are made of, in full.
- [Error processing](error-processing.md) — the stateless stage that consumes what capture produces:
  its interface, its input/output types, and why it is a many-to-fewer transform rather than a map.
