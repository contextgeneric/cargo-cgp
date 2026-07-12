# The error pipeline

`cargo-cgp` exists to turn rustc's raw diagnostics for CGP code into readable, root-cause-first
errors, and it does so through a pipeline of stages that now run **entirely inside the driver**; this
document is the map of that pipeline — the stages, where each runs, and how they cooperate. It is the
overview that ties the detailed sub-documents together: the driver-side transformations that shape
the diagnostics (the trait-solver and verbosity flag levers, the emitter that renames CGP wiring
notes, and the fallback post-processing) are the subject of [The driver](driver.md), the typed
root-cause resolution is [Typed root-cause resolution](typed-root-cause-resolution.md), and the
rustc-free string helpers those transforms are built on are [Error processing](error-processing.md).

## The stages

The pipeline has three stages, and all of them run in the driver, inside each compilation.
**Configure rustc** injects flags so the compiler emits better raw diagnostics than it otherwise
would. **Transform** rewrites each diagnostic the compiler builds — turning a check failure into its
root-cause dependency tree, renaming CGP wiring notes, and post-processing the text so no raw CGP
construct leaks. **Render** writes the transformed diagnostic out, as human text or as JSON,
whichever error format the invocation asked for.

The front-end plays no part in the pipeline beyond launching it. It runs `cargo check` with the
driver wired in as the workspace rustc wrapper and lets cargo's output stream straight through, so
the diagnostics a user sees are exactly what the driver rendered (see
[Executable structure](executable-structure.md)). This is a change from an earlier design in which
the front-end captured cargo's JSON, processed it, and re-rendered it; moving every stage into the
driver is what lets the front-end stay a plain pass-through and restores live cargo progress.

## Why a transformation layer exists

A CGP macro expands to ordinary Rust, so most CGP mistakes are caught not by the macro but by the
compiler type-checking the generated code, where the diagnostic is shaped by CGP's machinery in ways
that make it hard to read: a single mistake can cascade across generated types the programmer never
wrote, and the real cause is often buried or hidden entirely. The [CGP error
catalog](../../../cgp/docs/errors/README.md) maps those classes — which hide the root cause, which
surface it, and where the cause sits. `cargo-cgp`'s job is to take rustc's output for those classes
and re-present it with the cause first; this document maps the stages that do so, and
[The driver](driver.md) is the running account of the transformations themselves.

## Where transformation happens

Every transformation happens in the driver's custom emitter, because that is the one place with the
compiler state the transforms need. The driver runs the real compiler in-process through
`rustc_driver`, so its emitter reaches the live `TyCtxt` (from thread-local scope, valid because a
wiring message is built during trait solving) — which is what lets it name the consumer and provider
traits behind a component marker and re-run a check obligation to recover its root cause. A front-end
that only saw cargo's serialized output could do none of that, which is why the whole layer lives in
the driver.

The emitter transforms each diagnostic in two tiers, then post-processes the result. When the
diagnostic is a resolvable CGP wiring failure, the [typed root-cause resolver](typed-root-cause-resolution.md)
replaces it with its dependency tree(s) and a coded main message. Otherwise a text
[wiring-message rewrite](driver.md#naming-the-traits-behind-a-component-marker) renames the CGP wiring
notes it recognizes. Either way, the diagnostic then passes through the
[post-processing](error-processing.md) transforms — stripping CGP path prefixes, resugaring `Symbol!`,
rewording an unmet `HasField` bound — so a diagnostic the tool did not fully rewrite still reads
cleanly, and the compiler-formatted CGP type names a rewrite embeds are tidied too.

## Configuring rustc

The driver injects two flags into every workspace-crate compilation, each a parse-free lever on how
the compiler *produces* diagnostics rather than a rewrite of what it built. `-Znext-solver=globally`
([choosing the trait solver](driver.md#choosing-the-trait-solver)) turns on the next-generation
solver, which computes the missing leaf bound the default solver never even reaches, and `--verbose`
([un-eliding the diagnostic](driver.md#un-eliding-the-diagnostic)) stops rustc from compressing the
deep `Symbol` / `Cons` types a CGP error carries. Both are detailed in [The driver](driver.md).

## Rendering

Rendering turns the transformed diagnostic into the output a user (or a tool) reads, and the driver
does it the way vanilla `rustc` would. The driver's emitter, `CgpEmitter<E>`, is generic over an
inner emitter and wraps whichever the compiler's own `default_emitter` would build for the active
error format — a `JsonEmitter` for `--message-format=json`, an `AnnotateSnippetEmitter` for the
default human format. The emitter mutates the compiler's `DiagInner` in place before handing it to
that inner emitter, so the transform reaches both a JSON diagnostic's structured `children` and its
regenerated `rendered` field, and a human diagnostic's rendered text, with no re-parsing. Because the
inner emitter is the compiler's own, `cargo-cgp-driver`'s output matches plain `rustc`'s apart from
the CGP transforms — which is also why a fixture's `.cgp.stderr` and its plain-`cargo check`
`.rust.stderr` baseline share a renderer, so their diff is purely the tool's work (see
[Testing](testing.md)).

## Comparison with Clippy

Clippy is also a diagnostic tool built on this integration, but its pipeline has a different shape
because its aim is different: Clippy *adds* diagnostics, whereas `cargo-cgp` *rewrites and clarifies*
the ones rustc already produced. Both do their work inside the compilation — Clippy through lint
passes, `cargo-cgp` through a rewriting emitter — and both let cargo carry the output out unchanged,
so neither needs a front-end processing stage. The difference is that Clippy's emitter is the
compiler's default, while `cargo-cgp` wraps that default in one that edits each diagnostic first. How
the two diverge in the driver — Clippy registering lints where `cargo-cgp` installs a rewriting
emitter, and the flag levers `cargo-cgp` adds — is compared in the
[driver deep dive](driver.md#comparison-with-clippy).

## Tests

The tests that pin the driver's transformations are listed in the
[driver deep dive](driver.md#tests) and [Typed root-cause resolution](typed-root-cause-resolution.md#tests);
the rustc-free post-processing and rewrite transforms are unit-tested in the
[`cargo-cgp-error-processing`](../../crates/cargo-cgp-error-processing) crate. This document's own
concern — that the front-end forwards the driver's output faithfully — is exercised end to end by the
UI snapshot suite: every fixture's `.cgp.stderr` is what the whole tool, front-end and driver,
printed. The [Testing](testing.md) document describes that suite and its two passes.

## Source

- The driver-side configure, transform, and render stages are in
  [`crates/cargo-cgp-driver/src`](../../crates/cargo-cgp-driver/src); see the
  [driver deep dive](driver.md#source) for the per-module list.
- [`crates/cargo-cgp/src/check/command.rs`](../../crates/cargo-cgp/src/check/command.rs) — the
  front-end's whole role: run the wrapped `cargo check` with the driver installed and forward its
  output untouched.
- [`crates/cargo-cgp-error-processing/src`](../../crates/cargo-cgp-error-processing/src) — the
  rustc-free rewrite, post-processing, and tree-rendering helpers the driver's transforms are built
  on.

## Further reading

- [The driver](driver.md) — the driver-side transformations this pipeline's configure, transform, and
  render stages are made of, in full.
- [Error processing](error-processing.md) — the rustc-free string helpers the transform stage uses:
  the wiring rewrite, the post-processing text transforms, and the dependency-tree renderer.
