# The driver

`cargo-cgp-driver` is the `rustc` replacement cargo runs for each workspace crate: it executes the
real compiler in-process through `rustc_driver`, and on that foothold it applies the transformations
that make CGP errors readable and renders them like vanilla `rustc`. This document is the deep dive
into how the driver is built — how it prepares the compiler's arguments, how it links the compiler's
internal API, and the transformations it applies to the diagnostics — for an agent reviewing,
debugging, or extending it.

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

A *direct* (non-wrapper) invocation is handled before any of that: `--help`/`-h` or no arguments
prints the driver's help ([`help`](../../crates/cargo-cgp-driver/src/help.rs)) and `--version`/`-V`
prints its version handshake ([`version`](../../crates/cargo-cgp-driver/src/version.rs)), then the
driver exits. These are gated on *not* being in wrapper mode, because in wrapper mode the same flags
belong to the real compiler cargo is probing — so cargo's `rustc --version`/`--help` still reach the
compiler untouched.

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
API to reach the `TyCtxt`. That emitter is where the whole diagnostic layer now lives — the front-end
merely forwards what the driver renders — and it carries eight transformations. Five of them reshape a
specific compiler error into one coded CGP form: a duplicate-key conflict (`E0119`) is
[reshaped into its coded `[CGP-E004]`–`[CGP-E008]` form](#reshaping-a-duplicate-key-conflict), an
orphan-rule namespace registration (`E0210`/`E0117`) is
[reshaped into its `[CGP-E011]` form](#reshaping-an-orphan-rule-namespace-registration), a
capability used in a `#[cgp_fn]`/`#[cgp_impl]` body but not declared via `#[uses(…)]` — an `E0599` on
the generated `__Context__` generic — is reshaped into a `[CGP-E012]` header naming the capability,
with the `#[uses(…)]` fix in a `help` (recovered by
[`resolve::detect_undeclared_capability`](../../crates/cargo-cgp-driver/src/resolve/undeclared.rs) and
worded by the rustc-free `plan_undeclared_capability`), a `#[cgp_impl]` header — or a higher-order
provider's inner-provider bound — naming the wrong trait is
[reshaped into its `[CGP-E013]`/`[CGP-E014]`/`[CGP-E015]` form](#reshaping-a-cgp_impl-provider-definition-mistake),
and a higher-order provider calling an inner provider it never imported with `#[use_provider]` — an
`E0599` on an unbounded type parameter — is reshaped into a `[CGP-E016]` header naming the inner
provider, with the `#[use_provider(…)]` fix in a `help` (recovered by
[`resolve::detect_missing_use_provider`](../../crates/cargo-cgp-driver/src/resolve/missing_use_provider.rs)).
Any
`[T]: Sized` cascade the undeclared-capability case trails is left as
rustc wrote it — those errors can land off the failing expression, where suppressing them reliably
would risk hiding an unrelated error. Otherwise the
deepest transform, the
[typed root-cause resolution](typed-root-cause-resolution.md), *replaces* a
resolvable wiring failure with a root-cause-first diagnostic, covered in its own document; failing
that, the in-place [trait-renaming rewrite](#naming-the-traits-behind-a-component-marker) described
below renames the CGP wiring notes; and finally every diagnostic passes through the
[post-processing](error-processing.md) transforms — stripping CGP path prefixes, resugaring `Symbol!`
and `Path!`, rewording an unmet `HasField` bound — so no raw CGP construct leaks. The sections that follow detail
the two levers, the rename, and the two coherence reshapes; the replacement and the post-processing build on
the rename's `TyCtxt` access and rustc-free helpers, and are documented separately.

The emitter is also what *renders* the diagnostics, the way vanilla `rustc` would. The wrapper type
[`CgpEmitter`](../../crates/cargo-cgp-driver/src/emitter/cgp_emitter.rs) is generic over an inner
emitter, and [`install`](../../crates/cargo-cgp-driver/src/emitter/install.rs) rebuilds whichever
emitter the compiler's own `default_emitter` would build for the active error format — a `JsonEmitter` for `--message-format=json`
(the format cargo uses when a tool asks for JSON), an `AnnotateSnippetEmitter` for the default human
format (what a plain `cargo cgp check` produces) — and wraps it. The emitter transforms the compiler's
`DiagInner` in place before handing it to that inner emitter, so the transform reaches a JSON
diagnostic's structured `children` and its regenerated `rendered` field, and a human diagnostic's
rendered text, alike; because the inner emitter is the compiler's own, the driver's output matches
plain `rustc`'s apart from the CGP transforms.

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
[`acceptable/use-site/unsatisfied_dependency`](../../tests/ui/acceptable/use-site/unsatisfied_dependency.cgp.stderr)
UI snapshot — a fixture that now lives under `acceptable/` precisely because this solver switch, with
the typed resolver on top, turns its once-hidden cause into a clean, root-cause-first error.

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
[`acceptable/fields/base_area_1`](../../tests/ui/acceptable/fields/base_area_1.cgp.stderr) UI snapshot, a fixture
that now lives under `acceptable/` precisely because the flag, with the typed resolver on top, has
turned its once-hidden cause into a clean, root-cause-first error — the same graduation the solver
switch gave `unsatisfied_dependency`.

### Naming the traits behind a component marker

The driver rewrites the compiler's wiring diagnostics to name the consumer and provider traits a
reader thinks in, in place of the internal marker-based phrasing — both the obligation-chain notes and
the primary header the error opens with. Where rustc reports `` required for `RectangleArea` to
implement `IsProviderFor<AreaCalculatorComponent, Rectangle>` `` and `` required for `Rectangle` to
implement `CanUseComponent<AreaCalculatorComponent>` ``, the tool emits `` required for the provider
`RectangleArea` to implement the provider trait `AreaCalculator` for the context `Rectangle` `` and
`` required for the context `Rectangle` to implement the consumer trait `CanCalculateArea` ``. The
primary header is rewritten further, because an unsatisfied wiring bound is an identified CGP error
class and gains its [CGP error code](../error-code.md): `` the trait bound `Rectangle:
CanUseComponent<AreaCalculatorComponent>` is not satisfied `` becomes `` [CGP-E001] the consumer
trait `CanCalculateArea` is not implemented for context `Rectangle` ``, and a provider-side header
`` `RectangleArea: IsProviderFor<AreaCalculatorComponent, Rectangle>` `` becomes `` [CGP-E002] the
provider trait `AreaCalculator` with context `Rectangle` is not implemented for provider
`RectangleArea` `` — restating the fact the marker form stands in for, with the diagnostic's own
Rust code kept. This is the transform the `IsProviderFor` and `CanUseComponent` marker traits
otherwise hide: the component marker names neither trait, its `…Component` suffix is at best an
unreliable guess at the provider trait, and it says nothing at all about the consumer trait.

This is the one transformation that reads the compiler's own state rather than pulling an argument
lever, and that is why it needs everything the driver's in-process access provides. The two flag
levers above change how the compiler *produces* diagnostics; this one edits diagnostics the compiler
has already *built*, using the trait names only a live `TyCtxt` can supply. The driver installs a
custom diagnostic emitter through the callbacks' `config` hook, and that emitter rewrites each
diagnostic in place before handing it to a real inner emitter, so both a JSON diagnostic's `children`
and its regenerated `rendered` text — and a human diagnostic's rendered text — carry the new wording;
cargo then carries the transformed output out and the front-end forwards it untouched.

The inner emitter must be *rebuilt* rather than wrapped. The session's own emitter cannot be reached
to wrap it — `DiagCtxt::set_emitter` only replaces it, with no way to recover the original — so
[`emitter::install`](../../crates/cargo-cgp-driver/src/emitter/install.rs) reads the session options in the
callbacks' `config` hook and, from inside `psess_created`, rebuilds the emitter the compiler's own
`default_emitter` would build for the active error format — a `JsonEmitter` for JSON, an
`AnnotateSnippetEmitter` for the human format — then wraps *that* in the generic
[`CgpEmitter`](../../crates/cargo-cgp-driver/src/emitter/cgp_emitter.rs) and installs the wrapper. The wrapper
forwards every emitter method to the inner emitter unchanged except `emit_diagnostic`, which
transforms first.

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
[`config`](../../crates/cargo-cgp-driver/src/config.rs) as `CGP_COMPONENT_CRATE`.

Each entry is **keyed by the marker's full path** (`def_path_str`), not its bare name, so two distinct
structs sharing a marker name in different modules occupy separate entries. The map's sole consumer is
now the **text-rewrite fallback** below, which has only the marker *name* rendered into the diagnostic
text — where no `DefId` and rarely a full path survives — so it matches a key by its last path segment;
that is ambiguous only when two markers share a name, an unavoidable residual of working from rendered
text. (The [typed root-cause resolver](typed-root-cause-resolution.md) once looked the map up by a
marker's full-path `DefId`, but no longer uses it at all: it reads consumer and provider names straight
off the real trait `DefId`s it walks, part of resolving without depending on `IsProviderFor` — the very
supertrait this map is built by inverting. The map therefore serves only the declined-diagnostic text
path, where recognizing `IsProviderFor`/`CanUseComponent` in rustc's output is still essential.)

The walk is expensive — it visits every trait and its blanket impls — so it runs at most once,
wrapped in a [`ComponentNameMap`](../../crates/cargo-cgp-error-processing/src/rewrite/names.rs): a
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
[`rewrite`](../../crates/cargo-cgp-error-processing/src/rewrite), the driver's one ordinary
dependency) so it is unit-tested on any toolchain without a `TyCtxt`. Its
entry point, `rewrite_message`, dispatches to the `required for … to implement …` note forms
(`rewrite_required_for`), the `the trait bound … is not satisfied` header form
(`rewrite_trait_bound`, which stamps the `[CGP-Exxx]` code), and the
`overflow evaluating the requirement …` header form (`rewrite_wiring_overflow`, which stamps
`[CGP-E010]` on the `E0275` a wiring cycle produces — a `UseContext` delegation whose only
consumer-trait impl is that delegation, so the lookup recurses forever); each reads the marker out
of the trait's generic arguments and looks the names up in the map. When the overflow header is
rewritten, the emitter also drops the note pointing at the generated `__Check…` trait (the kept
caret already covers the check entry) and attaches a `help` naming the usual cause and the two
fixes (`wiring_overflow_help`); the
[`use_context_cycle`](../../tests/ui/acceptable/wiring/constraints/use_context_cycle.rs) fixture
pins the reshaped output. The emitter applies the full dispatch only to a
diagnostic's *main* message and the note rename to its children, since a CGP error code belongs on a
main message alone. A message whose marker is absent from the map, or any other message, passes
through untouched. A **generic component** — one whose marker carries extra
type parameters, so `CanUseComponent`/`IsProviderFor` gain arguments after the marker (and context) —
is handled in both forms, but differently by design: the descriptive notes name the bare trait and
elide the parameters, while the header *reattaches* them so the message stays precise. So
`CanUseComponent<AreaCalculatorComponent, f64>` yields the header `` the consumer trait
`CanCalculateArea<f64>` `` and the note "the consumer trait `CanCalculateArea`"; a two-parameter
component arrives tuple-grouped (`(u32, u64)`) and the header unwraps it to
`` `CanCalculateArea<u32, u64>` ``. One faithful oddity
follows from naming the obligation's subject verbatim: the subject is usually a provider
(`RectangleArea`, `ScaledArea<RectangleArea>`) but is the context itself when the context stands in as
its own provider, so a self-provider case reads `` the provider `Rectangle` … for the context
`Rectangle` ``. The before/after is pinned across the `acceptable/` check fixtures — the `fields/`,
`providers/`, `generic/`, `field-types/`, and `resolution/` subgroups — whose blessed snapshots show
the trait-named notes *and* headers;
[`base_area_1`](../../tests/ui/acceptable/fields/base_area_1.cgp.stderr) is the worked example.

The same emitter seam hosts a deeper transformation that rebuilds a diagnostic's sub-messages rather
than rewording them, and it is tried *before* the rename. Where the trait-renaming rewrite edits the
compiler's text in place, the [typed root-cause resolver](typed-root-cause-resolution.md) re-runs the
failing check obligation through the compiler's `InferCtxt` / `ObligationCtxt` API, descends to each
terminal leaf, and replaces rustc's cascade of sub-notes with one `root cause:` note over the
dependency graph of every leaf (wording the coded main message from typed data where the text lookup
could be ambiguous) — falling back to the in-place rename whenever it cannot fully resolve the cause. That kind
of work must happen in the driver, because it needs the live compiler the front-end never sees; it
happens in the *emitter* specifically because the natural `after_analysis` hook is unreachable once
the crate has errors (the resolver document explains why). Whichever of the two produced the
diagnostic, it then passes through the rustc-free [post-processing](error-processing.md) cleanup, so
the type names either transform embeds are stripped of CGP path prefixes and resugared.

The resolver is not limited to diagnostics worded in CGP terms. A failure that *never names a CGP
construct* can still be a consequence of a CGP component failing — a hand-written `Send`-recovery
wrapper whose `async fn` forwards to a wired method fails with an `E0271` opaque-future mismatch, a
downstream trait bound needs a method the context cannot supply — so the emitter offers the resolver
every method `E0599`, `E0271`, and `E0277` (not only the ones mentioning a wiring trait). The resolver
anchors such an error on the enclosing hand-written `impl` (whose supertrait is a CGP consumer trait)
and traces the dependency chain from there; if the chain reaches a CGP root cause it renders the tree,
and if it does not it declines and the error passes through untouched. A traced `E0271` whose cause is
*not* a projection mismatch (its opaque `type mismatch resolving …` message being unreadable) is given
the `[CGP-E001]` consumer header, since it is really the consumer trait failing to be implemented; one
that *is* takes the matching mismatch header — `[CGP-E003]` for a `HasField` value type, `[CGP-E017]`
for any other associated type.

A final gate de-duplicates the transformed diagnostics *across* the compilation, because CGP wiring is
lazy and so one mistake surfaces the same error at many sites. A missing dependency is reported at the
`check_components!` entry, again at every hand-written `impl` that references the broken consumer, and
again at each call — the transfer example's single un-wired password type produced eighteen identical
root-cause trees this way. The emitter records each transformed diagnostic in a
[`DedupLedger`](../../crates/cargo-cgp-error-processing/src/dedup.rs) — the rustc-free ledger that
owns the key scheme, so it is unit-tested without a compiler — and suppresses any later diagnostic
whose **span-independent signature** the ledger has seen. A resolved diagnostic is keyed by its
recovered cause — the context, the failing consumer
trait(s), and each root-cause leaf, via the rustc-free `cause_signature` — so the *same* consumer's
failure re-reported at several spans collapses to one, while two *distinct* consumers that happen to
share a cause keep separate signatures, so each survives de-duplication (no capability's failure is
ever hidden) to be *coalesced* at flush, described next. A diagnostic the resolver declined but the text rewrite still transformed is keyed by its
rendered message text instead (`message_signature`), so the fallback re-reports coalesce too. A third
key is the **coded main-message header**: a failure the resolver declined but still rewrote (falling
back to raw `IsProviderFor` scaffolding) carries the *same* `[CGP-Exxx]` header as the resolved tree of
the same failure, so keying on the header collapses that declined fallback into the resolved
occurrence even though their bodies differ. Only the tool's own transformed diagnostics are
de-duplicated; an untouched `rustc` error always passes through. The count stays honest because cargo
re-counts the diagnostics the emitter actually produces — a suppressed re-report drops out of its "N
errors" summary as well — so the visible block count and the summary agree. This is the
[one-mistake-many-errors](../issues/usability.md) usability class the per-diagnostic resolver could not
address on its own.

De-duplication collapses the *same* consumer's re-reports; a second gate, **coalescing**, collapses
*different* consumers that share one root cause into a single block — the transfer example's one
missing field breaks several endpoints, and a chain of dependent components fails top to bottom for
the one field the innermost provider needs. Listing every affected consumer in one headline is only
possible once they have all arrived, and the `Emitter` trait has no end-of-compilation hook, so the
emitter does not emit as it goes: it **holds every diagnostic in an arrival-ordered buffer** and
flushes it from its `Drop`. `Drop` runs during the `DiagCtxt`'s own teardown, after every diagnostic
has been handed to the emitter but while the inner emitter — a field of the wrapper, dropped only
afterward — is still alive to render; the diagnostics were counted by the `DiagCtxt` as they arrived,
so deferring their *rendering* to `Drop` leaves the error count untouched. At flush, each buffered
consumer-trait failure (`is_consumer_shaped` — a CGP consumer on the checked context, not a field
mismatch, provider check, or foreign wrapper) is grouped with every failure it **shares a root cause**
with, by the rustc-free `group_by_shared_cause` over the consumer-independent per-cause `cause_keys`.
Grouping on a shared cause rather than on one whole-failure key is what keeps a single mistake in a
single block, because the depths one mistake surfaces at each see a different *subset* of its causes: a
`check_components!` entry stops at the first unmet leaf on its own branch, while a use-site call walks
every wired component and reaches them all. Those subsets overlap without ever being equal, so an
exact-match key grouped none of them — and the block whose causes were the *union* of the others fared
worst, since every one of its roots had already been drawn and its chain was then
[fully elided](dependency-graph-rendering.md#eliding-across-blocks) into a bare `root causes:` list
with no chain at all. The relation is made transitive so it is a partition rather than an ambiguous
overlap graph, which also earns an invariant the exact key could not: **no two coalesced blocks share a
root cause**. A group of one emits its own
per-entry block unchanged; a group of several emits **one merged block** — a `[CGP-E001]` header
listing every affected consumer trait (`consumer_header` over a synthesized `Resolved` whose
`consumers` is the union), a caret at each failing entry, and one root-cause note built by folding
every member's paths into a single [dependency graph](dependency-graph-rendering.md). Because the
members were grouped for *sharing* a cause, the union of their causes holds one copy of it per
member, so the block folds those copies back into one through the rustc-free `merge_causes_by_leaf`
before wording anything — otherwise the same mistake is stated once per member, most visibly as an
underived field listed once per consumer that reads it. The graph does
the merging: when one failing consumer transitively depends on another — `CanCalculateDensity` needs
`CanCalculateArea`, so its chain *contains* the other's — the contained consumer is not a top-level
root and the block leads with the subsuming chain; when two consumers are independent but share the
cause, both chains render, converging at the shared node. A
member rustc happened to surface provider-side (a `[CGP-E002]` header as a lone diagnostic) is worded
uniformly as a consumer in the merged block, since a `check_components!` entry failing *is* the
consumer trait failing. An untouched `rustc` error, a conflict, or an orphan reshape is buffered as a
plain entry and emitted verbatim at its arrival position, so global ordering (a CGP block beside an
unrelated `E0308`, say) is preserved.

The flush is also where every resolution's `root cause:` note is *rendered*, not merely where the
coalesced ones are merged, and that is what lets the notes elide against one another. A resolution
that does not coalesce still buffers its causes rather than its note — a wrapper trait, a mismatch, a
provider-side check — because only at flush is the emission order known, and one
[dependency graph](dependency-graph-rendering.md#eliding-across-blocks) `seen` set threaded through
the blocks in that order lets a later one `(*)`-truncate at the subtree an earlier one already drew —
while still ending at the root cause, since the block it points into is one the reader may not have to
hand.
The redundancy this removes is large in real code: CGP wiring is lazy, so one mistake reaches several
diagnostics that legitimately do *not* de-duplicate, and their chains can share everything below their
own first few hops. A block whose whole chain was already drawn keeps its lead and drops the chain
entirely rather than heading a lone `(*)`. Either way the block stays actionable alone — its header,
its fix `help`, and its `root cause:` lead all still name the cause, so only chain *detail* is elided.
The
[`density_3`](../../tests/ui/acceptable/duplication/density_3.rs) (two components, one missing field),
[`dependency_cascade`](../../tests/ui/acceptable/duplication/dependency_cascade.rs) (three chained
providers), and [`missing_normal_bound`](../../tests/ui/acceptable/wiring/missing-wiring/missing_normal_bound.rs)
(two consumers sharing an `App: Clone` bound) fixtures pin the merged blocks where one consumer's
chain *subsumes* another's, so the deepest leads;
[`parallel_consumers`](../../tests/ui/acceptable/duplication/parallel_consumers.rs) pins the
complementary shape — two *independent* consumers reaching one cause by equal-depth chains that
neither subsumes — where the representative is the first check entry deterministically.

A declined consumer-method `E0599` gets one further cleanup before the fallback rewrite runs. rustc's
method probe, meeting CGP's `self`-less provider methods, frames the failure as a call-syntax mistake
— a "this is an associated function, not a method" caret label, a "found the following associated
functions …" note, and a "use associated function syntax instead" suggestion that is actively wrong —
while the real cause, the unmet wiring bound, sits in a later note. For a method-bounds `E0599` that
mentions a CGP wiring trait, the emitter strips that method-probe advice
(`strip_method_probe_advice`, over the phrasings
[`is_method_probe_advice_text`](../../crates/cargo-cgp-error-processing/src/signals.rs) recognizes),
so the wiring bound is the first note a reader meets. The
[`generic_consumer_unwritten_arg`](../../tests/ui/acceptable/use-site/generic_consumer_unwritten_arg.rs)
fixture — a use-site failure whose dispatch parameter no anchor can recover, so the resolver always
declines — pins the cleaned output.

The same span-keyed gate also drops a **downstream `?`-operator cascade** of a wiring failure. When a
consumer-method call fails and its result is consumed with `?` (`app.handle(…).await?`), the type of
`expr?` becomes unresolvable, so rustc adds a `Try` / `FromResidual` error on the same expression that
merely restates the wiring failure and dumps the unresolved `<Ctx as Consumer<…>>::Output` projection.
The emitter records the primary span of every CGP failure it recognizes (`cgp_spans`, populated before
de-duplication so even a dropped re-report still anchors its cascade) and suppresses a later, *untouched*
diagnostic that both looks like a `?`-operator error (`is_question_mark_cascade`, matching rustc's stable
"the `` ? `` operator can only …" wording) and whose span overlaps a recorded failure. rustc emits the
wiring failure before its cascade, so the span is always recorded in time. The scope is deliberately
tight — only a `?` error sitting on an expression where a CGP wiring error was already shown is dropped,
never a `?` misuse elsewhere — which is what makes suppressing an otherwise-untouched `rustc` error
sound. This removed the two projection-dumping cascade blocks a `Code`-dispatched use-site failure
used to trail (the
[call-site anchor](typed-resolution-call-site.md) has
since taken the failure itself from three fallback blocks to one resolved block); the pinning
fixture is
[`cascade_after_use_site`](../../tests/ui/acceptable/use-site/cascade_after_use_site.rs).

### Reshaping a duplicate-key conflict

The emitter's fourth transform runs *before* the other two and handles a different failure kind: the
coherence conflict (`E0119`) a duplicate wiring entry produces. A duplicate `delegate_components!`
key makes the expansion emit two overlapping `DelegateComponent` impls, and because each
context-wiring entry
also generates an `IsProviderFor` forwarding impl, the compiler reports the same conflict *twice* —
once keyed on `DelegateComponent<…>` and once on `IsProviderFor<…>`, both internal traits the user
never wrote. The transform recognizes the pair as one logical mistake: it **drops** the redundant
`IsProviderFor` diagnostic (emitting nothing) and **rewrites** the `DelegateComponent` one into a
coded headline that names the colliding key(s), keeping rustc's two carets — "first implementation
here" and "conflicting implementation" — which already point at the two entries. The code is one of
`[CGP-E004]`–`[CGP-E008]`, chosen by the shape of the collision.

The `IsProviderFor` suppression is gated on its **companion conflict being confirmed**, because
suppressing a lone `E0119` would leave a failing build with no error at all. Every generated
`IsProviderFor` impl rides alongside another impl from the same macro invocation — the
`DelegateComponent` entry impl of a wiring entry, or the provider-trait impl of a `#[cgp_provider]`
block — and the two conflict exactly when their `IsProviderFor` copies do. So the classifier
suppresses only when a genuine local `IsProviderFor` impl sits at the caret *and* the two colliding
sites each carry a local impl of one *common* other trait, whose own `E0119` rustc then reports.
That confirmation is what extends the suppression beyond delegate pairs to a duplicate provider
*name* — two `#[cgp_impl(new P)]` blocks, whose `E0428` and provider-trait `E0119` remain while the
redundant `IsProviderFor` block is dropped
([`duplicate_provider_name`](../../tests/ui/acceptable/wiring/duplicate-keys/duplicate_provider_name.rs))
— while an `IsProviderFor` conflict with no common companion is left alone.

Everything is recovered from the compiler, not the error text. rustc aims the `E0119` at
`tcx.def_span` of the conflicting impl, and the delegate macro re-spans each entry onto its key token,
so [`resolve::conflict`](../../crates/cargo-cgp-driver/src/resolve/conflict) matches the caret to
that `DelegateComponent` impl (by source range, since the two halves of the pair carry the same range
under different `SyntaxContext`s) and reads the entry off it — its context, its key, and its
`Delegate`. It does the same for the "first implementation here" impl at the diagnostic's other
labelled span, then classifies the shape into one of the five, each with its own code: a
**duplicate** (`CGP-E004`, the two keys render equal), an **overlap** (`CGP-E005`, distinct but
colliding keys — a generic over a specific, or a path prefixing another), **multiple namespaces**
(`CGP-E006`, both keys are blanket forwardings, so the context joins more than one namespace), a
**redirect** collision (`CGP-E007`), or a **duplicate redirect** (`CGP-E008`, both entries redirect).
A redirect collision is detected two ways: one entry's `Delegate` is literally a `RedirectLookup`
(an `open` or `=>` redirect), *or* one entry is a blanket namespace and the other a concrete key the
namespace maps to a `RedirectLookup` — recovered by normalizing `<key as Namespace<Ctx>>::Delegate`
through the trait solver (the same re-entrant normalization the typed resolver uses). Its header
names only the redirected path; the *fix* — wire the direct entry's provider under that key — rides
in a separate `help`, kept out of the header so the headline stays one short sentence. Each
key is rendered to its surface form off the types — a component marker to its name, a `PathCons<…>`
to its bare `@…` path (a generic tail or `for`-loop key collapsing to `.*`), a blanket forwarding to
the namespace/table trait that keys it — so the headline names what the programmer wrote. The message wording is decided
by the rustc-free [`plan_wiring_conflict`](../../crates/cargo-cgp-error-processing/src/diagnosis/wiring.rs)
(and `wiring_conflict_help` for the redirect fix)
over the owned `WiringConflict` the classifier fills in, so it is unit-tested without a compiler.

An `E0119` naming *neither* internal trait can still be a wiring conflict: a duplicate entry inside
a `cgp_namespace!` block conflicts on the user's own namespace trait
(`conflicting implementations of trait \`MyNamespace<_>\` for type \`PathCons<…>\``). The
classifier's namespace route recognizes that shape by the impls at the carets — a local impl of a
[namespace lookup trait](typed-root-cause-resolution.md) (the single-`Delegate` fingerprint) whose
`Self` is the entry's `@`-path — and words it through the same `WiringConflict` shapes, with the
namespace trait as the subject: two `=>` entries on one path become a `[CGP-E008]` duplicate
redirect naming both targets
([`namespace_duplicate_path_key`](../../tests/ui/acceptable/wiring/namespace-paths/namespace_duplicate_path_key.rs)).
A namespace conflict whose entry key is not a `PathCons` path (an inherited-override collision on a
bare marker, say) is deliberately left to the fallback, where rustc's own header already names the
namespace and the key.

The transform is anchored to the genuine CGP traits (by `DefId`, like the rest of the
resolver), so a same-named trait cannot drive it, and it declines a conflict whose carets carry
none of the recognized impls, leaving it to the fallback. The
blessed snapshots under [`acceptable/wiring/`](../../tests/ui/acceptable/wiring) pin each shape.

### Reshaping an orphan-rule namespace registration

The emitter's fifth transform handles the other coherence-class error, the orphan-rule violation
(`E0210`, or its sibling `E0117`) a namespace registration produces when the crate owns neither end.
Registering wiring into a namespace lowers to `impl<Param> Namespace<Param> for Key`, and Rust's
orphan rule accepts a foreign-trait impl only when a local type covers its parameters. When *both*
the namespace trait and the key are foreign — a downstream crate registering into an upstream
namespace it does not own, keyed on an upstream component it does not own either — nothing is local,
and the compiler rejects it, naming the machinery parameter (`__Components__` from a
`#[default_impl]`/`#[prefix]`, `__Table__` from a `cgp_namespace!` re-open) and framing a CGP wiring
decision as a bare coherence rule. The transform **rewrites** that headline into the coded
`[CGP-E011]` form, naming the foreign namespace and the key the programmer wrote, re-aims the caret at
the offending macro alone (dropping the "uncovered type parameter" label, which no longer applies),
and adds a `help` carrying the ownership-based fix the raw error never states.

Everything is recovered from the compiler, not the error text — the reverse of relying on the
`__Components__`/`__Table__` names the message happens to print. A cheap text pre-filter
([`mentions_orphan_param_text`](../../crates/cargo-cgp-error-processing/src/signals.rs)) gates the
scan on the `E0210` naming such a machinery parameter, then
[`resolve::orphan`](../../crates/cargo-cgp-driver/src/resolve/orphan.rs) finds the offending impl: a
local impl of a *foreign* [namespace lookup trait](typed-root-cause-resolution.md) (the
single-`Delegate` fingerprint, so a downstream crate's own namespace trait is recognized like CGP's
built-in `DefaultNamespace`) for a *foreign* key. Because rustc emits an `E0210`/`E0117` *only* for a
genuine orphan, matching the caret to that impl (by source range) is the whole confirmation; a lone
candidate is used even when the range match misses, and any ambiguity declines to the fallback. The
key renders to its surface form off the types — a component marker to its name, a `PathCons<…>` to its
bare `@…` path — through the same `describe_key` the conflict classifier uses, and the trigger (which
fix to word) is read from the impl's own machinery parameter name (`__Table__` for a re-open,
`__Components__` for a registration), a reliable `DefId`-independent discriminator. The message wording
is decided by the rustc-free
[`plan_orphan_conflict`](../../crates/cargo-cgp-error-processing/src/diagnosis/orphan.rs) (and
`orphan_conflict_help` for the fix) over the owned `OrphanConflict` the classifier fills in, so it is
unit-tested without a compiler. The blessed snapshots under
[`acceptable/wiring/orphan/`](../../tests/ui/acceptable/wiring/orphan) pin the component-key,
path-key, and re-open shapes.

### Reshaping a `#[cgp_impl]` provider-definition mistake

The emitter's remaining reshapes handle macro-lowering mistakes rather than coherence ones — the ways
a `#[cgp_impl]` provider names the wrong trait, or forgets to import one. `#[cgp_impl(new P)] impl AreaCalculator`
is the idiomatic provider form — the macro turns the header inside out into
`impl<__Context__> AreaCalculator<__Context__> for P`, inserting the context as the leading generic.
Naming the component's *consumer* trait `CanCalculateArea` there instead, or a trait that is not a
CGP component at all, makes the macro generate an inside-out impl of the wrong trait and reference a
`…Component` marker that does not exist. One mistake then produces a burst of cryptic errors — `E0425`
(the missing marker), `E0107` (the wrong trait given one generic argument too many, since the inserted
context always exceeds a consumer trait's arity), `E0186` (`&self` mismatch), `E0207` (`__Context__`
unconstrained) — plus a downstream check failure, none naming the real cause. The transform
**rewrites** the `E0107` — whose caret already sits on the misused trait name — into a `[CGP-E013]`
(consumer trait) or `[CGP-E014]` (non-CGP trait) header with the fix in a `help`, and **suppresses**
the rest of the cascade so one clean error remains.

The same reshape covers a higher-order provider's **inner-provider bound** naming the consumer trait,
typically through `#[use_provider]`, which fills the leading context argument in — so
`#[use_provider(Inner: CanCalculateArea)]` generates `Inner: CanCalculateArea<Self>`, the consumer
trait again given one argument too many. That `E0107` on the bound is reshaped into a `[CGP-E015]`
header, and the `E0308` body cascade the malformed bound trails — told from a user's own type error by
its mention of the generated `__Context__` — is suppressed alongside it.

The mistake is recovered off the compiler by
[`resolve::detect_cgp_impl_misuses`](../../crates/cargo-cgp-driver/src/resolve/cgp_impl_misuse.rs),
never from error text, using the consumer- and provider-trait fingerprints. Three structural
conditions select the *user's* `#[cgp_impl]` impl and exclude every blanket and forwarding impl the
CGP macros generate (which also carry `__Context__`): the impl has the `#[cgp_impl]`-inserted
`__Context__` generic; its `Self` is a concrete local struct/enum (the provider struct), where the
generated consumer/provider blanket impls have a bare type-parameter `Self`; and its header trait
reference is a token the user wrote (not from a macro expansion), where the generated
`IsProviderFor`/`DelegateComponent` forwarding impls carry a synthesized reference. The header trait
is then classified by [`consumer_provider_trait`](../../crates/cargo-cgp-driver/src/resolve/cgp_item.rs):
a consumer trait yields the provider trait to suggest (`[CGP-E013]`), a provider trait is the correct
target and is skipped, and anything else is a non-CGP trait (`[CGP-E014]`); each of the impl's
inner-provider `where`-bounds is scanned the same way for a consumer trait (`[CGP-E015]`). Detection
is triggered
only by the `E0107` — a type-lowering-phase error, always present for this mistake — because it forces
HIR and trait-graph queries that would re-enter the `DiagCtxt` lock and abort the compiler if run
while the earlier-phase `E0425` (emitted mid-name-resolution) is being handled, the
[re-entrant-emission hazard](rustc-diagnostic-internals.md#re-entering-the-diagnostic-context-lock-was-already-held).
The sibling `E0425`/`E0186`/`E0207`/`E0308` are suppressed by matching their spans against the impl body and
the `__Context__` parameter's call-site (a sibling arriving before the `E0107` is purged from the
buffer at reshape time), the downstream `NotAProvider` check re-report by matching its resolved leaf
against the offending provider struct, and rustc's trailing `rustc --explain` footer is rebuilt so it
does not list the suppressed ones. That footer rebuild is not particular to this reshape: rustc builds
the footer in `print_error_count` from every code it *registered*, which is a superset of what the
emitter emits once it has suppressed a re-report, a `?`-cascade, or this sibling burst, and once it has
*merged* the failures sharing a cause into one block carrying the first member's code. So
`rebuild_explain_footers` runs over the whole flushed list, keeping the footer's own codes filtered by
what survived — an intersection rather than a fresh set, since rustc lists only codes that *have* an
extended explanation and second-guessing that would offer `--explain` for a code with no such text.
A footer whose codes all survive is rewritten identically, so untouched output is unaffected. The message wording is decided by the
rustc-free
[`plan_cgp_impl_misuse`](../../crates/cargo-cgp-error-processing/src/diagnosis/cgp_impl_misuse.rs)
(and `cgp_impl_misuse_help` for the fix) over the owned `CgpImplMisuse` the detector fills in, so it
is unit-tested without a compiler. The blessed snapshots under
[`acceptable/lowering/`](../../tests/ui/acceptable/lowering) pin the consumer-trait, generic-component,
non-CGP-trait, and inner-bound shapes.

The reshape is deliberately specific to `#[cgp_impl]`, keyed on the `__Context__` marker it inserts.
The lower-level `#[cgp_provider]`/`#[cgp_new_provider]` forms — a hand-spelled inside-out impl with a
user-named context — are not covered, and cannot be safely: without that reserved marker,
`impl<Ctx> SomeConsumer<Ctx> for ConcreteType` cannot be told from a legitimate direct impl of a
generic consumer trait on a context, so recognizing it would risk a false positive on valid code.
`#[cgp_impl]` is the idiomatic provider form, so this is the case a programmer overwhelmingly hits.

A last reshape handles the mirror mistake: a higher-order provider that calls an inner provider it
never imported. The body invokes the inner provider as an associated function —
`InnerCalculator::area(self)` — which needs the parameter bounded as `InnerCalculator: AreaCalculator<Self>`,
declared through `#[use_provider(InnerCalculator: AreaCalculator)]`. Forgetting the import leaves the
parameter unbounded, so rustc reports a vague `E0599` — "no associated function `area` found for type
parameter `InnerCalculator`" — whose suggestion leaks the generated `__Context__` and offers the
*consumer* trait as a bound, the wrong fix for a higher-order provider. The transform rewrites it into
a `[CGP-E016]` header naming the inner provider, with the `#[use_provider(…)]` fix in a `help`,
recovered by
[`resolve::detect_missing_use_provider`](../../crates/cargo-cgp-driver/src/resolve/missing_use_provider.rs):
the failing call is an associated-function call `Param::method(…)` on a generic parameter of an
enclosing provider-trait impl, the method belongs to a CGP provider trait, and the parameter is not
already bounded by it. It is gated to the "item on an unbounded type parameter" `E0599` shape —
reported during typeck of the calling body, where the detector's queries are cached, unlike the
resolution-class `E0599` emitted mid-`predicates_of` — recognized by its plain-string help
(`the type parameter is bounded by the trait`), since the `E0599`'s main message is a Fluent
(non-string) message. Its wording is the rustc-free `plan_missing_use_provider` /
`missing_use_provider_help` over the owned `MissingUseProvider`.

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
  surfaces as a plain panic rather than a formatted ICE report. The panics most within the tool's
  power to cause come from the emitter re-running compiler code, and the ways to avoid them are
  catalogued in
  [rustc diagnostic internals](rustc-diagnostic-internals.md#panic-hazards-running-compiler-code-inside-the-emitter).
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
- [`crates/cargo-cgp-error-processing/tests/postprocess.rs`](../../crates/cargo-cgp-error-processing/tests/postprocess.rs)
  — the compiler-free post-processing transforms the emitter applies as its final pass: prefix
  stripping, the `Symbol!` and `Path!` resugaring, and the missing-field reword branches.
- [`tests/ui/acceptable/use-site/unsatisfied_dependency.cgp.stderr`](../../tests/ui/acceptable/use-site/unsatisfied_dependency.cgp.stderr)
  — pins the un-hidden output the solver switch produces.
- [`tests/ui/acceptable/fields/base_area_1.cgp.stderr`](../../tests/ui/acceptable/fields/base_area_1.cgp.stderr)
  — pins the un-elided field name (`--verbose`) and the trait-named header and wiring notes; watch for
  a `_` returning inside its `Symbol`, or a marker-based header/note returning.
- [`tests/ui/acceptable/generic/generic_area.cgp.stderr`](../../tests/ui/acceptable/generic/generic_area.cgp.stderr)
  — the end-to-end regression guard that the transform still names the traits when the component is
  generic: the header reattaches the single `<f64>` parameter, the notes name the traits and elide it.
- [`tests/ui/acceptable/generic/generic_area_multi.cgp.stderr`](../../tests/ui/acceptable/generic/generic_area_multi.cgp.stderr)
  — the same, for a *three-parameter* component: the header unwraps the `(u32, u64, bool)` tuple to
  `CanCalculateArea<u32, u64, bool>`.
- [`tests/ui/acceptable/`](../../tests/ui/acceptable) — the blessed `.cgp.stderr` snapshots across the
  check subgroups (`fields/`, `providers/`, `generic/`, `field-types/`, `resolution/`) pin the
  trait-renaming transform end to end.

## Source

- [`crates/cargo-cgp-driver/src/run.rs`](../../crates/cargo-cgp-driver/src/run.rs) — the entrypoint:
  reads the sysroot, prepares the argument vector, runs the compiler under `catch_with_exit_code`.
- [`crates/cargo-cgp-driver/src/args.rs`](../../crates/cargo-cgp-driver/src/args.rs) — `rustc_args`:
  wrapper-mode stripping, sysroot injection, flag injection, and the info-query skip.
- [`crates/cargo-cgp-driver/src/config.rs`](../../crates/cargo-cgp-driver/src/config.rs) — the shared
  names: the injected flags (`NEXT_SOLVER_FLAG`, `VERBOSE_FLAG`, `SYSROOT_ENV`) and the
  identity anchor (`CGP_COMPONENT_CRATE`, `IS_PROVIDER_FOR_TRAIT`), each with its rationale.
- [`crates/cargo-cgp-driver/src/callbacks.rs`](../../crates/cargo-cgp-driver/src/callbacks.rs) — the
  `Callbacks` implementation; its `config` hook installs the transforming emitter.
- [`crates/cargo-cgp-driver/src/emitter/`](../../crates/cargo-cgp-driver/src/emitter) — the generic
  `CgpEmitter<E>`, split behind a re-exporting `mod.rs`: `install.rs` rebuilds the compiler's default
  emitter for the active format (a `JsonEmitter` or an `AnnotateSnippetEmitter`) and wraps it,
  `cgp_emitter.rs` holds the `CgpEmitter<E>` type (holding the `ComponentNameMap`, the rustc-free
  `DedupLedger` of already-emitted failures, the `cgp_spans` list of recognized-failure spans that
  anchors the `?`-operator cascade suppression, and the arrival-ordered `buffer` of `BufEntry`s
  flushed from `Drop` — `Plain` verbatim in place, `Resolved` having its `root cause:` note rendered
  there against the flush's shared `seen` set, and the `coalescible` ones additionally partitioned by
  `group_by_shared_cause` and merged into one consumer header per group by `merged_diag`, after which
  `rebuild_explain_footers` makes the trailing `rustc --explain` footer name only the codes that
  survived) and its transform/post-process/de-duplicate/coalesce orchestration, and
  `edit.rs` holds the `DiagInner`-editing helpers (including `message_signature`, the span-independent
  text key for de-duplicating a declined-but-rewritten diagnostic; `strip_method_probe_advice`, the
  drop of rustc's associated-function framing on a declined consumer-method `E0599`; and
  `is_question_mark_cascade`, the `Try`/`FromResidual`-shape recognizer for the cascade drop — the
  text phrasings all three key on live in the rustc-free
  [`signals`](../../crates/cargo-cgp-error-processing/src/signals.rs) module).
- [`crates/cargo-cgp-driver/src/component_map.rs`](../../crates/cargo-cgp-driver/src/component_map.rs)
  — builds the component-marker → trait-names map by inverting the `IsProviderFor` supertrait
  (anchored by `DefId` identity to the `cgp_component` crate) and the consumer-blanket-impl links, and
  exposes `build_name_map_from_tls`, the `TyCtxt`-reading `fn` the `ComponentNameMap` is built with.
- [`crates/cargo-cgp-error-processing/src/rewrite/`](../../crates/cargo-cgp-error-processing/src/rewrite)
  — the rustc-free home of the string rewrite (`message.rs`'s `rewrite_message` and the note/header
  forms) and the lazy `ComponentNameMap` (`names.rs`).
- [`crates/cargo-cgp-driver/src/lib.rs`](../../crates/cargo-cgp-driver/src/lib.rs) — the
  `rustc_private` feature gate and the `extern crate` declarations.
