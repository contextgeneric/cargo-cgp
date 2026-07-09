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

This document owns the two stages that run inside a compilation, the configure and capture stages,
because both are coupled to the compiler and the driver. The process stage is a self-contained pure
function documented separately in [Error processing](error-processing.md); the render stage formats
what processing returns. The configure stage is the only one implemented today, and its two
transformations are the substance below.

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

Everything in this document happens in the driver, because the driver runs the real compiler
in-process through `rustc_driver` (the front-end only wires the driver into cargo — see
[Executable structure](executable-structure.md)). That gives two levers, and each transformation
below is one or the other:

- **The rustc arguments the driver injects** before compilation. This is coarse — it changes how the
  compiler produces diagnostics — but needs no diagnostic parsing. Both current transformations are of
  this kind, and both belong to the configure stage.
- **The driver's [`Callbacks`](../../crates/cargo-cgp-driver/src/callbacks.rs)**, which can read the
  compiler's diagnostics after analysis. This is the finer lever and is where the capture stage will
  live, including the enrichment that needs live compiler state. It is currently unused
  (`CgpCallbacks` is empty).

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

## The capture stage (planned)

Capture is the not-yet-built stage that collects rustc's diagnostics as the structured, serializable
data the [processing stage](error-processing.md) consumes. Two mechanisms are open, and the choice is
a tradeoff between simplicity and reach. The simpler one runs `cargo check --message-format=json` and
parses the stream with `cargo_metadata::Message::parse_stream` in the plain front-end, yielding
fully-rendered diagnostics with no `rustc_private`. The richer one installs a custom `Emitter` in the
driver — via `interface::Config.psess_created` — to intercept each `DiagInner` as the compiler builds
it, which is heavier (it must reconstruct rendering and resolve spans through the `SourceMap`) but is
the only path that can enrich a diagnostic with facts only the live compiler holds.

That enrichment is the reason the richer path may be worth its cost. Where even the next-gen solver
renders an obligation chain tersely, the driver can re-run trait fulfillment on the failing obligation
through the compiler's `InferCtxt` / `ObligationCtxt` API to reconstruct the full derived-obligation
chain — the surfaced form that `check_components!` forces at the source level — and attach it to the
diagnostic before processing sees it. This kind of work must happen here, during compilation, because
processing is stateless and cannot ask the compiler anything; it belongs to capture and not to the
processing stage's own planned work.

## Comparison with Clippy

Clippy is also a diagnostic tool built on this integration, but it transforms diagnostics differently.
It works entirely on the `Callbacks` lever: its `config` callback calls `register_lints` to add lint
passes that run during the same compilation (see
[`external/rust-clippy/src/driver.rs`](../../../external/rust-clippy/src/driver.rs)). `cargo-cgp`'s
current transformations are coarser and of the other lever — solver and verbosity flags, not
callbacks — because they buy a large improvement for no diagnostic-parsing work. The planned capture
and processing stages move onto the diagnostics themselves, but the aim still differs from Clippy's:
Clippy *adds* new diagnostics (lints), whereas `cargo-cgp` *rewrites and clarifies* the diagnostics
rustc already produces — which is why it needs a capture-and-process pipeline that Clippy has no
equivalent of.

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
  (empty) hook the planned capture stage will use.

## Further reading

- [Error processing](error-processing.md) — the stateless stage that consumes what capture produces:
  its interface, its input/output types, and why it is a many-to-fewer transform rather than a map.
- [Next-gen trait solving — Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/solve/trait-solving.html)
  — what the solver `-Znext-solver` selects is and how it evaluates goals.
- [Significant changes and quirks — Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/solve/significant-changes.html)
  — how the new solver differs from the old, the basis for the compatibility caveat above.
