# The error pipeline

`cargo-cgp` exists to turn rustc's raw diagnostics for CGP code into readable, root-cause-first
errors, and it does so through a pipeline of stages; this document is the map of that pipeline and the
detailed record of the stages that run inside a compilation. Today the tool implements the first
stage in two coarse ways — choosing the trait solver and un-eliding the diagnostic — and this is where
each such transformation is written down as it is added.

## The stages

The pipeline has four stages, and separating them is what lets each be reasoned about and tested on
its own. **Configure rustc** injects flags so the compiler emits better raw diagnostics than it
otherwise would. **Capture** collects those diagnostics as structured, serializable data. **Process**
transforms the captured diagnostics into CGP diagnostics — deduplicating a cascade and lifting the
root cause to the top — as a stateless function. **Render** formats the result as human-readable text
or, later, JSON.

These stages split across the two executables. The **configure** stage runs in the driver, inside
each compilation, because injecting rustc flags is coupled to the compiler; its two flag
transformations are the substance below. The **capture**, **process**, and **render** stages all run
in the plain front-end, which invokes `cargo check --message-format=json`, parses the diagnostics
back out, transforms them, and re-emits them. The process stage is a self-contained pure function
documented separately in [Error processing](error-processing.md). All four stages exist today, but
process is still a pass-through, so the output matches rustc's own diagnostics — the flag levers
below are the only transformations that change what a user sees.

## Why a transformation layer exists

A CGP macro expands to ordinary Rust, so most CGP mistakes are caught not by the macro but by the
compiler type-checking the generated code, where the diagnostic is shaped by CGP's machinery in ways
that make it hard to read: a single mistake can cascade across generated types the programmer never
wrote, and the real cause is often buried or hidden entirely. The [CGP error
catalog](../../../cgp/docs/errors/README.md) maps those classes — which hide the root cause, which
surface it, and where the cause sits. `cargo-cgp`'s job is to take rustc's output for those classes
and re-present it with the cause first; this document is the running account of the compilation-side
transformations it applies to get there.

## Where transformation happens

The two current transformations both happen in the driver, because the driver runs the real compiler
in-process through `rustc_driver` and can inject flags into each compilation (the front-end wires the
driver into cargo — see [Executable structure](executable-structure.md)). Injecting a flag is a
coarse lever — it changes how the compiler produces diagnostics — but needs no diagnostic parsing,
and both transformations below are of this kind. This is the configure stage.

Reading and rewriting the diagnostics themselves is a separate, finer concern, and today it happens
in the **front-end**, not the driver. The front-end runs `cargo check --message-format=json`, so
cargo's diagnostics arrive as a structured JSON stream it can parse, transform, and re-emit — no
`rustc_private` required. That is where the capture, process, and render stages live. The driver's
[`Callbacks`](../../crates/cargo-cgp-driver/src/callbacks.rs) is a *future* second capture lever, for
the enrichment that needs live compiler state (below); it can read the compiler's diagnostics after
analysis but is currently unused (`CgpCallbacks` is empty).

## Choosing the trait solver (current)

The driver injects `-Znext-solver=globally` into every workspace-crate compilation
([`config::NEXT_SOLVER_FLAG`](../../crates/cargo-cgp-driver/src/config.rs), applied in
[`args::rustc_args`](../../crates/cargo-cgp-driver/src/args.rs)), turning on the next-generation trait
solver. This is an argument-lever transformation, and it targets the class of CGP error whose root
cause the *default* solver hides.

When a provider's impl-side dependency is unmet — say a `name` field the context does not carry — and
the failure is reached by calling the consumer method directly, the default solver's
method-resolution path bottoms out at the provider trait (`Person: Greeter<Person>`) and never
reports the real missing leaf bound. That leaf is not merely omitted from the printed diagnostic: on
this path the default solver does not compute it at all (confirmed by tracing
`rustc_hir_typeck::method::probe` — the leaf predicate never appears).

