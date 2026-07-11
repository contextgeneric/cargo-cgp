# The driver

`cargo-cgp-driver` is the `rustc` replacement cargo runs for each workspace crate: it executes the
real compiler in-process through `rustc_driver`, and on that foothold it applies the transformations
that make CGP errors readable. This document is the deep dive into how the driver is built — how it
prepares the compiler's arguments, how it links the compiler's internal API, and the three
transformations it applies to the diagnostics — for an agent reviewing, debugging, or extending it.

The driver is one half of a two-executable design, and this document assumes the shape of the other
half. The front-end that invokes it, the reason the tool is split in two, and the environment
contract between them are in [Executable structure](executable-structure.md); the four-stage pipeline
the driver's transformations sit within — configure, capture, process, render — is
[The error pipeline](error-pipeline.md). This document owns the driver's internals; those two own the
surrounding structure.

## How cargo invokes the driver

Cargo runs the driver the way `RUSTC_WORKSPACE_WRAPPER` prescribes — the wrapper name, then the real
compiler path, then the rustc arguments:

```text
cargo-cgp-driver  /path/to/rustc  --edition=2024  --crate-name foo  src/lib.rs  ...
```

The driver runs the compiler in-process rather than shelling out, which is the whole reason it
exists: only in-process, through `rustc_driver`, can it install the callbacks and emitter that read
and rewrite diagnostics. The entrypoint is [`run::run`](../../crates/cargo-cgp-driver/src/run.rs),
called by the thin [`bin/cargo-cgp-driver.rs`](../../crates/cargo-cgp-driver/bin/cargo-cgp-driver.rs)
wrapper; it reads the sysroot from the environment, prepares the argument vector, and runs the
compiler.

## Preparing the argument vector

Before handing control to the compiler, [`args::rustc_args`](../../crates/cargo-cgp-driver/src/args.rs)
turns the wrapper's argument vector into a rustc one. It detects "wrapper mode" — the second argument
is a path whose file stem is `rustc` — and removes that injected compiler path, because
`rustc_driver::run_compiler` treats the vector's first element as the ignored program name and
everything after it as flags; leaving the `rustc` path in would make the compiler treat it as an
input file.

