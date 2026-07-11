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
entry and whose ultimate cause is a **missing struct field**. That is the surfaced [check-trait
failure](../../../cgp/docs/errors/checks/check-trait-failure.md) class, the most common CGP error a
programmer meets, and the one whose root cause the compiler renders as an unreadable nested `Symbol`.
For such a diagnostic the resolver emits, in place of rustc's cascade, a single line naming the field
and the context — `missing field \`height\` on context \`Rectangle\`` — with the caret still on the
wiring entry and one note explaining which capability needed the field.

Every other diagnostic is left untouched for the existing pipeline to handle. The resolver is a
strict addition guarded on both ends: it only *attempts* an `E0277` on a check entry, and it only
*replaces* when it can follow the failure all the way down to a genuine CGP `HasField` bound. A check
failure rooted in an ordinary trait bound (`f64: Eq`), an unmet abstract type (`HasScalarType`), or a
namespace lookup resolves to `None`, and the original diagnostic flows on through the in-place text
rewrite exactly as before. This is the fallback the resolver depends on: the older
`rewrite`/`preprocess` stages are not modified, and they remain the handler for everything the typed
path cannot fully resolve. Across the UI suite this shows as a clean split — every one of the 15
missing-field check fixtures is replaced, and the five check fixtures whose cause is not a field,
along with all the hidden, wiring, and lowering fixtures, pass through unchanged.

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
The component marker for the eventual note is read straight off this obligation's arguments.

**Solve, descend, and capture the whole chain.** The resolver registers that obligation in a fresh
`ObligationCtxt`, solves it, and reads the fulfillment errors — but it keeps not just the failing leaf
but the *whole dependency chain* that leads to it, because the replacement note shows the transitive
path, not only the root. Each failing obligation carries a derived-cause chain — the same "required
for … to implement …" ancestry rustc renders as notes — and the resolver walks it (`parent_trait_pred`
by `parent_trait_pred`) to recover every intermediate predicate as typed data. When the leaf is a
genuine `cgp_field::HasField` bound, that *is* the root cause and the chain is complete. When it is not
— because the solver reports the failure at an intermediate wiring obligation one dependency layer up
(an `IsProviderFor` or `CanUseComponent` for a provider that itself depends on the missing field) — the
resolver re-solves that intermediate obligation to descend one layer deeper, stitching each segment's
cause chain onto the growing chain, until a `HasField` surfaces or a depth bound is hit. The descent is
confined to CGP wiring traits, so it never wanders into an unrelated failing bound; an ordinary-trait
leaf simply ends the search with `None`. This is what lets a deep cascade collapse to one clean
message: in `dependency_cascade`, three checked components whose providers chain down to the same
missing `name` field each resolve to `missing field \`name\``, attributed to their own capability,
instead of three walls of nested types.

**Decode the field name.** The `HasField` leaf carries the field name as a type-level `Symbol!`, a
nested `Chars<'h', Chars<'e', …>>` spine. The resolver decodes it structurally — walking the spine and
reading each `char` const argument until `Nil` — rather than un-sugaring the printed type. Reading the
name from the type rather than the text is why the replacement never needs the `--verbose` un-eliding
the [text path depends on](driver.md#un-eliding-the-diagnostic): the characters are in the `Symbol`
arguments whether or not the diagnostic would have printed them.

**Render the chain as a tree, replacing the machinery.** The captured chain is a list of typed
predicates, and rendering it is where each CGP wiring trait is replaced by the concept it stands for,
so the reader never meets a raw `IsProviderFor` or `Symbol`. `CanUseComponent<Marker>` becomes the
consumer capability (`\`App\` uses consumer trait \`CanBaz\``), an `IsProviderFor` becomes the concrete
provider and its provider trait (`provider \`ProvideBaz\` (provider trait \`Baz\`)`), and `HasField`
becomes `missing field \`name\``; the marker-to-trait-name lookups reuse the same
[`ComponentNameMap`](error-processing.md) the trait-renaming rewrite is built on. Pure plumbing that
carries no information — the `DelegateComponent` table, the routing `IsProviderFor` for the *context
itself* (as opposed to the real provider), and a bare provider-trait obligation that an `IsProviderFor`
node already stands for — is dropped, so the chain stays legible without losing a real dependency step.
The cleaned labels are folded into a [`DependencyTree`](error-processing.md) and rendered as
`cargo tree`-style indented text by the [`termtree`](https://crates.io/crates/termtree) crate (a tiny,
dependency-free renderer), hosted in the rustc-free `cargo-cgp-error-processing` crate so the rendering
is unit-tested on any toolchain.

With the field name and the rendered tree in hand, the emitter builds the replacement `DiagInner` — a
root-cause header (`missing field \`height\` on context \`Rectangle\``), the compiler's `E0277` code
preserved so `rustc --explain` still works, the caret on the entry, and one note carrying the whole
dependency tree. Emitting a hand-built `DiagInner` renders correctly for free, because the JSON emitter
regenerates every rendered and structured field from it, and rustc's note-continuation indentation
aligns the tree's box-drawing under the `= note:`.

## Boundaries and open ends

The resolver is deliberately narrow, and four of its edges are worth recording. It handles only the
**missing-field** root cause; other surfaced leaves an obligation can bottom out on — an ordinary
trait bound, an unmet abstract type — are recognized well enough to *decline* but not yet to replace,
and are candidates for the same treatment. It correlates a diagnostic to an entry by **exact span
match**, which holds because the check macro re-spans the context type onto the entry; a future
change to that spanning would need to be matched here. It currently uses an **empty parameter
environment** when re-solving, which suits the concrete check impls the fixtures exercise but will
need the impl's own environment to extend cleanly to checks that carry generic parameters. And the
dependency tree is currently a **single spine**: at each layer the descent follows the first
fulfillment error, so a provider with two independently unmet dependencies would show only the first
path rather than branching — the `DependencyTree` type already models children, so this is a matter of
following every error rather than a structural limit.

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
  the cause chain and descending to the `HasField` leaf, decoding the `Symbol!` field name, and folding
  the chain into a `DependencyTree` with each wiring trait replaced by its human form.
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

The resolver is exercised end to end by the UI snapshot suite: the 15 missing-field check fixtures
under [`tests/ui/usability/checks/`](../../tests/ui) carry `.cgp.stderr` snapshots showing the
replaced output, and the check fixtures whose cause is not a field keep their fallback snapshots,
which together pin both the replacement and the decline-to-replace boundary. [Testing](testing.md)
describes the suite and its passes.

## Further reading

- [The driver](driver.md) — the emitter seam this resolver extends, and the trait-renaming rewrite it
  falls back to.
- [The error pipeline](error-pipeline.md) — where this driver-side transformation sits among the
  pipeline's four stages.
- [CGP check-trait failure](../../../cgp/docs/errors/checks/check-trait-failure.md) — the upstream
  error class the resolver reshapes.
