# Typed root-cause resolution

The driver can replace a CGP check-failure diagnostic with its own, built from the root cause it
recovers by *asking the compiler* rather than by reading the error text. This document describes that
resolver: what it replaces, why it runs where it does, how it recovers the root cause through the
trait solver, and the boundaries that decide when it steps aside and lets the older text-rewrite path
handle the diagnostic instead.

This is the second, deeper transformation the driver's emitter performs. The first —
[naming the traits behind a component marker](driver.md#naming-the-traits-behind-a-component-marker) —
edits the compiler's diagnostic in place, keeping its structure and rewriting its wording. The
resolver goes further: when it succeeds it discards the compiler's diagnostic entirely and emits a
fresh one whose header *is* the root cause. It realizes the compiler-state enrichment that
[The driver](driver.md) and [The error pipeline](error-pipeline.md) anticipated.

## What it replaces, and what it leaves alone

The resolver targets exactly one thing today: an `E0277` whose caret sits on a `check_components!`
entry and whose ultimate cause is an **unmet `HasField` bound**. That is the surfaced [check-trait
failure](../../../cgp/docs/errors/checks/check-trait-failure.md) class, the most common CGP error a
programmer meets, and the one whose root cause the compiler renders as an unreadable nested `Symbol`.
For such a diagnostic the resolver emits, in place of rustc's cascade, a header naming the field(s)
and the context, with the caret still on the wiring entry, and, for each field, a note whose body is
the transitive dependency chain that needs it, rendered as a `cargo tree`-style tree.

The header and each note are worded by *why* the bound is unmet, which the resolver decides by
inspecting the actual struct the bound lands on (detailed under [How the root cause is
recovered](#how-the-root-cause-is-recovered)). A genuinely absent field reads as
`missing field \`height\` on context \`Rectangle\``. A field the struct *does* carry — the wiring is
unmet because the struct lacks its `#[derive(HasField)]` — reads instead as `accessor trait
\`HasField\` with field \`name\` is not implemented for \`Person\``, and a separate `help`
subdiagnostic names the fix: `make sure that \`#[derive(HasField)]\` is used for \`Person\``. A field
reached only through the context's `Deref` chain points that `help` at the target struct that must
derive it. This distinction matters because CGP's own [`missing_has_field_derive`
fixture](../../tests/ui/usability/checks/missing_has_field_derive.rs) is exactly the present-but-underived
case a plain "missing field" would misdescribe.

One kind of field fault is *not* this path: a field whose name matches but whose *type* does not. With
`#[derive(HasField)]` present, the `HasField` trait bound still holds (for the wrong `Value` type), and
only the associated-type projection fails — an `E0271`, not an `E0277`. The resolver's field branch
never sees it, so it declines and the [`field_type_mismatch`
fixture](../../tests/ui/usability/checks/field_type_mismatch.rs) flows through the text-rewrite fallback,
which already points the caret at the offending field and its expected type.

Every other diagnostic is left untouched for the existing pipeline to handle. The resolver is a
strict addition guarded on both ends: it only *attempts* an `E0277` on a check entry, and it only
*replaces* when it can follow the failure all the way down to a genuine CGP `HasField` bound. A check
failure rooted in an ordinary trait bound (`f64: Eq`), an unmet abstract type (`HasScalarType`), or a
namespace lookup resolves to `None`, and the original diagnostic flows on through the in-place text
rewrite exactly as before. This is the fallback the resolver depends on: the older
`rewrite`/`preprocess` stages are not modified, and they remain the handler for everything the typed
path cannot fully resolve. Across the UI suite this shows as a clean split — every missing-field check
fixture is replaced, and the check fixtures whose cause is not a field, along with all the hidden,
wiring, and lowering fixtures, pass through unchanged. `mixed_rust_error` shows both sides at once: its
CGP check failure is replaced with a tree while its ordinary `E0308` type mismatch flows through the
fallback untouched.

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
emitter's `build_replacement`, and it is a chain of typed lookups with no string parsing until the
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

**Walk the dependency graph downward.** From that obligation the resolver walks *down* the wiring's
trait obligations, because the replacement shows the transitive path to each root cause, not only the
root. For a failing obligation it finds the impl that would satisfy it and takes that impl's
`where`-clause obligations as the obligation's direct dependencies, then recurses into just the ones
that do **not** already hold — a satisfied dependency (an already-present field, a wired provider that
checks out) is pruned. A branch that bottoms out on an unmet `cgp_field::HasField` is a root cause.
Two properties matter here. First, following *every* unmet dependency, not just the first, is what
surfaces independent missing fields as **separate** paths — the next-generation solver short-circuits a
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

**Render each root cause as its own sub-error.** Each root-cause path is a list of typed predicates,
and rendering it is where every CGP wiring trait is replaced by the concept it stands for, so the reader
never meets a raw `IsProviderFor` or `Symbol`. `CanUseComponent<Marker>` becomes the consumer-trait impl
(`consumer trait impl \`CanCalculateArea\` for context \`Rectangle\``), an `IsProviderFor` becomes the
provider-trait impl naming its provider trait, context, and provider struct (`provider trait impl
\`AreaCalculator\` with context \`Rectangle\` for provider \`RectangleArea\``), and `HasField` becomes
the field-trait impl (`field trait impl \`HasField\` with field \`height\` for \`Rectangle\``); a user's
own capability trait renders as `trait impl \`HasRectangleFields\` for \`Rectangle\``. The
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

The emitter then builds one replacement `DiagInner` per check entry: a header worded by the field
classification — `missing field \`height\` on context \`Rectangle\`` when every field is absent,
`accessor trait \`HasField\` with field \`name\` is not implemented for \`Person\`` when at least one
is present-but-underived, and the plural list when several are unmet (`missing fields \`first_name\`
and \`last_name\` on context \`Person\``) — the compiler's `E0277` code preserved so `rustc --explain`
still works, and the caret on the entry. When any field is present-but-underived, a `help`
subdiagnostic per distinct type that must derive names the fix once —
`make sure that \`#[derive(HasField)]\` is used for \`Rectangle\`` (or the `Deref` target for a
`Deref`-reachable field) — rather than repeating it in every note. Then — this is what "separate
sub-errors" means — **one terse note per root cause**, each opening `field \`x\` is required through
this dependency chain:` and carrying that field's own dependency tree. A provider with two absent
fields therefore yields two notes, each a self-contained path to its field, rather than one merged
tree. Emitting a hand-built `DiagInner` renders correctly for free, because the JSON emitter
regenerates every rendered and structured field from it, and rustc's note-continuation indentation
aligns each tree's box-drawing under its `= note:`.

## Boundaries and open ends

The resolver is deliberately narrow, and three of its edges are worth recording. It handles only the
**`HasField`-leaf** root cause (whether the field is absent, present-but-unwired, or behind a `Deref`);
a branch that ends on any other unmet leaf — an ordinary trait bound, an unmet abstract type — is pruned
as not-a-field, so a check that fails *solely* on such a bound finds no root cause and declines to the
fallback, and one that fails on *both* a field and such a bound replaces with only the field's tree,
dropping the other. Widening the leaf kinds the resolver renders is
the natural next step. It correlates a diagnostic to an entry by **exact span match**, which holds
because the check macro re-spans the context type onto the entry; a future change to that spanning would
need to be matched here. And it uses an **empty parameter environment** throughout, which suits the
concrete check impls the fixtures exercise but will need the impl's own environment to extend cleanly to
checks that carry generic parameters. (Parallel branches and deep nesting, by contrast, are handled:
independent unmet dependencies become separate sub-errors, and the descent follows the wiring to any
depth up to a recursion bound.)

One consistency gap is known and left for a deliberate decision rather than silently closed. The
front-end's header preprocessor brands a transformed diagnostic's header as `CGP[E0277]`, gated on the
diagnostic carrying a recognizable CGP marker. A replaced diagnostic carries none of those markers —
its text is already clean — so it renders as a plain `error[E0277]`. The message is unmistakably
CGP-shaped without the brand, so this is not wrong, but it does mean replaced and fallback diagnostics
are branded differently; closing the gap would mean teaching the front-end recognizer about the new
form, which touches the preprocessing stage the resolver otherwise leaves untouched.

## Source

- [`crates/cargo-cgp-driver/src/resolve.rs`](../../crates/cargo-cgp-driver/src/resolve.rs) — the typed
  resolution: finding the check impl by span, recovering and solving the concrete obligation, walking
  the cause chain and descending to the `HasField` leaf, decoding the `Symbol!` field name, classifying
  the field by inspecting the struct and its `Deref` chain, resolving component markers to trait names
  by full path, and folding the chain into a `DependencyTree` with each wiring trait replaced by its
  human form.
- [`crates/cargo-cgp-driver/src/emitter.rs`](../../crates/cargo-cgp-driver/src/emitter.rs) — the
  `build_replacement` seam that tries the resolver first and builds the replacement `DiagInner` (header
  plus the dependency-tree note), falling back to the in-place text rewrite when it returns `None`.
- [`crates/cargo-cgp-error-processing/src/tree.rs`](../../crates/cargo-cgp-error-processing/src/tree.rs)
  — the rustc-free `DependencyTree` type and its `cargo tree`-style renderer (over `termtree`), with
  unit tests in [`tests/tree.rs`](../../crates/cargo-cgp-error-processing/tests/tree.rs).
- [`crates/cargo-cgp-driver/src/config.rs`](../../crates/cargo-cgp-driver/src/config.rs) — the crate
  and trait-name anchors (`CanUseComponent`, `IsProviderFor`, `HasField`, and the `Symbol` spine's
  crate) the resolution matches against.

## Tests

The resolver is exercised end to end by the UI snapshot suite: the missing-field check fixtures under
[`tests/ui/usability/checks/`](../../tests/ui) carry `.cgp.stderr` snapshots showing the replaced
output, and the check fixtures whose cause is not a field keep their fallback snapshots, which together
pin both the replacement and the decline-to-replace boundary. Several fixtures pin the harder cases:
`parallel_branches` (two independent missing fields → two sub-errors), `deep_nesting` (a stack of
higher-order providers nested four deep → one long spine), `dependency_cascade` (a chain of providers
each depending on the next), `mixed_rust_error` (a CGP tree beside an untouched ordinary `E0308`),
`missing_has_field_derive` (a field the struct carries but has not derived → the unimplemented-accessor
header plus the derive `help`), `field_via_deref` (a field on a `Deref` target that does not derive
`HasField` → the `help` pointed at the target), `field_type_mismatch` (a matching field name with a
mismatched type → the `E0271` boundary that declines to the fallback), and `same_name_components` (two
components forced to share a marker name in different modules, with distinct consumer *and* provider
trait names, both checked → full-path resolution names each one's own traits with no cross-over). The
field classification is unit-tested through the name map in
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