It then injects flags, each only when the invocation does not already set it. `--sysroot` is injected
with the value the front-end passes through `CARGO_CGP_SYSROOT`, because a `rustc_driver` binary
outside a toolchain cannot locate `std` on its own — the
[environment contract](executable-structure.md#the-environment-contract) covers both sides of that
hand-off. The two diagnostic flags `-Znext-solver=globally` and `--verbose` are injected next; they
drive the [trait-solver](#choosing-the-trait-solver) and [un-eliding](#un-eliding-the-diagnostic)
transformations below.

Injection is skipped entirely for cargo's info queries — `rustc -vV` and `--print`, which carry no
code to diagnose. `--verbose` would actively break `-vV`, since that already implies `-v` and a
second one makes rustc reject the invocation. Because each flag is injected only when absent, an
explicit choice on the command line always wins: a user's own `--sysroot`, or an explicit
`-Znext-solver=no`, is left in place.

## Running the compiler

The prepared vector runs under
[`rustc_driver::catch_with_exit_code`](../../crates/cargo-cgp-driver/src/run.rs), which executes the
compiler and converts a compiler-signalled failure into the process `ExitCode`, matching what plain
`rustc` returns. The compiler behavior the driver adds is installed through
[`callbacks::CgpCallbacks`](../../crates/cargo-cgp-driver/src/callbacks.rs); its `config` hook
installs the diagnostic-rewriting emitter described under
[Naming the traits behind a component marker](#naming-the-traits-behind-a-component-marker). Aside
from the injected flags and that emitter, the driver compiles exactly as `rustc` would.

## Accessing the Rust compiler API

The driver reaches the compiler through the `rustc_private` feature, which permits linking the
compiler's internal crates from the sysroot. Three facts about that access shape the crate.

First, the internal crates are pulled in by `extern crate`, not through Cargo. The library
[`lib.rs`](../../crates/cargo-cgp-driver/src/lib.rs) carries `#![feature(rustc_private)]` and one
`extern crate rustc_*;` line per compiler crate it uses — `rustc_driver` to run the compiler, and
`rustc_interface`, `rustc_errors`, `rustc_middle`, `rustc_session`, `rustc_span`,
`rustc_data_structures`, and `rustc_lint_defs` for the emitter and its queries. A module needing a
further crate adds another line there; the compiler crates are never declared under `[dependencies]`.
The crate's one ordinary Cargo dependency is `cargo-cgp-error-processing`, the rustc-free crate that
holds the message-rewrite logic and the `ComponentNameMap` the driver fills in from the compiler.

Second, the feature gate is needed on **both** the library and the binary crate. The binary
[`bin/cargo-cgp-driver.rs`](../../crates/cargo-cgp-driver/bin/cargo-cgp-driver.rs) repeats
`#![feature(rustc_private)]`, because the binary is what ultimately links the compiler dylib, and
that link is only permitted when the linking crate opts into the feature.

Third, the API is unstable and only ships with a nightly toolchain carrying the `rustc-dev`
component, so the toolchain is pinned in [`rust-toolchain.toml`](../../rust-toolchain.toml) to an
exact dated nightly and bumped deliberately. The pinned nightly is the compiler the driver *embeds*,
so when `cargo cgp check` runs against a project, that nightly does the checking, and the sysroot the
front-end discovers must belong to the same nightly — a mismatched sysroot would load the wrong
`librustc_driver`. In practice the tool runs under the pinned toolchain. Because the API moves
between nightlies, every use of it is verified against the read-only compiler checkout at
[`../external/rust`](../../../external/rust) before being relied on.

## The driver's diagnostic transformations

The driver applies its diagnostic transformations in two kinds. Two are **argument levers** — a flag
that changes how the compiler *produces* diagnostics, needing no diagnostic parsing. The rest run in a
**custom emitter** that acts on diagnostics the compiler has already *built*, using facts only the
live compiler holds; this is far more involved than a flag, because it links the compiler's internal
API to reach the `TyCtxt`. The emitter carries two transformations of its own: the in-place
[trait-renaming rewrite](#naming-the-traits-behind-a-component-marker) described below, and the deeper
[typed root-cause resolution](typed-root-cause-resolution.md) that *replaces* a missing-field check
failure with a root-cause-first diagnostic, covered in its own document. The three sections that
follow detail the two levers and the rename; the replacement builds on the rename's `TyCtxt` access
and is documented separately.

### Choosing the trait solver

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
workspace crate under the new solver un-hides the cause — no diagnostic parsing required. The flag is
scoped to workspace crates (only they go through the driver), so dependencies still build with the
default solver. The before/after is pinned by the
[`usability/unsatisfied_dependency`](../../tests/ui/usability/unsatisfied-dependency/unsatisfied_dependency.cgp.stderr)
UI snapshot — a fixture that lives under `usability/` precisely because this solver switch has
already turned its once-hidden cause into a recoverable (if still verbose) one.

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
snapshot — and it is a *missing* error, not a suppressed root cause. Separately, the richer
cross-crate diagnostics name absolute paths (the `cgp` checkout) and can point at a hash-named temp
file for an elided long type — volatile details the UI-test harness normalizes away, described in
[Testing](testing.md).

### Un-eliding the diagnostic

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
before/after is pinned by the
[`usability/base_area_1`](../../tests/ui/usability/checks/base_area_1.cgp.stderr) UI snapshot, a fixture
that lives under `usability/` precisely because the flag has turned its once-hidden cause into a
recoverable (if still verbose) one — the same graduation the solver switch gave
`unsatisfied_dependency`.

### Naming the traits behind a component marker

The driver rewrites the compiler's wiring diagnostics to name the consumer and provider traits a
reader thinks in, in place of the internal marker-based phrasing — both the obligation-chain notes and
the primary header the error opens with. Where rustc reports `` required for `RectangleArea` to
implement `IsProviderFor<AreaCalculatorComponent, Rectangle>` `` and `` required for `Rectangle` to
implement `CanUseComponent<AreaCalculatorComponent>` ``, the tool emits `` required for the provider
`RectangleArea` to implement the provider trait `AreaCalculator` for the context `Rectangle` `` and
`` required for the context `Rectangle` to implement the consumer trait `CanCalculateArea` ``. The
header is rewritten the same way: `` the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent>`
is not satisfied `` becomes `` the consumer trait bound `Rectangle: CanCalculateArea` is not
satisfied ``, and a provider-side header `` `RectangleArea: IsProviderFor<AreaCalculatorComponent,
Rectangle>` `` becomes the provider trait bound `` `RectangleArea: AreaCalculator<Rectangle>` `` —
recovering the actual provider-trait bound the marker form stands in for. This is the transform the
`IsProviderFor` and `CanUseComponent` marker traits otherwise hide: the component marker names neither
trait, its `…Component` suffix is at best an unreliable guess at the provider trait, and it says
nothing at all about the consumer trait.

This is the one transformation that reads the compiler's own state rather than pulling an argument
lever, and that is why it needs everything the driver's in-process access provides. The two flag
levers above change how the compiler *produces* diagnostics; this one edits diagnostics the compiler
has already *built*, using the trait names only a live `TyCtxt` can supply. The driver installs a
custom diagnostic emitter through the callbacks' `config` hook, and that emitter rewrites each
diagnostic in place before handing it to a real `JsonEmitter`, so both the JSON `children` and the
regenerated `rendered` text carry the new wording — the front-end receives the diagnostic already
transformed.

The emitter must be *rebuilt* rather than wrapped. The session's own emitter cannot be reached to
wrap it — `DiagCtxt::set_emitter` only replaces it, with no way to recover the original — so
[`emitter::install`](../../crates/cargo-cgp-driver/src/emitter.rs) reads the session options in the
callbacks' `config` hook and, from inside `psess_created`, rebuilds a `JsonEmitter` matching how the
compiler builds its default one, then wraps *that* and installs the wrapper. It rebuilds only for the
JSON error format — the one the front-end drives cargo with — and leaves a human-format invocation
(the driver run by hand) on the compiler's own emitter. The wrapper forwards every emitter method to
the inner `JsonEmitter` unchanged except `emit_diagnostic`, which rewrites first.

Recovering the names inverts two links `#[cgp_component]` generates, both built in
[`component_map`](../../crates/cargo-cgp-driver/src/component_map.rs). A component marker
(`AreaCalculatorComponent`) is an empty struct with no reference to its traits, so the map is built by
walking the trait graph. The provider trait carries `IsProviderFor<Marker, …>` as a supertrait, so
scanning every trait's super-predicates yields each (provider trait, marker) pair. The consumer
trait's blanket impl reads `impl<C> Consumer for C where C: Provider<C>`, so a blanket impl bounding
its *own* self type on a known provider trait names that provider's consumer — and requiring the
bound's self type to be the impl's self type is what tells this apart from the provider blanket impl,
which bounds the same provider trait on a projected `<C as DelegateComponent<…>>::Delegate` rather
than on `C` itself. Composing the two gives, per marker, both trait names.

The `IsProviderFor` supertrait is matched by **identity, not spelling**. Each candidate bound's
resolved `DefId` is checked to be the `IsProviderFor` *defined in the `cgp_component` crate* — the
crate that owns the trait, since `cgp` and `cgp_core` only re-export it and a `pub use` mints no new
`DefId` — so a trait merely named `IsProviderFor` in some unrelated crate never seeds an entry, and
the map is provably rooted in real CGP provider traits. The defining crate name lives in
[`config`](../../crates/cargo-cgp-driver/src/config.rs) as `CGP_COMPONENT_CRATE`. One reach of this
anchor is limited: the string rewrite that consumes the map (below) keys on the marker's *name* pulled
from rendered text, where no `DefId` survives, so two distinct structs that share a marker name would
still collapse to one key — a residual, text-only ambiguity the identity check here cannot close.

The walk is expensive — it visits every trait and its blanket impls — so it runs at most once,
wrapped in a [`ComponentNameMap`](../../crates/cargo-cgp-error-processing/src/rewrite.rs): a
`LazyLock` whose initializer performs the walk on the first lookup and is cached for every lookup
after. The emitter's `emit_diagnostic` runs once per diagnostic, not once per compilation, so this
laziness is what keeps the walk from repeating. And because a lookup happens only when a message
actually parses as a wiring form (the rewrite functions consult the map last, after matching the
message shape), the walk never runs at all for a diagnostic that mentions no CGP wiring — which is why
the emitter needs no separate "is this a CGP diagnostic?" pre-check. The initializer reaches the
`TyCtxt` from thread-local scope (`rustc_middle::ty::tls`), valid because a wiring message is built
during trait solving when a `TyCtxt` is in scope; the driver supplies it as the plain `fn`
[`build_name_map_from_tls`](../../crates/cargo-cgp-driver/src/component_map.rs). Because that
initializer is a `fn` pointer capturing no compiler state, the `ComponentNameMap` type itself carries
no compiler types and lives in the rustc-free crate alongside the rewrite it feeds.

Caching across lookups carries no staleness risk. The map draws only on data that is fixed for the
rest of the compilation once the crate is resolved and lowered — the trait set, the `IsProviderFor`
supertraits, and the blanket impls, none of which the type-checking phase that emits these diagnostics
ever mutates — and it stores owned `String`s rather than `DefId`s or other compiler handles that later
interning or arena churn could invalidate. It is one `TyCtxt` for one driver invocation over one
crate, with no cross-session, incremental database (unlike rust-analyzer's) that could change
underneath the cache between calls.

The rewrite itself is a plain string transform, kept in the rustc-free
[`cargo-cgp-error-processing`](../../crates/cargo-cgp-error-processing) crate (module
[`rewrite`](../../crates/cargo-cgp-error-processing/src/rewrite.rs), the driver's one ordinary
dependency) so it is unit-tested on any toolchain without a `TyCtxt`. Its
entry point, `rewrite_message`, dispatches to the `required for … to implement …` note forms and the
`the trait bound … is not satisfied` header form; each reads the marker out of the trait's generic
arguments and looks the names up in the map. A message whose marker is absent from the map, or any
other message, passes through untouched. A **generic component** — one whose marker carries extra
type parameters, so `CanUseComponent`/`IsProviderFor` gain arguments after the marker (and context) —
is handled in both forms, but differently by design: the descriptive notes name the bare trait and
elide the parameters, while the header *reattaches* them so the bound stays precise. So
`CanUseComponent<AreaCalculatorComponent, f64>` yields the header `` `Rectangle: CanCalculateArea<f64>` ``
and the note "the consumer trait `CanCalculateArea`"; a two-parameter component arrives tuple-grouped
(`(u32, u64)`) and the header unwraps it to `` `CanCalculateArea<u32, u64>` ``. One faithful oddity
follows from naming the obligation's subject verbatim: the subject is usually a provider
(`RectangleArea`, `ScaledArea<RectangleArea>`) but is the context itself when the context stands in as
its own provider, so a self-provider case reads `` the provider `Rectangle` … for the context
`Rectangle` ``. The before/after is pinned across the whole `usability/checks` fixture set, whose
blessed snapshots show the trait-named notes *and* headers;
[`base_area_1`](../../tests/ui/usability/checks/base_area_1.cgp.stderr) is the worked example.

The same emitter seam now hosts a deeper transformation that *replaces* a diagnostic rather than
rewording it. Where the trait-renaming rewrite edits the compiler's diagnostic in place, the
[typed root-cause resolver](typed-root-cause-resolution.md) re-runs the failing check obligation
through the compiler's `InferCtxt` / `ObligationCtxt` API, descends to the `HasField` leaf, and emits
a fresh, root-cause-first diagnostic in place of rustc's cascade — falling back to the in-place rewrite
whenever it cannot fully resolve the cause. That kind of work must happen in the driver, because the
front-end's [processing stage](error-processing.md) is stateless and cannot ask the compiler anything;
it happens in the *emitter* specifically because the natural `after_analysis` hook is unreachable once
the crate has errors (the resolver document explains why).

## Comparison with Clippy

The driver follows `clippy-driver` closely in its skeleton: both detect wrapper mode by testing
whether the second argument's file stem is `rustc` and drop it, both inject `--sysroot` only when one
is absent, and both run the compiler with `rustc_driver::run_compiler` inside `catch_with_exit_code`.
Reading [`external/rust-clippy/src/driver.rs`](../../../external/rust-clippy/src/driver.rs) alongside
this crate, the correspondence maps function for function.

The transformations diverge where the purpose does. Clippy's `config` callback calls `register_lints`
to add lint passes that emit *new* diagnostics; `cargo-cgp`'s `config` callback installs a diagnostic
*emitter* that *rewrites* the compiler's existing notes. That is the same `Callbacks` lever put to the
opposite end — adding versus clarifying — and it is why the driver reads the `TyCtxt` from a custom
emitter where Clippy reads it from a lint pass. The `--sysroot` handling also diverges structurally,
because `cargo-cgp-driver` is out-of-tree while `clippy-driver` ships inside the toolchain; that
difference belongs to the shared environment contract and is covered in
[Executable structure](executable-structure.md#comparison-with-clippy).

The remaining differences are gaps where the driver is deliberately simpler than Clippy today and
will likely grow toward it:

- **Argument reading.** The driver reads `env::args()`, whereas Clippy uses
  `rustc_driver::args::raw_args`, which also expands `@argfile` arguments that cargo passes on some
  platforms (notably Windows, to dodge command-line length limits). Until this is adopted, an
  `@argfile` invocation would not be handled.
- **Driver front-matter.** Clippy's driver installs a logger (`init_rustc_env_logger`) and an ICE
  hook (`install_ice_hook`) with a bug-report URL, and handles `--version`, `--help`, and a
  `--rustc` passthrough. `cargo-cgp` installs none of these yet; a panic in the driver therefore
  surfaces as a plain panic rather than a formatted ICE report.
- **Info-query handling.** Clippy detects cargo's info queries (`-vV`, `--print`) and its
  `--cap-lints=allow` / `--no-deps` cases to *skip* running its lints for them. The driver already
  skips *flag injection* for `-vV` and `--print` (above), and its emitter rewrite is a no-op on the
  diagnostic-free output of an info query, so it needs no further guard today — but one will be needed
  once the callbacks do heavier work that should not run for an info query.
- **Callbacks.** Clippy carries three `Callbacks` implementations (default, rustc-only, and
  lint-registering) and selects among them per invocation. `cargo-cgp` has one `CgpCallbacks`, whose
  `config` hook installs the rewriting emitter; the differentiation will grow as the driver does more
  post-processing.

## Further reading

- [rustc_driver and rustc_interface — Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/rustc-driver/intro.html)
  describes `rustc_driver::run_compiler` and the `Callbacks` trait — the entry point the driver calls
  and the hook `CgpCallbacks` implements.
- [Example: Getting diagnostics — Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/rustc-driver/getting-diagnostics.html)
  walks a minimal `rustc_driver` program that installs a custom emitter through `psess_created`, the
  same hook the trait-renaming emitter uses (note the guide still shows the older `Translate`-based
  emitter API, which the pinned nightly has removed).
- [Tracking issue for crates that are compiler dependencies (#27812) — rust-lang/rust](https://github.com/rust-lang/rust/issues/27812)
  is the `rustc_private` feature's tracking issue, the background for why linking the compiler crates
  needs a nightly toolchain with the `rustc-dev` component.
- [Next-gen trait solving — Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/solve/trait-solving.html)
  — what `-Znext-solver` selects and how it evaluates goals.

## Tests

- [`crates/cargo-cgp-driver/tests/args.rs`](../../crates/cargo-cgp-driver/tests/args.rs) — `rustc_args`
  wrapper-mode stripping, sysroot injection, an existing sysroot left alone, injected-flag appending,
  an explicit `-Znext-solver` override, and the `-vV`/`--print` info-query skips.
- [`crates/cargo-cgp-error-processing/tests/rewrite.rs`](../../crates/cargo-cgp-error-processing/tests/rewrite.rs)
  — the compiler-free rewrite over a hand-built name map, run on any toolchain: both note forms and
  both header forms; generic components (a single parameter reattached to the header, a
  multi-parameter tuple unwrapped, and the notes eliding parameters); the module-prefix and
  generic-subject/context cases; the non-CGP and unknown-marker pass-throughs; and a check that the
  `ComponentNameMap` lazy initializer is *not* forced when no message matches.
- [`tests/ui/usability/unsatisfied-dependency/unsatisfied_dependency.cgp.stderr`](../../tests/ui/usability/unsatisfied-dependency/unsatisfied_dependency.cgp.stderr)
  — pins the un-hidden output the solver switch produces.
- [`tests/ui/usability/checks/base_area_1.cgp.stderr`](../../tests/ui/usability/checks/base_area_1.cgp.stderr)
  — pins the un-elided field name (`--verbose`) and the trait-named header and wiring notes; watch for
  a `_` returning inside its `Symbol`, or a marker-based header/note returning.
- [`tests/ui/usability/checks/generic_area.cgp.stderr`](../../tests/ui/usability/checks/generic_area.cgp.stderr)
  — the end-to-end regression guard that the transform still names the traits when the component is
  generic: the header reattaches the single `<f64>` parameter, the notes name the traits and elide it.
- [`tests/ui/usability/checks/generic_area_multi.cgp.stderr`](../../tests/ui/usability/checks/generic_area_multi.cgp.stderr)
  — the same, for a *three-parameter* component: the header unwraps the `(u32, u64, bool)` tuple to
  `CanCalculateArea<u32, u64, bool>`.
- [`tests/ui/usability/checks/`](../../tests/ui/usability/checks) — the blessed `.cgp.stderr`/`.output.json`
  snapshots across the set pin the trait-renaming transform end to end.

## Source

- [`crates/cargo-cgp-driver/src/run.rs`](../../crates/cargo-cgp-driver/src/run.rs) — the entrypoint:
  reads the sysroot, prepares the argument vector, runs the compiler under `catch_with_exit_code`.
- [`crates/cargo-cgp-driver/src/args.rs`](../../crates/cargo-cgp-driver/src/args.rs) — `rustc_args`:
  wrapper-mode stripping, sysroot injection, flag injection, and the info-query skip.
- [`crates/cargo-cgp-driver/src/config.rs`](../../crates/cargo-cgp-driver/src/config.rs) — the shared
  names: the injected flags (`NEXT_SOLVER_FLAG`, `VERBOSE_FLAG`, `SYSROOT_ENV`) and the
  identity anchor (`CGP_COMPONENT_CRATE`, `IS_PROVIDER_FOR_TRAIT`), each with its rationale.
- [`crates/cargo-cgp-driver/src/callbacks.rs`](../../crates/cargo-cgp-driver/src/callbacks.rs) — the
  `Callbacks` implementation; its `config` hook installs the rewriting emitter.
- [`crates/cargo-cgp-driver/src/emitter.rs`](../../crates/cargo-cgp-driver/src/emitter.rs) — rebuilds
  the default `JsonEmitter`, holds the `ComponentNameMap`, and rewrites messages in place before
  delegating.
- [`crates/cargo-cgp-driver/src/component_map.rs`](../../crates/cargo-cgp-driver/src/component_map.rs)
  — builds the component-marker → trait-names map by inverting the `IsProviderFor` supertrait
  (anchored by `DefId` identity to the `cgp_component` crate) and the consumer-blanket-impl links, and
  exposes `build_name_map_from_tls`, the `TyCtxt`-reading `fn` the `ComponentNameMap` is built with.
- [`crates/cargo-cgp-error-processing/src/rewrite.rs`](../../crates/cargo-cgp-error-processing/src/rewrite.rs)
  — the rustc-free home of the string rewrite (`rewrite_message` and the note/header forms) and the
  lazy `ComponentNameMap`.
- [`crates/cargo-cgp-driver/src/lib.rs`](../../crates/cargo-cgp-driver/src/lib.rs) — the
  `rustc_private` feature gate and the `extern crate` declarations.
