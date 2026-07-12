# Typed root-cause resolution

The driver can transform a CGP check-failure diagnostic into the root-cause **dependency tree** it
recovers by *asking the compiler* rather than by reading the error text. This document describes that
resolver: what it transforms, why it runs where it does, how it recovers the chain through the trait
solver, and the boundaries that decide when it steps aside and lets the older text-rewrite path
handle the diagnostic instead.

This is the second, deeper transformation the driver's emitter performs. The first —
[naming the traits behind a component marker](driver.md#naming-the-traits-behind-a-component-marker) —
edits the compiler's diagnostic in place, renaming its wording. The resolver goes further: it walks
the wiring's typed obligations to the real root cause and renders the whole chain as a `cargo tree`,
rewriting the main message into its coded CGP form when it is an identified CGP class and replacing
the sub-notes with one `root cause:` note per leaf. It realizes the compiler-state enrichment that
[The driver](driver.md) and [The error pipeline](error-pipeline.md) anticipated.

## What it transforms, and what it leaves alone

The resolver considers **any diagnostic that names a CGP wiring or field trait**
(`CanUseComponent`, `IsProviderFor`, or `HasField`) and **every method `E0599`** — no longer only an
`E0277`. It recovers a starting obligation two ways: from a `check_components!` entry when the caret
sits on one, and otherwise from the *use site* of a broken consumer-method call (below). Either way it
walks the wiring obligations down to the terminal unmet bound(s) they rest on, and the diagnostic is
then transformed in two independent halves.

**The main message is rewritten — and stamped with its [CGP error code](../error-code.md) — only when
it is an identified CGP class.** An unsatisfied `CanUseComponent` bound is a
[check-trait failure](../../../cgp/docs/errors/checks/check-trait-failure.md), so it becomes
`[CGP-E001] the consumer trait \`CanCalculateArea\` is not implemented for context \`Rectangle\``,
worded from the typed resolution (whose full-path marker keys make the consumer name exact even for
same-named components); a consumer-method `E0599`, whose text names no wiring trait, gets the same
`CGP-E001` form from the resolution; and an unsatisfied `IsProviderFor` bound becomes the `[CGP-E002]`
provider form via the text rewrite. The rewrite restates the same fact readably — the caret is re-aimed
at the failing entry alone, and the diagnostic's own Rust code (`E0277`, `E0599`) is kept. A main
message that is *not* a CGP class — an ordinary bound (`f64: Eq`) the next-gen solver already descended
to — stays rustc's own, header, labels, and caret untouched
([`ordinary_bound_unsatisfied`](../../tests/ui/usability/checks/ordinary_bound_unsatisfied.rs)).

**The sub-messages are replaced either way.** rustc's obligation-chain notes, supplementary help, and
structured suggestions are discarded, and each recovered root cause becomes one `= note:` opening with
a `root cause:` lead naming its leaf, followed by the dependency chain rendered as a tree. The lead is
worded by *why* the leaf is unmet, which the resolver decides by inspecting the actual struct a
`HasField` bound lands on (detailed under
[How the root cause is recovered](#how-the-root-cause-is-recovered)): a genuinely absent field reads as
`root cause: missing field \`height\` on \`Rectangle\`` (no `context` qualifier, since `HasField` can
land on any struct), while a field the struct *does* carry but has not derived reads as `root cause:
accessor trait \`HasField\` with field \`name\` is not implemented for \`Person\``, with a separate
`help` naming the fix — `make sure that \`#[derive(HasField)]\` is used for \`Person\`` (pointed at the
`Deref` target when the field is only reachable through one; that is exactly the
[`missing_has_field_derive` fixture](../../tests/ui/usability/checks/missing_has_field_derive.rs), the
present-but-underived case a plain "missing field" would misdescribe). Any other leaf restates its
bound — `root cause: the trait bound \`f64: std::cmp::Eq\` is not satisfied` — except when the kept
main message already states that very bound, where the lead would only repeat the header and the note
carries the chain alone
([`ordinary_bound_unsatisfied`](../../tests/ui/usability/checks/ordinary_bound_unsatisfied.rs) versus
[`unregistered_prefix_path`](../../tests/ui/usability/checks/unregistered_prefix_path.rs), whose
rewritten `CGP-E002` header names the `RedirectLookup` step, not the `DefaultNamespace` leaf).

**A use-site failure is handled the same way, once its obligation is recovered.** CGP wiring is lazy,
so a broken provider dependency often surfaces not at a check but where the consumer method is *called*
— `person.greet()` on a `Person` that cannot satisfy `HasName` — as an `E0599` "method exists but its
trait bounds were not satisfied". There is no check impl to anchor on, so the resolver instead recovers
the context type from the diagnostic's own spans (the "method not found for this struct" span lands on
`Person`'s definition) and re-checks every component that context wires. So
[`missing_dependency`](../../tests/ui/usability/unsatisfied-dependency/missing_dependency.rs) and
[`unsatisfied_dependency`](../../tests/ui/usability/unsatisfied-dependency/unsatisfied_dependency.rs)
become `[CGP-E001] the consumer trait \`CanGreet\` is not implemented for context \`Person\`` (the
code stays `E0599`, since the error is still rustc's) over a `missing field` root-cause note — and the
misleading "this is an associated function… use associated function syntax instead" advice, which the
method probe emits for CGP's `self`-less provider methods, is dropped with the rest of rustc's
sub-notes. The ordinary-bound use-site case
([`unsatisfied-dependency/ordinary_bound_unsatisfied`](../../tests/ui/usability/unsatisfied-dependency/ordinary_bound_unsatisfied.rs))
gets the same `CGP-E001` header over a `root cause: the trait bound \`f64: std::cmp::Eq\` is not
satisfied` note.

Two boundaries keep the transform honest. A field whose name matches but whose **type** does not is
*not* handled: with the derive present, the `HasField` trait bound still holds (for the wrong `Value`),
and only the associated-type projection fails — an `E0271` the walk cannot see, so the resolver
declines and [`field_type_mismatch`](../../tests/ui/usability/checks/field_type_mismatch.rs) keeps
rustc's already-precise output. And a diagnostic that is neither a check entry nor a method `E0599` — a
manual supertrait bound like `use_type_foreign_unsatisfied`/`use_type_nested_unsatisfied` — has no
context to recover and falls back. Everything the resolver declines flows through the untouched
`rewrite`/`preprocess` stages exactly as before. `mixed_rust_error` shows both sides at once: its CGP
check failure becomes a tree while its ordinary `E0308` type mismatch passes through the fallback.

## Why it runs in the emitter

The natural home for whole-crate typed analysis would be an `after_analysis` callback, where the
compiler hands the driver a `TyCtxt` directly. That door is closed for the crates that matter here.
The `analysis` query raises a fatal error the moment type-checking reports any non-lint error
(`rustc_interface`'s `analysis` calls `has_errors_excluding_lint_errors().raise_fatal()`), and that
unwind happens *before* the driver's `after_analysis` hook is reached — so for a crate with a CGP
check failure, which by definition has an error, `after_analysis` never runs. The same fact is why
Clippy's late passes only see code that type-checks.

The one place that executes *while the error exists but before the fatal* is the diagnostic emitter,
which the compiler calls as it emits each error during trait solving. At that moment a `TyCtxt` is in
thread-local scope — the driver already relies on this for the trait-renaming rewrite — so the
resolver reaches the compiler through `rustc_middle::ty::tls` from inside `emit_diagnostic`. The cost
is a subtlety the resolver has to be sound against: it re-enters the trait solver *from within a
diagnostic that is itself being emitted mid-solve*. Building a fresh `InferCtxt` and `ObligationCtxt`
and solving a concrete obligation there turns out to work cleanly, and that re-entrancy is the
load-bearing assumption of the whole design — it was proven on `base_area_1` before any of the
machinery was built.

## How the root cause is recovered

The recovery runs in [`resolve.rs`](../../crates/cargo-cgp-driver/src/resolve.rs), driven by the
emitter's `transform_resolved`, and it is a chain of typed lookups with no string parsing until the
very last step decodes a field name. Each stage is anchored by `DefId` to the CGP crate that defines
the trait or type it matches, so a same-named item from an unrelated crate can never drive a
replacement — the same discipline [`component_map`](../../crates/cargo-cgp-driver/src/component_map.rs)
uses for `IsProviderFor`.

**Find the entry.** A `check_components!` entry expands to a concrete impl of a generated check
trait — `impl __CheckRectangle<AreaCalculatorComponent, ()> for Rectangle {}` — whose check trait
carries `CanUseComponent<Marker, Params>` as a supertrait. The macro re-spans the context type in
that impl onto the entry the user wrote, so the impl's `Self`-type span equals the failing
diagnostic's primary span. The resolver walks the crate's check traits (those with a
`cgp_component::CanUseComponent` supertrait) and their impls, and picks the impl whose `Self` span
matches the caret. This span match is what ties *this* diagnostic to *this* entry without reading
either one's text.

**Recover the concrete obligation.** The check trait's supertrait is generic —
`Self: CanUseComponent<__Component__, __Params__>`. Instantiating it with the matched impl's trait
reference (`instantiate_supertrait`) substitutes the concrete types back in, yielding the real
obligation the compiler failed to prove: `Rectangle: CanUseComponent<AreaCalculatorComponent, ()>`.

**Or recover it at a use site.** When no check impl matches the caret — a consumer-method `E0599` —
`resolve_use_site` recovers the obligation instead from the diagnostic's spans. It scans every local
struct/enum whose definition span contains one of the diagnostic's spans (the receiver's type is one
such), and for each candidate reads the `DelegateComponent<Marker>` impls that context carries — the
components it wires — building a fresh `Ctx: CanUseComponent<Marker, ()>` per marker and keeping the
ones that do not hold. A diagnostic span can also land on a *provider* struct, so a candidate that
wires no failing component is discarded, which selects the real context. From there the walk is
identical.

**Walk the dependency graph downward.** From that obligation the resolver walks *down* the wiring's
trait obligations, because the tree shows the transitive path to each root cause, not only the root.
For a failing obligation it finds the impl that would satisfy it and takes that impl's `where`-clause
obligations as its direct dependencies, then recurses into just the ones that do **not** already hold —
a satisfied dependency (an already-present field, a wired provider that checks out) is pruned.

A branch ends at a **terminal leaf**, and which obligations count as terminal is what keeps the tree
honest. The descent follows only the CGP wiring vocabulary — `CanUseComponent`, `IsProviderFor`,
`DelegateComponent`, any provider trait, and any obligation whose `Self` is the context (its getter and
capability traits) — and treats everything else as a leaf. An unmet `HasField` is the field leaf. An
ordinary bound on a *foreign* type (`f64: Eq`) is a terminal leaf too, and crucially the descent stops
there rather than walking into whatever unrelated `std` blanket impl happens to match its `Self` (an
`impl<F: FnPtr> Eq for F` would otherwise fabricate a misleading `f64: FnPtr` step). Two further rules
prune noise: an obligation whose satisfying impl's `where`-clauses **all hold**, yet is itself unmet,
is a projection/associated-type mismatch the trait-clause walk cannot see, so that branch yields
nothing (this is the `E0271` type-mismatch decline); and a branch that bottoms out on pure wiring
plumbing — a `CanUseComponent`/`IsProviderFor`/`DelegateComponent` routing dead-end — is dropped,
since the real cause is found down another branch.

Two mechanical properties matter. First, following *every* unmet dependency, not just the first, is
what surfaces independent causes as **separate** paths — the next-generation solver short-circuits a
conjunction at its first unmet bound, so a provider that needs two absent fields would otherwise hide
one. Second, finding the satisfying impl uses the `fresh_args_for_item`-plus-unification dance rather
than `SelectionContext`, which asserts against the next-generation solver the driver runs under; each
matched impl's predicates are instantiated, normalized, and region-erased before they cross into the
fresh inference context that checks whether they hold, since a stray inference or region variable from
one context panics another.

**Decode the field name.** The `HasField` leaf carries the field name as a type-level `Symbol!`, a
nested `Chars<'h', Chars<'e', …>>` spine. The resolver decodes it structurally — walking the spine and
reading each `char` const argument until `Nil` — rather than un-sugaring the printed type. Reading the
name from the type rather than the text is why the replacement never needs the `--verbose` un-eliding
the [text path depends on](driver.md#un-eliding-the-diagnostic): the characters are in the `Symbol`
arguments whether or not the diagnostic would have printed them.

**Classify why the field is unmet.** A "missing" `HasField` bound does not always mean an absent
field. The resolver inspects the struct the bound lands on — the leaf's self type — and its `Deref`
chain, to tell three cases apart. If the struct carries no field of that name and neither does any
`Deref` target, the field is genuinely **missing**. If the struct itself carries the field, the bound
is unmet only because the struct is missing (or has an incomplete) `#[derive(HasField)]` — **present**.
If the field lives on a struct reached through `Deref` (CGP's `HasField` forwards across `Deref` via a
blanket impl, so the bound *would* hold if that target derived the field), the fault is on the target —
**present-via-`Deref`** — and the resolver records that target's name so the fix can point at it. The
inspection reads named struct fields directly and follows `Deref` by reading each `impl Deref`'s
`Target` associated type, so it needs no inference context; it is bounded against a cyclic `Deref`.
This classification is what lets the emitter word a present field's diagnostic as an unimplemented
accessor with a concrete fix rather than as a bare "missing field". (A field present with a
mismatched *type* is not one of these three: its `HasField` trait impl holds, so the branch never
reaches it — see the `E0271` boundary above.)

A non-field leaf carries no struct to inspect, so it is simply restated as `self: Trait`
(`f64: std::cmp::Eq`) for its note lead and for de-duplicating a leaf reached by several paths.

**Render each root cause as its own sub-error.** Each root-cause path is a list of typed predicates,
and rendering it is where every CGP wiring trait is replaced by the concept it stands for, so the reader
never meets a raw `IsProviderFor` or `Symbol`. `CanUseComponent<Marker>` becomes the consumer-trait impl
(`consumer trait impl \`CanCalculateArea\` for context \`Rectangle\``), an `IsProviderFor` becomes the
provider-trait impl naming its provider trait, context, and provider struct (`provider trait impl
\`AreaCalculator\` with context \`Rectangle\` for provider \`RectangleArea\``), and `HasField` becomes
the field-trait impl (`field trait impl \`HasField\` with field \`height\` for \`Rectangle\``); a user's
own capability trait — or a terminal ordinary bound — renders as `trait impl \`Trait\` for \`Self\``
(`trait impl \`HasRectangleFields\` for \`Rectangle\``, `trait impl \`Eq\` for \`f64\``). A **generic**
component's parameters are reattached to its consumer and provider labels from the `Params` slot of
`CanUseComponent`/`IsProviderFor` — a single one bare, several unwrapped from their tuple — so the
trait reads as written (`CanCalculateArea<u32, u64, bool>`, `AreaCalculator<u32, u64, bool>`). The
marker-to-trait-name lookups go through the same [`ComponentNameMap`](error-processing.md) the
trait-renaming rewrite is built on, but keyed by each marker's **full path** (`def_path_str`) rather
than its bare name, so two components that share a name in different modules resolve to their own trait
names instead of one clobbering the other. Pure plumbing that carries no information — the
`DelegateComponent` table, the routing `IsProviderFor` for the *context itself* (as opposed to the real
provider), and a bare provider-trait obligation that an `IsProviderFor` node already stands for — is
dropped, so the chain stays legible without losing a real dependency step. Each cleaned path folds into
a [`DependencyTree`](error-processing.md) spine, rendered as `cargo tree`-style indented text by the
[`termtree`](https://crates.io/crates/termtree) crate (a tiny, dependency-free renderer) hosted in the
rustc-free `cargo-cgp-error-processing` crate so the rendering is unit-tested on any toolchain.

**Emit.** The emitter (`transform_resolved`) mutates rustc's own `DiagInner` in place. The main
message is replaced with its coded form when `categorized_header` recognizes it — the `CGP-E001`
consumer form worded from the resolution's context and consumer trait(s) (pluralized when a use-site
failure spans several components), or the `CGP-E002` provider form from the text rewrite — and the
span is then collapsed to the primary caret, since the original labels restate the replaced message;
an unrecognized main message keeps its header, labels, and caret. The children are replaced in either
case: a `help` per distinct type that must derive
(`make sure that \`#[derive(HasField)]\` is used for \`Rectangle\``, or the `Deref` target), then
**one note per root cause**, each opening with its `root cause:` lead over `this is required through
the dependency chain:` and the tree indented beneath (the lead is omitted when the kept header states
the same bound, where the note is the chain alone). rustc's structured suggestions are discarded with
its notes (the misleading "use associated function syntax" a method `E0599` carries). The diagnostic's
code is never touched, so a check failure stays `E0277` and a use-site failure stays `E0599`. A
provider with two absent dependencies yields two notes, each a self-contained path to its leaf, and
the JSON emitter regenerates every rendered and structured field from the `DiagInner` for free, with
rustc's note-continuation indentation aligning each tree's box-drawing under its `= note:`.

## Boundaries and open ends

The resolver is deliberately bounded, and a few of its edges are worth recording. It recovers a
starting obligation two ways — a `check_components!` entry by **exact span match** (the check macro
re-spans the context type onto the entry) and a use-site `E0599` by finding the context ADT from the
diagnostic's spans — so a wiring failure that is *neither* — a manual supertrait bound like
`use_type_foreign_unsatisfied`/`use_type_nested_unsatisfied` — still finds nothing to anchor on and
declines. The use-site path builds each `CanUseComponent<Marker, ()>` with an **empty `Params` slot**,
so a generic component whose real parameters matter is not re-checked there (only the check path
recovers those). It renders only leaves it can trust: a `HasField` field, an ordinary bound on a
foreign type, or a terminal capability bound — but it deliberately *declines* the
projection/associated-type mismatch (the `E0271` field-type case) and drops pure wiring-plumbing
dead-ends, so a diagnostic whose only recoverable leaf is one of those falls back. And it uses an
**empty parameter environment**
throughout, which suits the concrete check impls the fixtures exercise but will need the impl's own
environment to extend cleanly to checks that carry generic parameters. (Parallel branches, deep nesting,
and non-field leaves, by contrast, are handled: independent unmet dependencies become separate sub-errors,
the descent follows the wiring to any depth up to a recursion bound, and an ordinary or capability bound
renders as its own tree.)

How a transformed diagnostic is *marked* as CGP is settled by the [error-code scheme](../error-code.md):
a rewritten, classified main message carries its `[CGP-Exxx]` code inline, and everything else — a kept
header over rewritten sub-messages included — is deliberately unmarked, keeping rustc's own
`error[E0277]:` form. There is no separate header brand; an earlier `CGP[E0277]` wrapper the front-end
once applied has been removed, since the inline code says the same thing without altering the header's
shape.

## Source

- [`crates/cargo-cgp-driver/src/resolve.rs`](../../crates/cargo-cgp-driver/src/resolve.rs) — the typed
  resolution: `resolve_check_failure` finding the check impl by span, `resolve_use_site` recovering the
  context ADT from the diagnostic's spans and its wired components from `DelegateComponent` impls,
  walking the cause chain down to each terminal leaf (the descendable-vocabulary rule, the plumbing-leaf
  and projection-mismatch drops), classifying a leaf as a field (inspecting the struct and its `Deref`
  chain) or a bound, decoding the `Symbol!` field name, resolving component markers to trait names by
  full path, and folding each chain into a `DependencyTree` with each wiring trait replaced by its human
  form (generic parameters reattached).
- [`crates/cargo-cgp-driver/src/emitter.rs`](../../crates/cargo-cgp-driver/src/emitter.rs) — the
  `try_resolve` seam (gated by a cheap `mentions_wiring` scan, or a method `E0599`) that tries the
  check anchor then the use-site anchor, and the `transform_resolved` mutation it feeds: the
  `categorized_header` classification and coded main-message rewrite, the `derive_helps`, and the
  per-cause `tree_notes`, falling back to the in-place text rewrite when resolution returns `None`.
- [`crates/cargo-cgp-error-processing/src/tree.rs`](../../crates/cargo-cgp-error-processing/src/tree.rs)
  — the rustc-free `DependencyTree` type and its `cargo tree`-style renderer (over `termtree`), with
  unit tests in [`tests/tree.rs`](../../crates/cargo-cgp-error-processing/tests/tree.rs).
- [`crates/cargo-cgp-driver/src/config.rs`](../../crates/cargo-cgp-driver/src/config.rs) — the crate
  and trait-name anchors (`CanUseComponent`, `IsProviderFor`, `HasField`, and the `Symbol` spine's
  crate) the resolution matches against.

## Tests

The resolver is exercised end to end by the UI snapshot suite: the check fixtures under
[`tests/ui/usability/checks/`](../../tests/ui) carry `.cgp.stderr` snapshots showing the transformed
output, and the fixtures the resolver declines keep their fallback snapshots, which together pin both
the transform and the decline boundary. Several fixtures pin the harder cases: `parallel_branches` (two
independent missing fields → two sub-errors), `deep_nesting` (a stack of higher-order providers nested
four deep → one long spine), `dependency_cascade` (a chain of providers each depending on the next),
`mixed_rust_error` (a CGP tree beside an untouched ordinary `E0308`), `missing_has_field_derive` (a
field the struct carries but has not derived → the unimplemented-accessor header plus the derive
`help`), `field_via_deref` (a field on a `Deref` target that does not derive `HasField` → the `help`
pointed at the target), `field_type_mismatch` (a matching field name with a mismatched type → the
`E0271` boundary that declines to the fallback), `same_name_components` (two components forced to share
a marker name in different modules, with distinct consumer *and* provider trait names, both checked →
full-path resolution names each one's own traits with no cross-over), `generic_area_multi` (a
three-parameter component → the parameters reattached to the consumer and provider labels), and
`ordinary_bound_unsatisfied`/`unregistered_prefix_path` (non-field leaves — an `f64: Eq` bound, whose
rustc header is kept over a lead-less chain note, and an unregistered `DefaultNamespace` behind an
`IsProviderFor` header, rewritten to the `CGP-E002` form over a `root cause:` note). The use-site path
is pinned by the [`unsatisfied-dependency/`](../../tests/ui/usability/unsatisfied-dependency)
fixtures: `missing_dependency` and `unsatisfied_dependency` (a consumer-method `E0599` → the
`CGP-E001` header, the misleading method-syntax advice dropped, and a `missing field` root-cause note)
and its `ordinary_bound_unsatisfied` (a use-site `f64: Eq` → the `CGP-E001` header, code kept `E0599`,
over the `f64: Eq` root-cause note). The field classification is unit-tested through the name map in
[`cargo-cgp-error-processing/tests/rewrite.rs`](../../crates/cargo-cgp-error-processing/tests/rewrite.rs),
and the renderer itself in
[`cargo-cgp-error-processing/tests/tree.rs`](../../crates/cargo-cgp-error-processing/tests/tree.rs).
[Testing](testing.md) describes the suite and its passes.

## Further reading

- [The driver](driver.md) — the emitter seam this resolver extends, and the trait-renaming rewrite it
  falls back to.
- [The error pipeline](error-pipeline.md) — where this driver-side transformation sits among the
  pipeline's four stages.
- [CGP check-trait failure](../../../cgp/docs/errors/checks/check-trait-failure.md) — the upstream
  error class the resolver reshapes.
