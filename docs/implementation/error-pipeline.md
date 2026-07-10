# The error pipeline

`cargo-cgp` exists to turn rustc's raw diagnostics for CGP code into readable, root-cause-first
errors, and it does so through a pipeline of stages; this document is the map of that pipeline and the
detailed record of the stages that run inside a compilation. The driver applies three transformations
today: two coarse argument levers — choosing the trait solver and un-eliding the diagnostic — and a
finer one that reads the compiler's own state to rename CGP wiring notes. This is where each such
transformation is written down as it is added.

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
documented separately in [Error processing](error-processing.md). All four stages exist today; the
process stage now runs its per-diagnostic preprocessing pipeline (stripping CGP path prefixes,
resugaring `Symbol!`, rewriting unmet `HasField` bounds into missing-field messages), so it too
changes what a user sees, on top of the flag levers below. Its cross-diagnostic aggregation sub-stage
— collapsing cascades — is still to come.

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

Two of the three transformations are flag injections, and they happen in the driver because it runs
the real compiler in-process through `rustc_driver` and can inject flags into each compilation (the
front-end wires the driver into cargo — see [Executable structure](executable-structure.md)).
Injecting a flag is a coarse lever — it changes how the compiler produces diagnostics — but needs no
diagnostic parsing. This is the configure stage.

Reading and rewriting the diagnostics themselves is a separate, finer concern, and it happens in two
places. In the **front-end**, the capture, process, and render stages run over cargo's JSON output —
the front-end runs `cargo check --message-format=json`, so the diagnostics arrive as a structured
stream it can parse, transform, and re-emit with no `rustc_private` required. In the **driver**, a
custom diagnostic emitter rewrites diagnostics that need facts only the live compiler holds, before
they are serialized; the trait-renaming transform below is the first use of it, and it is installed
through the driver's [`Callbacks`](../../crates/cargo-cgp-driver/src/callbacks.rs). The two rewriting
sites split by what each needs: text-only rewrites that any consumer of the JSON could do live in the
front-end, and rewrites that must consult the `TyCtxt` live in the driver.

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

## Naming the traits behind a component marker (current)

The driver rewrites the compiler's wiring notes to name the consumer and provider traits a reader
thinks in, in place of the internal marker-based phrasing. Where rustc reports `` required for
`RectangleArea` to implement `IsProviderFor<AreaCalculatorComponent, Rectangle>` `` and `` required
for `Rectangle` to implement `CanUseComponent<AreaCalculatorComponent>` ``, the tool now emits
`` required for the provider `RectangleArea` to implement the provider trait `AreaCalculator` for the
context `Rectangle` `` and `` required for the context `Rectangle` to implement the consumer trait
`CanCalculateArea` ``. This is the transform the `IsProviderFor` and `CanUseComponent` marker traits
otherwise hide: the component marker names neither trait, its `…Component` suffix is at best an
unreliable guess at the provider trait, and it says nothing at all about the consumer trait.

This is the first transformation that reads the compiler's own state rather than pulling an argument
lever, and that is why it lives in the driver. The two flag levers above change how the compiler
*produces* diagnostics; this one edits diagnostics the compiler has already *built*, using the trait
names only a live `TyCtxt` can supply. The driver installs a custom diagnostic emitter through the
callbacks' `config` hook, and that emitter rewrites each diagnostic in place before handing it to a
real `JsonEmitter`, so both the JSON `children` and the regenerated `rendered` text carry the new
wording — the front-end receives the diagnostic already transformed. The emitter must be rebuilt to
match the compiler's default because `set_emitter` replaces rather than wraps; the mechanics are in
[`emitter`](../../crates/cargo-cgp-driver/src/emitter.rs).

Recovering the names inverts two links `#[cgp_component]` generates, both in
[`component_map`](../../crates/cargo-cgp-driver/src/component_map.rs). A component marker
(`AreaCalculatorComponent`) is an empty struct with no reference to its traits, so the map is built by
walking the trait graph: the provider trait carries `IsProviderFor<Marker, …>` as a supertrait, so
scanning every trait's super-predicates yields each (provider trait, marker) pair, and the consumer
trait's blanket impl reads `impl<C> Consumer for C where C: Provider<C>`, so a blanket impl bounding
its *own* self type on a known provider trait names that provider's consumer (the self-type check is
what tells this apart from the provider blanket impl, which bounds the same trait on a projected
delegate). The walk runs once, lazily on the first wiring note and cached thereafter, and it is
reachable from inside the emitter because a wiring note is built during trait solving, when a
`TyCtxt` is in thread-local scope (`rustc_middle::ty::tls`).