The next-generation solver does compute it. Under `-Znext-solver=globally` the same mistake reports
`HasField is not implemented for Person with the field: Symbol<…"name"…>`, names the concrete
`Person: HasField<Symbol!("name")>` bound, and even renders CGP's own
`#[diagnostic::on_unimplemented]` hint ("add `#[derive(HasField)]`"). So merely compiling the
workspace crate under the new solver un-hides the cause — no diagnostic parsing required yet. The
flag is scoped to workspace crates (only they go through the driver), so dependencies still build
with the default solver. The before/after is pinned by the
[`usability/unsatisfied_dependency`](../../tests/ui/usability/unsatisfied-dependency/unsatisfied_dependency.stderr) UI
snapshot — a fixture that lives under `usability/` precisely because this solver switch has already
turned its once-hidden cause into a recoverable (if still verbose) one.

### Caveats

Two things follow from changing the solver. The new solver is not perfectly compatible with the old
one (see [Significant changes and quirks](https://rustc-dev-guide.rust-lang.org/solve/significant-changes.html)),
so in principle a crate could compile under a plain `cargo check` yet report a spurious error under
`cargo cgp check`, or the reverse; this is the accepted cost of using the new solver as the
diagnostic engine, and an explicit `-Znext-solver` on the command line still overrides the injection.
The reverse case is not merely hypothetical: the upstream `inheritance_cycle` fixture (two namespaces
that inherit from each other) is rejected by a plain `cargo check` with an `E0275` overflow but
**compiles clean** under `cargo cgp check`, because the next-gen solver does not eagerly overflow on
the mutually-recursive inheritance impls. That is why it is among the fixtures deliberately not
imported into the [usability mirror](../../tests/ui/usability/README.md) — there is no error to
snapshot — and it is a *missing* error, not a suppressed root cause.
Separately, the richer cross-crate diagnostics name absolute paths (the `cgp` checkout) and can point
at a hash-named temp file for an elided long type — volatile details the UI-test harness normalizes
away, described in [Testing](testing.md).

## Un-eliding the diagnostic (current)

The driver injects the stable `--verbose` flag into every workspace-crate compilation
([`config::VERBOSE_FLAG`](../../crates/cargo-cgp-driver/src/config.rs), applied in
[`args::rustc_args`](../../crates/cargo-cgp-driver/src/args.rs)). This is the second argument-lever
transformation, and it targets a different failure from the solver switch: not a cause the compiler
never computes, but a cause the compiler computes and then *elides while printing*.

rustc compresses long or repetitive types in its diagnostics, and every one of those compressions is
destructive to a CGP error, whose types are deep `Symbol` / `Cons` spines. When it reports "trait `X`
is not implemented … but trait `Y` is" it diffs the two traits and replaces every generic argument
they share with `_`; when a type's printed form grows long it truncates it and writes the full form to
a `long-type-*.txt` file; when more than nine impls could apply it collapses the rest to "and N
others". All three are gated on the compiler's `opts.verbose` flag, which `--verbose` sets, so the one
flag turns all three off and the full type is always present in the output. Crucially `--verbose` is
*not* `-Zverbose-internals`: it un-elides without switching on the compiler's internal debug printing
(disambiguator suffixes, region ids), so the diagnostics keep their ordinary shape. The full mechanism
— which function performs each elision, and the two-verbosity-switch distinction that makes `--verbose`
the surgical choice — is documented in
[rustc diagnostic internals](rustc-diagnostic-internals.md#the-suppression-points).

The worked case is a missing field surfaced through the two-line similar-impl hint. Asking for a
`height` field on a `Rectangle` that has only `width`, rustc diffs `HasField<Symbol!("height")>`
against `HasField<Symbol!("width")>`; because the two symbols share the character `'h'`, the shared
`'h'` was collapsed to `_` in *both* names, printing `h,e,i,g,_,t` and `w,i,d,t,_` — the field name
could not be read back from the text at all. Under `--verbose` both symbols print in full. The
before/after is pinned by the [`usability/base_area_1`](../../tests/ui/usability/checks/base_area_1.stderr) UI
snapshot, a fixture that lives under `usability/` precisely because the flag has turned its once-hidden
cause into a recoverable (if still verbose) one — the same graduation the solver switch gave
`unsatisfied_dependency`.

Like the solver flag, this one is skipped for cargo's info queries (`rustc -vV` and `--print`), which
carry no code to diagnose and which `--verbose` would actively break — `-vV` already implies `-v`, so a
second `--verbose` makes rustc reject the invocation. The skip is handled in `args::rustc_args`; see
[`config::VERBOSE_FLAG`](../../crates/cargo-cgp-driver/src/config.rs) for the full rationale.

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

A second, richer capture mechanism is planned but not built: installing a custom `Emitter` in the
driver — via `interface::Config.psess_created` — to intercept each `DiagInner` as the compiler builds
it. It is heavier (it must reconstruct rendering and resolve spans through the `SourceMap`) but is the
only path that can enrich a diagnostic with facts only the live compiler holds. Where even the
next-gen solver renders an obligation chain tersely, the driver could re-run trait fulfillment on the
failing obligation through the compiler's `InferCtxt` / `ObligationCtxt` API to reconstruct the full
derived-obligation chain — the surfaced form that `check_components!` forces at the source level — and
attach it to the diagnostic before processing sees it. This kind of work must happen during
compilation, because processing is stateless and cannot ask the compiler anything; it belongs to
capture and not to the processing stage's own planned work.

## Comparison with Clippy

Clippy is also a diagnostic tool built on this integration, but it transforms diagnostics differently.
It works entirely on the `Callbacks` lever: its `config` callback calls `register_lints` to add lint
passes that run during the same compilation (see
[`external/rust-clippy/src/driver.rs`](../../../external/rust-clippy/src/driver.rs)). `cargo-cgp`'s
current *flag* transformations are coarser and of the other lever — solver and verbosity flags, not
callbacks — because they buy a large improvement for no diagnostic-parsing work. Its capture and
processing stages do move onto the diagnostics themselves, but in the front-end over cargo's JSON
rather than in a callback, and the aim still differs from Clippy's: Clippy *adds* new diagnostics
(lints) during the compilation, whereas `cargo-cgp` *rewrites and clarifies* the diagnostics rustc
already produced, after the build — which is why it needs a capture-and-process pipeline that Clippy
has no equivalent of.

## Tests

- [`tests/ui/usability/unsatisfied-dependency/unsatisfied_dependency.stderr`](../../tests/ui/usability/unsatisfied-dependency/unsatisfied_dependency.stderr) —
  the UI snapshot that pins the un-hidden output the solver switch produces; the regression guard for
  the trait-solver transformation.
- [`tests/ui/usability/checks/base_area_1.stderr`](../../tests/ui/usability/checks/base_area_1.stderr) — the UI
  snapshot that pins the un-elided field name the `--verbose` injection produces; the regression guard
  for the un-eliding transformation (watch for a `_` returning inside its `Symbol`).
- [`crates/cargo-cgp-driver/tests/args.rs`](../../crates/cargo-cgp-driver/tests/args.rs) — tests that
  each injected flag is appended when absent, skipped when the invocation already sets it, and skipped
  entirely for the `-vV` and `--print` info queries.

## Source

- [`crates/cargo-cgp-driver/src/config.rs`](../../crates/cargo-cgp-driver/src/config.rs) —
  `NEXT_SOLVER_FLAG` and `VERBOSE_FLAG`, each with its rationale.
- [`crates/cargo-cgp-driver/src/args.rs`](../../crates/cargo-cgp-driver/src/args.rs) — injects the
  flags into the rustc argument vector, and skips injection for info queries.
- [`crates/cargo-cgp-driver/src/run.rs`](../../crates/cargo-cgp-driver/src/run.rs) — passes the flag
  set to `rustc_args`.
- [`crates/cargo-cgp-driver/src/callbacks.rs`](../../crates/cargo-cgp-driver/src/callbacks.rs) — the
  (empty) hook the planned driver-side capture mechanism will use.
- [`crates/cargo-cgp/src/check/command.rs`](../../crates/cargo-cgp/src/check/command.rs) — the
  front-end's capture and render: appends `--message-format=json`, captures cargo's output, runs the
  diagnostics through processing, and re-emits.
- [`crates/cargo-cgp/src/check/diagnostics.rs`](../../crates/cargo-cgp/src/check/diagnostics.rs) —
  parses cargo's JSON stream into diagnostics and re-renders the processed result, with the
  render-fidelity deduplication.

## Further reading

- [Error processing](error-processing.md) — the stateless stage that consumes what capture produces:
  its interface, its input/output types, and why it is a many-to-fewer transform rather than a map.
- [Next-gen trait solving — Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/solve/trait-solving.html)
  — what the solver `-Znext-solver` selects is and how it evaluates goals.
- [Significant changes and quirks — Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/solve/significant-changes.html)
  — how the new solver differs from the old, the basis for the compatibility caveat above.