The rewrite itself is a plain string transform, kept compiler-free in
[`rewrite`](../../crates/cargo-cgp-driver/src/rewrite.rs) so it is unit-tested without a `TyCtxt`. It
matches the two `required for … to implement …` note forms, reads the marker out of the trait's
generic arguments, and looks the names up in the map; a note whose marker is absent from the map, or
any other note, passes through untouched. One faithful oddity follows from naming the obligation's
subject verbatim: the subject is usually a provider (`RectangleArea`, `ScaledArea<RectangleArea>`) but
is the context itself when the context stands in as its own provider, so a self-provider case reads
`` the provider `Rectangle` … for the context `Rectangle` ``. The before/after is pinned across the
whole `usability/checks` fixture set, whose blessed snapshots now show the trait-named notes;
[`base_area_1`](../../tests/ui/usability/checks/base_area_1.stderr) is the worked example.

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

A second, richer capture mechanism is now partly built: a custom `Emitter` in the driver, installed
via `interface::Config.psess_created`, intercepts each `DiagInner` as the compiler builds it. The
[trait-renaming transform](#naming-the-traits-behind-a-component-marker-current) is its first use — it
reads the `TyCtxt` from thread-local scope to rename wiring notes — and the same seam is where further
compiler-state enrichment will hang. Where even the next-gen solver renders an obligation chain
tersely, the driver could re-run trait fulfillment on the failing obligation through the compiler's
`InferCtxt` / `ObligationCtxt` API to reconstruct the full derived-obligation chain — the surfaced
form that `check_components!` forces at the source level — and attach it to the diagnostic before the
front-end sees it. This kind of work must happen during compilation, because the front-end's
processing stage is stateless and cannot ask the compiler anything; it belongs to the driver, not to
the processing stage's own planned work.

## Comparison with Clippy

Clippy is also a diagnostic tool built on this integration, but it transforms diagnostics differently.
It works on the `Callbacks` lever: its `config` callback calls `register_lints` to add lint passes that
run during the same compilation (see
[`external/rust-clippy/src/driver.rs`](../../../external/rust-clippy/src/driver.rs)). `cargo-cgp` uses
the same lever but for a different end — its `config` callback installs a diagnostic *emitter* that
rewrites the compiler's own notes, where Clippy's *registers lints* that emit new ones. Its other two
driver transformations are coarser still, of the flag lever — solver and verbosity flags — because
they buy a large improvement for no diagnostic work. And its front-end capture and processing stages
move onto the diagnostics over cargo's JSON, which Clippy has no equivalent of. The throughline is the
aim: Clippy *adds* diagnostics, whereas `cargo-cgp` *rewrites and clarifies* the ones rustc already
produced — in the driver's emitter for what needs the compiler, and in the front-end for what does
not.

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
- [`crates/cargo-cgp-driver/tests/rewrite.rs`](../../crates/cargo-cgp-driver/tests/rewrite.rs) — tests
  the compiler-free note rewrite over a hand-built name map: both note forms, the module-prefix and
  generic-context cases, and the unknown-marker and unrelated-note pass-throughs.
- [`tests/ui/usability/checks/`](../../tests/ui/usability/checks) — the blessed `.stderr`/`.output.json`
  snapshots pin the trait-named notes end to end; a regression to the marker-based phrasing changes
  them.

## Source

- [`crates/cargo-cgp-driver/src/config.rs`](../../crates/cargo-cgp-driver/src/config.rs) —
  `NEXT_SOLVER_FLAG` and `VERBOSE_FLAG`, each with its rationale.
- [`crates/cargo-cgp-driver/src/args.rs`](../../crates/cargo-cgp-driver/src/args.rs) — injects the
  flags into the rustc argument vector, and skips injection for info queries.
- [`crates/cargo-cgp-driver/src/run.rs`](../../crates/cargo-cgp-driver/src/run.rs) — passes the flag
  set to `rustc_args`.
- [`crates/cargo-cgp-driver/src/callbacks.rs`](../../crates/cargo-cgp-driver/src/callbacks.rs) — the
  `config` hook that installs the rewriting emitter.
- [`crates/cargo-cgp-driver/src/emitter.rs`](../../crates/cargo-cgp-driver/src/emitter.rs) — the
  diagnostic-rewriting emitter: rebuilds the default `JsonEmitter`, reaches the `TyCtxt` via TLS, and
  rewrites wiring notes in place before delegating.
- [`crates/cargo-cgp-driver/src/component_map.rs`](../../crates/cargo-cgp-driver/src/component_map.rs) —
  builds the component-marker → trait-names map by inverting the `IsProviderFor` supertrait and
  consumer-blanket-impl links.
- [`crates/cargo-cgp-driver/src/rewrite.rs`](../../crates/cargo-cgp-driver/src/rewrite.rs) — the
  compiler-free string rewrite of the two wiring-note forms.
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
