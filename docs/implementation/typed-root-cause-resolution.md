# Typed root-cause resolution

The driver's most valuable transformation turns a CGP check-failure diagnostic into a compact,
root-cause-first error by *asking the compiler* what really failed rather than parsing rustc's text.
When a context is wired wrong, the compiler reports the failure against generated types the
programmer never wrote, and the one fact that matters — a missing field, an unwired component — is
buried under `IsProviderFor`/`CanUseComponent` scaffolding or dropped entirely. The resolver re-runs
the failing obligation through the trait solver, walks the wiring down to the actual root cause, and
re-renders the whole diagnostic as a `cargo tree`-style dependency chain with a single coded
headline.

The resolver reads the failure from the **real consumer and provider trait obligations**, never from
the `CanUseComponent`/`IsProviderFor` scaffolding. Those two traits exist only so that plain rustc
can surface a wiring failure at all (see [check traits](../../../cgp/docs/concepts/check-traits.md)):
`IsProviderFor` is generated to carry a *copy* of a provider's `where` bounds precisely so the
compiler names the missing one. cargo-cgp does not need that copy — it re-runs the trait solver on
the real provider impl, whose own `where` clause holds the same bounds — so it treats `IsProviderFor`
and `CanUseComponent` as plumbing to resolve *around*, reading the actual traits instead. This is a
deliberate constraint, not an accident of implementation: cargo-cgp aims to make `IsProviderFor`
*removable*, so its dependency resolution must not lean on it. (The text-rewrite fallback for
diagnostics the resolver declines still recognizes `IsProviderFor`/`CanUseComponent` in rustc's
rendered output; that is a separate concern, covered in [The driver](driver.md).)

This is the second, deeper transformation the driver's emitter performs, and it builds on the first.
[Naming the traits behind a component marker](driver.md#naming-the-traits-behind-a-component-marker)
edits a diagnostic in place, renaming its wording; the resolver instead reconstructs the failure from
compiler state and replaces the diagnostic wholesale. It realizes the compiler-state enrichment that
[The driver](driver.md) and [The error pipeline](error-pipeline.md) anticipated, and everything
downstream of its rustc-free [`Resolved`](../../crates/cargo-cgp-error-processing/src/diagnosis/resolved.rs)
model — the wording, the tree — is unit-tested without a compiler.

## A worked example

The clearest way in is one failure end to end, from the [area-calculation
example](../../../cgp/docs/examples/area-calculation.md). A `Rectangle` computes its area through a
wired `RectangleArea` provider that reads the rectangle's fields, but the struct is missing one of
them:

```rust
#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_auto_getter]
pub trait HasRectangleFields {
    fn width(&self) -> f64;
    fn height(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator
where
    Self: HasRectangleFields,
{
    fn area(&self) -> f64 {
        self.width() * self.height()
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    // the `height` field is missing
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}
```

`RectangleArea` reads `width` and `height` through the `HasRectangleFields` getter, whose
`#[cgp_auto_getter]` blanket impl requires the context to have both fields. `Rectangle` derives
`HasField` but declares only `width`, so the wiring cannot be satisfied — the **root cause is the
absent `height` field** — and `check_components!` fails. Left to rustc, the failure reads as an unmet
`HasField<Symbol!("height")>` bound (often with the field name itself compressed to an unreadable
`Symbol<6, Chars<'h', …>>` spine), routed through `IsProviderFor` and `CanUseComponent`. The resolver
replaces all of that with:

```text
error[E0277]: [CGP-E001] the consumer trait `CanCalculateArea` is not implemented for context `Rectangle`
  --> src/main.rs:61:9
   |
61 |         AreaCalculatorComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: root cause: [CGP-E106] missing field `height` on `Rectangle`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
             └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
               └─ [CGP-E105] trait impl `HasRectangleFields` for `Rectangle`
                 └─ [CGP-E106] missing field `height` on `Rectangle`
```

Every element of that output is reconstructed from the compiler, not read from rustc's message: the
headline names the *consumer trait* the reader called (`CanCalculateArea`) and the context that
cannot implement it; the `root cause:` note names the actual mistake in one sentence; and the tree
shows the transitive path from the check entry down to the missing field, with each node named
straight off the trait it stands for. Read the chain as the real obligation chain the walk descended:
`Rectangle: CanCalculateArea` (the consumer) needs `RectangleArea: AreaCalculator<Rectangle>` (the
wired provider), which needs `Rectangle: HasRectangleFields` (the getter), which needs the `height`
field. No `IsProviderFor` node appears because the walk never went through one. The `[CGP-Exxx]` codes
are catalogued in [error-code.md](../error-code.md). The rest of this document explains how each piece
is produced.

## When the resolver engages, and when it declines

The resolver treats a diagnostic as a candidate whenever it plausibly stems from a CGP component
failure, then either traces it to a root cause or steps aside. Concretely, a diagnostic is a
candidate when it **names a CGP wiring or field trait** (`CanUseComponent`, `IsProviderFor`, or
`HasField` — matched in rustc's rendered text, the one place these still serve as a *signal* that the
diagnostic is CGP-related), or carries code **`E0271`**, **`E0277`**, or a **method-bounds `E0599`** —
the "the method `…` exists … but its trait bounds were not satisfied" shape. The breadth past the
wiring-worded cases is deliberate, because a failure that names no CGP construct can still be one a
CGP component caused: a hand-written `Send`-recovery wrapper's `async fn` fails with an `E0271`
opaque-future mismatch, a downstream bound needs a method the context cannot supply. The resolver
traces the dependency chain and treats the error as CGP-related exactly when a CGP component failure
sits in that chain; a candidate whose chain reaches no CGP cause **declines** and passes through to
the fallback text rewrite untouched.

The `E0599` arm is narrowed to the method-bounds shape for a reason beyond relevance: a
*resolution*-class `E0599` (`no variant named …`, `no associated item …`) is emitted while type
lowering is still mid-flight, and running the resolver's trait solver on it re-enters the diagnostic
context and aborts the compiler. Declining such an `E0599` before any solving is both crash-safe and
correct — the resolver has nothing to say about a name-resolution error — whereas `E0271`/`E0277` are
trait-solving failures reported after collection and do not hit the hazard. This is the
re-entrant-emission panic catalogued in
[rustc diagnostic internals](rustc-diagnostic-internals.md#re-entering-the-diagnostic-context-lock-was-already-held),
where the phase a diagnostic is emitted in is what decides whether the solver may safely run on it.

A candidate the resolver accepts is transformed in two independent halves — the coded headline and
the root-cause notes — described in
[Typed resolution: the transformed diagnostic](typed-resolution-output.md). A candidate it declines keeps rustc's diagnostic, cleaned only
by the fallback [post-processing](error-processing.md).

## Why it runs in the emitter

The natural home for whole-crate typed analysis would be an `after_analysis` callback, where the
compiler hands the driver a `TyCtxt` directly — but that door is closed for exactly the crates that
matter. The `analysis` query raises a fatal error the moment type-checking reports any non-lint error
(`rustc_interface`'s `analysis` calls `has_errors_excluding_lint_errors().raise_fatal()`), and that
unwind happens *before* `after_analysis` runs. A crate with a CGP check failure has an error by
definition, so `after_analysis` never sees it — the same reason Clippy's late passes only run on code
that type-checks.

The one place that executes *while the error exists but before the fatal unwind* is the diagnostic
emitter, which the compiler calls as it emits each error during trait solving. A `TyCtxt` is in
thread-local scope there (the driver already relies on this for the trait-renaming rewrite), so the
resolver reaches the compiler through `rustc_middle::ty::tls` from inside `emit_diagnostic`. The
subtlety this design must be sound against is that it re-enters the trait solver *from within a
diagnostic being emitted mid-solve*. Building a fresh `InferCtxt` and `ObligationCtxt` and solving a
concrete obligation there works cleanly, and that re-entrancy is the load-bearing assumption of the
whole approach — proven on the area example before any of the machinery was built.

Running compiler code from this position is also the source of every panic the tool has hit. The
constraints it imposes — never force a query that emits, instantiate a binder before relating it, keep
each fresh `InferCtxt`'s variables to itself — are catalogued together in
[rustc diagnostic internals](rustc-diagnostic-internals.md#panic-hazards-running-compiler-code-inside-the-emitter),
and the boundaries at the end of this document note where a hazard puts a case out of reach.

## How the root cause is recovered

The recovery is a pipeline of typed lookups with no string parsing until the very last step decodes a
field name. It runs in the driver's [`resolve`](../../crates/cargo-cgp-driver/src/resolve) module —
one sub-directory per stage (`anchor/`, `call_site/`, `walk/`, `classify/`, `label/`) over the shared
`cgp_item.rs`, each behind a re-exporting `mod.rs` — and fills the rustc-free
`Cause`/`Leaf`/`FieldIssue`/`Resolved` types with owned `String`s, so the wording that consumes them
needs no compiler. Every stage is anchored by `DefId` to the CGP crate that defines the trait or type
it matches, so a same-named item from an unrelated crate can never drive a replacement. The stages
run in order: anchor the starting obligation, walk it down to the leaves, decode and classify each
leaf, render the chain, and emit. Each stage has its own document, and this section is the map of
them.

Two facts hold across every stage, and everything below assumes them. **The obligation the walk works
on is always a real consumer-trait obligation** `Ctx: ConsumerTrait<Params…>` — never a
`CanUseComponent` wrapper — and **the traits are recognized structurally, without `IsProviderFor`**:

- A **provider trait** is identified by the delegation blanket `#[cgp_component]` generates —
  `impl<Ctx, P> ProviderTrait<Ctx> for P where P: DelegateComponent<Marker>, …` — whose `Self` is a
  bare type parameter bounded by `DelegateComponent` (`is_provider_trait` / `provider_blanket_marker`
  in `cgp_item.rs`). That same blanket's `DelegateComponent<Marker>` bound also yields the component
  marker when an anchor needs it, in place of reading the `IsProviderFor<Marker, …>` supertrait.
- A **consumer trait** is identified by its blanket impl `impl<C> Consumer for C where C: Provider<C>`
  routing to such a provider (`consumer_provider_trait` / `is_consumer_trait`).
- The **marker → consumer** inversion the check and use-site anchors need (`marker_to_consumer`) is the
  composition of those two: the provider trait whose blanket keys on the marker, then the consumer
  whose blanket routes to that provider.

**[Anchoring the starting obligation](typed-resolution-anchors.md)** recovers the obligation the
compiler failed to prove, one of six ways tried in order. A `check_components!` entry is matched by
its caret to the check impl and its `CanUseComponent<Marker, Params>` assertion mapped to the real
consumer obligation; a failure inside a hand-written `impl Trait for Context` block is recovered
from the impl's CGP consumer supertrait, and one inside an `impl … for Foreign` wrapper by
descending its supertrait's `where`-clause hops to a consumer on the context; a consumer-method
`E0599` is recovered from the context's own wired components, or from the consumer trait the
diagnostic names (the anchor that reaches a namespace-joined context).

**[The call-site anchor](typed-resolution-call-site.md)** is the last resort, for the use-site
failure whose spans touch nothing the other anchors can read — a wiring that matches the called
component unconditionally, so the failure is an `E0277` on the call itself. It re-reads the failing
call expression from HIR alone: the receiver carries the context, the component's parameters come
from unifying the call's *written* argument types against the method's own declared signature, and
every parameter the call leaves to inference is seeded as a rigid placeholder the walk resolves
around but never reports on.

**[Walking to the root cause](typed-resolution-walk.md)** descends the seeded obligation's
dependency graph — following only the CGP wiring vocabulary and obligations on the context itself,
never `IsProviderFor`/`CanUseComponent` — to every terminal unmet bound, then decodes each terminal
(a `Symbol!` field name read structurally, the struct and its `Deref` chain inspected, a missing
wiring or dispatch entry named) and renders each hop as a tree label with the type-level spines
resugared.

**[The transformed diagnostic](typed-resolution-output.md)** is the output side: the coded
`[CGP-E001]`/`[CGP-E002]`/`[CGP-E003]`/`[CGP-E009]` headline classes, the `root cause:` notes over
their merged dependency trees (with underived-field causes coalesced and repeated trait generics
elided), and the emitter's application of the rustc-free plan to the compiler's `DiagInner`.

## Boundaries and open ends

The resolver is deliberately bounded, and a few edges are worth recording. Because it anchors the six
ways above, a wiring failure that is *none* of them still declines. The consumer-trait use-site anchor
widened the reach considerably — a manual supertrait bound in a trait definition or `where` clause, and
a namespace-joined use-site call, both now resolve whenever a local CGP consumer trait and its context
appear in the diagnostic's spans (the once-declining `use_type_foreign_unsatisfied`,
`use_type_nested_unsatisfied`, and `namespace_join_use_site` fixtures, now under
[`acceptable/`](../../tests/ui/acceptable)) — and the
[call-site anchor](typed-resolution-call-site.md) closed the once-declining
*foreign generic consumer* gap, the unconditionally-matching dispatch shape
([`cascade_after_use_site`](../../tests/ui/acceptable/use-site/cascade_after_use_site.rs), also now
under `acceptable/`). What still declines is a failure none of the recoveries reach: a caret only on a
*provider* struct's own impl (whose `Self` is the provider, reaching no consumer on a context), a
generic component's trait definition, a call whose *receiver's type* is not syntactically recoverable
(a method call's result or a field access, `self.app.handle(…)` — typing those needs the typeck
results the emitter can never force), or a written type beyond the call-site anchor's small hand
lowering. The wrapper-chain descent is itself bounded — it follows only real impl `where`-clauses,
reports a cause only at a genuine CGP consumer on a local context, and stops at a recursion bound —
and every call-site seed is gated on actually failing, so neither can fabricate a chain from an
unrelated bound.

One use-site shape is out of reach for a hard reason worth recording: a **consumer-method call whose
failure is an `E0271`, not an `E0599`** — `app.deserialize_json_string::<Payload>(…)` on a context that
cannot deserialize `Payload`, which fails as a type mismatch on the capability's output (the
modular-serialization arena test hits this). Its caret sits on the method call, naming no
context-definition span, so neither use-site anchor finds a context; and recovering the obligation would
need the compiler's **typeck results**, which the resolver cannot obtain. `tcx.typeck` replays its
cached diagnostics when forced, so forcing it from the emitter re-enters the diagnostic context and
panics — the re-entrant-emission hazard in
[rustc diagnostic internals](rustc-diagnostic-internals.md#re-entering-the-diagnostic-context-lock-was-already-held).
Only the fresh-`InferCtxt` trait solver is safe to re-enter mid-emit; a full query is not, and there is
no hook between typeck and the fatal error to precompute the result. So this failure falls through to
rustc's output, usually redundant with the `check_components!` failure for the same capability, which
the resolver *does* reshape.

A few parameter-recovery limits remain. The impl-site path recovers a generic component's concrete
parameter from the supertrait, the by-component use-site path recovers it for an `open`-dispatched
component from the `PathCons<Component, Value>` redirect key, and the call-site path recovers every
parameter the call *writes*, seeding the rest as unknowns; what no path recovers is a parameter
whose only carrier is an argument the call does **not** type syntactically — a plain variable, an
unsuffixed literal — where the by-component path re-checks its bare marker with an empty `()` slot,
the by-consumer path only fires for a consumer whose sole generic is `Self`, and the call-site
seed's unknown makes every root cause parameter-dependent. Such a failure declines to the fallback —
whose method-probe advice the emitter strips, so the unmet wiring bound leads
([`generic_consumer_unwritten_arg`](../../tests/ui/acceptable/use-site/generic_consumer_unwritten_arg.rs)
pins the cleaned output). The plausible recovery, should the class need resolving in full, is to
re-check the wired delegate's *implemented* parameter values instead of the meaningless `()` form.
And the walk
uses an **empty parameter environment** throughout, which suits the concrete check
impls the fixtures exercise but will need the impl's own environment to extend cleanly to checks that
carry generic parameters. The resolver renders only leaves it can trust — a `HasField` field (missing,
underived, or type-mismatched), a missing wiring, a missing dispatch entry on a non-context delegation
table, a namespace redirect the context does not terminate,
an ordinary foreign bound, or a terminal capability bound — and declines an associated-type projection
mismatch that is *not* a `HasField` one, dropping pure wiring-plumbing dead-ends, so a diagnostic whose
only recoverable leaf is one of those falls back. Parallel branches, deep nesting, and non-field leaves,
by contrast, are all handled.

How a transformed diagnostic is *marked* as CGP is settled by the [error-code scheme](../error-code.md):
a rewritten, classified main message carries its `[CGP-Exxx]` code inline, and everything else — a kept
header over rewritten sub-messages included — stays in rustc's own `error[E0277]:` form. There is no
separate header brand; the inline code is the only marking.

## Source

- [`crates/cargo-cgp-driver/src/resolve/`](../../crates/cargo-cgp-driver/src/resolve) — the typed
  resolution, split by stage into sub-directories behind re-exporting `mod.rs` files and building the
  rustc-free `Resolved` model. Every anchor feeds the walk the real consumer obligation
  `Ctx: ConsumerTrait<Params…>`, never a `CanUseComponent` wrapper.
  - [`anchor/`](../../crates/cargo-cgp-driver/src/resolve/anchor) holds five of the six anchors, one
    file each, plus their two shared ingredients: `seed.rs` (`consumer_obligation`, the `Params`-slot
    ungrouping decided by the consumer's own generics — a single tuple-typed parameter kept whole, a
    lifetime restored from `Life<'a>` via `life_region`, any mismatch declining rather than handing
    the solver a malformed trait ref) and `spans.rs` (the local impls and struct definitions a
    diagnostic's spans land on). `check_failure.rs` matches the check impl by span, then
    `can_use_to_consumer_obligation` maps its `CanUseComponent<Marker, Params>` assertion through
    `marker_to_consumer` to the consumer obligation. `impl_site.rs` recovers the context and the
    failing supertrait from an enclosing `impl Trait for Context` block, heading the tree with the
    impl's own wrapper trait — `[CGP-E001]` or `[CGP-E009]` by its blanket-impl fingerprint — through
    `wrapper_consumer_causes`, which seeds the supertrait directly when it is a CGP consumer *or* a
    `#[cgp_fn]`/`#[blanket_trait]` blanket-impl trait (`is_local_blanket_trait`), the latter
    reshaping a `#[cgp_fn]` capability check whose cause is a missing field. `wrapper_chain.rs` is
    the foreign-wrapper case, descending each hop's `where`-clauses via `wrapper_chain_children` read
    un-normalized so an associated-type bound descends to its base trait, until
    `consumer_handoff_causes` reaches a CGP consumer on the context, named plainly with
    `subject_is_context = false`. `use_site.rs` recovers the context ADT from the diagnostic's spans
    and its wired components from `DelegateComponent` impls via `delegated_check_targets`, mapping
    each marker to its consumer and recovering an `open`-dispatch value from a `PathCons` key through
    `open_dispatch_target`, while skipping a raw path key, a redundant bare marker, a free-parameter
    catch-all, and a `namespace …;` blanket `__Key__` key. `use_site_consumer.rs` recovers a local,
    non-generic CGP consumer trait from the diagnostic's spans and walks `Ctx: Consumer` directly —
    the anchor that reaches a namespace-joined context.
  - [`call_site/`](../../crates/cargo-cgp-driver/src/resolve/call_site) holds the sixth anchor,
    `resolve_call_site` — the HIR re-read of the failing call, one stage per file:
    `find_call.rs` (`method_calls_at`, the calls at, or inside an expression at, the diagnostic's
    spans — the latter for the await-desugar wrappers — and the candidate consumer traits by method
    name), `receiver.rs` (`receiver_context`/`local_binding_context`, the receiver's type from its
    binding, annotation, parameter, literal, or constructor-call signature), `seed.rs`
    (`seed_from_call`, the signature unification: fresh variables for the method's item, `Self`
    pinned to the context, each written argument type unified with its declared input, the trait's
    parameters read back with `walk`'s `unknowns_to_placeholders` folding what stayed unresolved),
    `written_ty.rs` (`expr_written_ty`, the argument shapes whose types the call writes — a tuple
    read partially, its shape recovered with a fresh inference variable per unwritten element), and
    `lower.rs` (`lower_hir_ty`/`instantiate_written`, the small syntactic type lowering over cached
    `type_of`).
  - [`walk/`](../../crates/cargo-cgp-driver/src/resolve/walk) walks the cause chain to each terminal
    leaf. `leaves.rs` drives the recursion (`resolve_leaves`/`collect_leaf_paths`, the cycle guard
    and `MAX_DEPTH` backstop, the foreign-getter descent into just context-side dependencies plus a
    same-trait list recursion, the drop of a leaf still carrying a call-site placeholder — an
    unknowable `_: Send` is never reported — and the repeated-generics elision applied to the
    collected labels). `vocabulary.rs` decides what the descent walks into (`is_descendable` —
    provider traits, `DelegateComponent`, and context obligations, *not*
    `IsProviderFor`/`CanUseComponent` — and the `is_workaround_plumbing` drop of a
    `CanUseComponent`/`IsProviderFor` dependency beside the real obligation). `impl_match.rs` finds
    the satisfying impl and reads its dependencies (`impl_where_obligations` preferring a
    concrete-`Self` impl over the delegation blanket, the placeholder instantiation of a
    higher-ranked binder via `enter_forall_and_leak_universe`, and solving satisfiable clauses
    first). `unknowns.rs` carries a call-site unknown across inference contexts
    (`unknowns_to_placeholders`, shared with the call-site anchor; `resolve_fixed_projections` /
    `try_project_fixed` / `placeholders_to_infer` recovering a stalled associated-type projection
    whose value is *fixed* independent of an unknown input — structurally, for any trait and
    associated type — keeping only a fully-concrete result, so a later pipeline stage keyed on an
    earlier stage's un-normalized output projection is still descended and reported on when its
    input concretizes). `projection_mismatch.rs` finds an unmet `HasField` projection on the
    concrete-`Self` impl (`has_field_projection_mismatch`/`impl_field_projection_mismatch`,
    deferring the blanket), and `holds.rs` asks the solver whether a predicate is satisfied.
  - [`classify/`](../../crates/cargo-cgp-driver/src/resolve/classify) classifies a terminal:
    `leaf.rs` turns it into the rustc-free `Leaf` (a field by inspecting the struct and its `Deref`
    chain in `field.rs`, a field-type mismatch with `field_type` reading the actual type by `DefId`,
    a missing wiring on the context, a missing dispatch entry on a non-context delegation table, a
    not-a-provider on a non-table type, a missing redirect wiring told apart by `is_path_cons`, or a
    bound); `reportable.rs` holds `is_reportable_leaf`, keeping an unmet `DelegateComponent` on the
    context (a missing wiring), on a non-context delegation table (a missing dispatch entry —
    recognized as a separate-table lookup by `is_dispatch_lookup`, the owner a proper part of the
    parent obligation's `Self`, or by owner property via `owner_has_impl_of` for
    `DelegateComponent`), or on a non-table type with no concrete impl of the parent provider trait
    (a not-a-provider, via `owner_has_impl_of` against that trait), while dropping a
    higher-order-provider dead-end (a non-table owner that *does* have a concrete impl of the trait).
  - [`label/`](../../crates/cargo-cgp-driver/src/resolve/label) renders the chain:
    `predicate_label.rs` names each consumer/provider node off its trait `DefId` and the obligation's
    arguments (`trait_generics`) through the pure label constructors in the rustc-free
    `diagnosis::labels`, dropping the plumbing; `render_ty.rs` resugars a `DefId`-anchored
    `Cons`/`Nil` or `Either`/`Void` self type to `Product![…]`/`Sum![…]`, or — when every element is
    a `Field` — to `Struct! { … }`/`Enum! { … }`, rendering a call-site placeholder as `_` (recursing
    into a tuple so a nested placeholder prints `_` rather than the raw `!N` form).
  - `cgp_item.rs` holds the structural, `IsProviderFor`-free trait recognition — `is_provider_trait` /
    `provider_blanket_marker` (the `DelegateComponent`-bounded provider blanket), `consumer_provider_trait`
    / `is_consumer_trait`, and `marker_to_consumer` — plus the `Symbol!` field-name decode,
    `is_namespace_lookup_trait` (by the single-`Delegate`-associated-type fingerprint), and the
    shared `PathCons`/`Nil`/local-ADT recognizers. A sibling
    [`conflict/`](../../crates/cargo-cgp-driver/src/resolve/conflict) handles the duplicate-key
    `E0119` conflict — a separate transform documented in
    [The driver](driver.md#reshaping-a-duplicate-key-conflict). A sibling `cache.rs` memoizes
    `resolve_leaves` on its region-erased seed and context, so a wiring failure re-reported at many
    sites is walked once (see [Cached dependency resolution](cached-dependency-resolution.md)).
- [`crates/cargo-cgp-driver/src/emitter/`](../../crates/cargo-cgp-driver/src/emitter) — the `try_resolve`
  seam (gated by a cheap `mentions_wiring` scan, an `E0271`/`E0277` code, or a method-bounds `E0599`,
  with a resolution-class `E0599` excluded so the solver never runs on an error emitted
  mid-`predicates_of`) that tries the six anchors in turn, and the `transform_resolved` mutation it
  feeds — mapping the rustc code to a `DiagKind` (overridden to the use-site kind for a call-anchored
  resolution), calling `plan_resolved`, and applying the plan to the
  `DiagInner`, falling back to the in-place text rewrite when resolution returns `None`. A final
  cross-diagnostic de-duplication suppresses a re-report of a failure already shown.
- [`crates/cargo-cgp-error-processing/src/diagnosis/`](../../crates/cargo-cgp-error-processing/src/diagnosis)
  — the rustc-free model and wording: `leaf.rs`/`resolved.rs` (the `Leaf`, `FieldIssue`, `Cause`, and
  `Resolved` types), the `wording/` directory (the coded headers, the `root cause:`/`root causes:`
  notes — the latter merging causes that share a dependency root via `cause_notes` →
  `merge_dependency_forest` — the leads and their codes, the derive `help`s, and the de-duplication
  `cause_signature`), `coalesce.rs` (`coalesce_underived_fields`, merging several underived fields on
  one struct into a single cause), `labels.rs` (the pure tree-label constructors and
  `elide_repeated_generics`), and
  `plan.rs` (`DiagKind`, `DiagnosisPlan`, and `plan_resolved` with its `categorized_header`),
  unit-tested in [`tests/diagnosis.rs`](../../crates/cargo-cgp-error-processing/tests/diagnosis.rs),
  [`tests/coalesce.rs`](../../crates/cargo-cgp-error-processing/tests/coalesce.rs), and
  [`tests/labels.rs`](../../crates/cargo-cgp-error-processing/tests/labels.rs).
- [`crates/cargo-cgp-error-processing/src/tree.rs`](../../crates/cargo-cgp-error-processing/src/tree.rs) —
  the `DependencyTree` type (with `from_chain`, folding a chain of labels into a spine) and its
  `cargo tree`-style renderer over `termtree`, unit-tested in
  [`tests/tree.rs`](../../crates/cargo-cgp-error-processing/tests/tree.rs).
- [`crates/cargo-cgp-driver/src/config.rs`](../../crates/cargo-cgp-driver/src/config.rs) — the crate and
  trait-name anchors the resolution matches against.

## Tests

The resolver is exercised end to end by the UI snapshot suite. The fixtures it reshapes live under
[`tests/ui/acceptable/`](../../tests/ui/acceptable) — the `fields/`, `field-types/`, `providers/`,
`generic/`, `resolution/`, `verbosity/`, `wiring/`, `use-site/`, and `use-type/` subgroups, carrying
`.cgp.stderr` snapshots of the transformed output. The failure it still declines — a use-site `E0599`
on a generic consumer whose dispatch parameter rides in an argument the call does not type
syntactically — pins its fallback (with the method-probe advice stripped) in
[`acceptable/use-site/generic_consumer_unwritten_arg`](../../tests/ui/acceptable/use-site/generic_consumer_unwritten_arg.rs),
so the two sides together pin both the transform and the decline boundary;
[`acceptable/verbosity/deep_dispatch_chain`](../../tests/ui/acceptable/verbosity/deep_dispatch_chain.rs)
pins the repeated-generics elision on a resolved deep dispatch chain. [Testing](testing.md)
describes the suite and its bless workflow. The fixtures group by what they pin.

Each **leaf class** has fixtures for its field, wiring, and redirect shapes:

- `base_area_1` — a genuinely missing field (the worked example).
- `missing_has_field_derive` — a present-but-underived field, with the derive `help`.
- `field_via_deref` — a field on a `Deref` target, with the `help` pointed at the target.
- `field_type_mismatch` and `field_type_mismatch_1` — a matching name with a mismatched type, read
  through a getter and directly via an `#[implicit]` argument.
- `field_type_mismatch_modules` — two `Rectangle` contexts in separate modules with differently-typed
  `height` fields, proving the actual-type query is `DefId`-anchored.
- `basic_missing_wiring` — a `#[uses]` dependency on an unwired component.
- `direct_missing_wiring` — a checked component wired nowhere (a single-node chain).
- `parallel_missing_wiring` — two unwired components (two notes).
- `transitive_missing_wiring` — a component wired through an *aggregate provider* that the aggregate
  itself does not wire, the `[CGP-E110]` missing-dispatch-entry leaf on a non-context delegation table
  (`provider \`CommonProvider\` does not contain any delegate entry for \`BarProviderComponent\``), with
  the whole tree correctly rooted at the checked context.
- `record_field_chain` — a record provider building each field through the context over a recursive
  `Cons`/`Nil` handler (the modular-serialization `DeserializeRecordFields`/`HandleMapEntry` shape),
  whose tree entries also pin the `Cons`/`Nil` → `Struct! { … }` resugaring.
- `sum_variant_chain` — the sum counterpart over a `Sum![u64, f64]` spine of bare types, pinning the
  `Either`/`Void` → `Sum![…]` resugaring left as a plain list.
- `enum_variant_chain` — a sum of *named* variants, pinning the `Enum! { Rect(u64), … }` form.
- `unregistered_prefix_path`, `qualified_prefix_path` (a module-qualified path still folding to a clean
  `@…`), `multi_redirect_missing` (several hops), and `open_missing_type_key` — the namespace-redirect
  variants.

Several fixtures pin the **harder mechanics**:

- `parallel_branches` — two independent missing fields, two sub-errors.
- `deep_nesting` — higher-order providers nested four deep, one long spine.
- `dependency_cascade` — a chain of providers each depending on the next, its intermediate consumers
  each a `[CGP-E101]` node.
- `mixed_rust_error` — a CGP tree beside an untouched `E0308`.
- `same_name_components` — two components sharing a marker name in different modules, resolved to their
  own traits (off their `DefId`s) with no cross-over.
- `generic_area_multi` — a three-parameter component, its parameters reattached to the labels from the
  obligation's own arguments.
- `lifetime_component` — a component carrying a *lifetime* parameter (`(Life<'a>, str)` in its check
  entry), the lifetime restored from its `Life<'a>` lift to a region when the consumer obligation is
  rebuilt, and the provider label's context read by type position past the leading lifetime.
- `tuple_param_component` — a component whose single parameter is itself a *tuple* type, kept whole
  (`CanFormatPair<(u32, u64)>`) rather than spread into two parameters by the params-slot ungrouping.
- `check_providers_layer` (under [`acceptable/providers/`](../../tests/ui/acceptable/providers)) — a
  `#[check_providers(...)]` per-layer assertion, whose `IsProviderFor`-supertraited check impl no
  anchor matches directly, resolved through the use-site anchor instead (rustc's "not implemented for
  `Rectangle`" note spans the context's struct definition) into the failing layer's root-cause tree.
- `ordinary_bound_unsatisfied` — a non-field `f64: Eq` bound whose rustc header is kept over a lead-less
  chain note.
- `constrained_delegate_key` and `pipe_handlers_empty` (under
  [`acceptable/wiring/constrained-key/`](../../tests/ui/acceptable/wiring/constrained-key)) — a
  delegation whose `DelegateComponent` impl carries a constrained key that is unsatisfied, so the walk
  descends the delegation impl and bottoms out on its unmet `where`-bound as an ordinary-bound leaf.
  The first is a self-contained dispatcher (`PickFirstProvider<Product![]>`, whose `Nil` list has no
  `PickFirst` impl); the second is the canonical core-CGP form, an empty
  `PipeHandlers<Product![]>` whose `Nil` fails `ComposeProviders`.
- `foreign_getter_missing_wiring` — the money-transfer `UseBasicAuth` shape, where the walk descends a
  request getter's blanket impl into its context-side dependency and the misleading second root cause
  collapses into the one missing wiring, under a promoted `CGP-E001` header.
- `higher_ranked_descent` — a recursive provider with a `Self: for<'a> CanEncodeItem<&'a Value>`
  dependency (the `SerializeIterator` shape) that used to feed an escaping bound variable into the
  solver and panic rustc, now resolved through the placeholder instantiation.
- `nested_higher_ranked_descent` — the same nested twice through the record machinery (the
  `MessagesArchive` shape), which used to decline to the raw fallback.
- `enum_hasfields_lock` — a resolution-class `E0599` emitted mid-`predicates_of`, which the resolver
  must decline rather than run its solver on and re-enter the `DiagCtxt` lock.

The **use-site paths** are pinned by the [`acceptable/use-site/`](../../tests/ui/acceptable/use-site)
and [`acceptable/use-type/`](../../tests/ui/acceptable/use-type) fixtures:

- `missing_dependency` and `unsatisfied_dependency` — a consumer-method `E0599` giving the `CGP-E001`
  header with the method-syntax advice dropped over a `missing field` note.
- `missing_wiring` — a use-site `E0599` whose provider needs an unwired component.
- `ordinary_bound_unsatisfied` — a use-site `f64: Eq`, code kept `E0599`.
- `open_dispatch_use_site` — the dispatch value recovered from the redirect key, so the header names
  `CanEncodeItem<Seq<u64>>` and the note reaches the real `@ItemEncoderComponent.u64` wiring rather than
  reporting the internal `PathCons` key as a bogus consumer trait.
- `namespace_join_use_site` — a use-site `E0599` on a namespace-joined context, anchored on the
  `CanGreet` consumer trait from the diagnostic and walked through the namespace's `RedirectLookup` to
  the missing field, with the blanket `__Key__` forwarding skipped.
- `cascade_after_use_site` — the unconditionally-dispatched `E0277` of the
  [call-site anchor](typed-resolution-call-site.md)'s worked example, resolved into one
  `[CGP-E001]` block whose await-site re-report de-duplicates and whose `?`-operator cascade stays
  suppressed.
- `cascade_later_stage` — the same dispatch shape but with the missing field read by the *second*
  pipeline stage rather than the first, so the cause sits behind a `ComposeHandlers` stage keyed on the
  first stage's un-normalized `::Output`. Pins the placeholder-fold of a still-unresolved clause: the
  walk descends the later stage instead of dropping it as inference-laden, reaching the field the first
  stage's un-reportable `_: Send` would otherwise have hidden. This is a common real-world shape — a
  multi-stage pipeline whose later stage reads a field the context has not wired, behind an earlier
  stage that consumes the pipeline's own input — and the fixture is the self-contained distillation of
  it.
- `cascade_later_stage_input` — the harder variant of the same shape, where the later stage's only
  cause is a requirement on its **input type** (`Input: AsRef<[u8]>`) and that input is the earlier
  stage's *fixed* output, threaded through a forwarding handler so it resolves through `CanHandle`.
  Folding the stalled projection to a placeholder leaves the requirement an unreportable
  `_: AsRef<[u8]>`; pins the [`resolve_fixed_projections`](typed-resolution-walk.md#walking-the-dependency-graph-downward)
  recovery that reduces the earlier stage's fixed output by re-normalizing the projection with the
  unknown input treated as deferrable, so the later stage's `<output>: AsRef<[u8]>` becomes the
  reported cause.
- `cascade_nested_projection` — the three-stage variant distilled from the `http_checksum_native`
  hypershell example, where the later stage's input (an earlier stage's fixed, projection-typed output
  produced by a nested `PipeHandlers`) reaches an **input dispatcher** (`UseInputDelegate`) that has no
  entry for it. Pins the `[CGP-E110]` missing-dispatch-entry leaf: the cause is an unmet
  `DelegateComponent` on the dispatch *table* rather than the context, which the walk used to drop as
  plumbing — declining the whole resolution — and now reports, leading with
  `provider \`SinkHandlers\` does not contain any delegate entry for \`Tagged<Bytes>\`` over the chain.
- `empty_dispatch_table` — the same missing-dispatch-entry leaf on an **empty** `UseInputDelegate`
  table, the case the owner-property check cannot see (no `DelegateComponent` impl to find). Pins the
  structural `is_dispatch_lookup` recognition: the unmet `DelegateComponent`'s owner is a proper part
  of the parent `UseInputDelegate<EmptySink>` obligation's `Self`, so it is reported regardless of the
  table wiring any other key.
- `non_provider_wired` (under [`acceptable/providers/`](../../tests/ui/acceptable/providers)) — a type
  wired where a provider was expected that does not implement the provider trait at all
  (`WrapGreeter<NotAGreeter>` with `NotAGreeter` an ordinary struct, the `money-transfer-api`
  `UseBasicAuth<QueryBalanceRequest>` shape). Pins the `[CGP-E111]` not-a-provider leaf and its
  discriminator against the `cascade_after_use_site` dead-end: `NotAGreeter` has no concrete `Greeter`
  impl (`owner_has_impl_of` false), so it is reported as `the provider trait \`Greeter\` is not
  implemented for \`NotAGreeter\`` rather than declining to a `[CGP-E002]` block naming the whole
  `WrapGreeter<NotAGreeter>` pipeline.
- `generic_consumer_use_site` — the same anchor's value-argument case: the dispatch parameter
  recovered by signature unification from a written tuple, no tag argument involved, with the
  misleading method-syntax advice dropped.
- `call_site_tuple_input` — a provider that destructures its input on a tuple shape
  (`(CondInput, Rest)`), reached by a call whose tuple argument the code does not type
  (`(Vec::new(), Vec::new())`). Pins the *partial* tuple recovery: the anchor keeps the tuple arity
  (with each unwritten element a placeholder) so the provider's impl matches and the walk reaches the
  field a branch reads, where collapsing the tuple to one flat unknown used to leave the impl
  unmatched and decline. This is the shape a branching/comparison DSL interpreter hits — e.g. a real
  `If<Compare<…>, …>` program reading an unwired field inside its condition.
- `use_type_foreign_unsatisfied` and `use_type_nested_unsatisfied` — an unsatisfiable `#[use_type]`
  abstract-type import in a trait definition, recovered by the consumer-trait anchor into a
  `[CGP-E001]` missing-wiring tree instead of leaking generated `__…__` placeholder names.

The **impl-site and wrapper-chain paths** are pinned by:

- `manual_supertrait_impl` — a wrapper carrying a generic CGP consumer supertrait implemented directly
  on the context (the `CanHandleApiSend` shape), failing at both the impl header `E0277` and its
  forwarding-call `E0599`, both collapsing to one `[CGP-E009]` block.
- `traced_send_wrapper` — an async `Send`-recovery wrapper whose opaque-future `E0271` names no CGP
  construct, traced to the wrapper-headed tree.
- `foreign_wrapper_chain` — a routing trait on a foreign `Box<App>` whose `where`-clause chain reaches a
  CGP consumer two hops down, the cause reached through a projection bound's base trait, headed by the
  `[CGP-E009]` foreign-plain form.
- `cgp_fn_missing_field` (under [`acceptable/fields/`](../../tests/ui/acceptable/fields)) — a
  `#[cgp_fn]` capability asserted through a wrapper (`pub trait CheckFormatName: FormatName {}` +
  `impl CheckFormatName for App {}`) whose blanket-impl supertrait is not a CGP component, pinning the
  `is_local_blanket_trait` extension: the impl-site anchor walks the `#[cgp_fn]` blanket's `where`
  clause (and a `#[uses]`-chained sibling) to the `` missing field `name` `` cause instead of
  declining to rustc's misleading `#[derive(HasField)]` note.

Finally, the leaf wording and the tree renderer are unit-tested over hand-built `Resolved` values,
independently of the compiler:

- [`cargo-cgp-error-processing/tests/diagnosis.rs`](../../crates/cargo-cgp-error-processing/tests/diagnosis.rs)
  — the coded headers, the `root cause:` notes, and the derive `help`s.
- [`cargo-cgp-error-processing/tests/tree.rs`](../../crates/cargo-cgp-error-processing/tests/tree.rs) —
  the `cargo tree`-style renderer.

## Further reading

- The per-stage documents this overview links: [anchoring the starting
  obligation](typed-resolution-anchors.md), [the call-site anchor](typed-resolution-call-site.md),
  [walking to the root cause](typed-resolution-walk.md), and
  [the transformed diagnostic](typed-resolution-output.md).
- [The driver](driver.md) — the emitter seam this resolver extends, and the trait-renaming text rewrite
  it falls back to (which still recognizes `IsProviderFor`/`CanUseComponent` in rustc's output).
- [Error processing](error-processing.md) — the rustc-free crate that holds the `Resolved` model, the
  wording, and the fallback post-processing.
- [Check traits](../../../cgp/docs/concepts/check-traits.md) — why `IsProviderFor`/`CanUseComponent`
  exist, the workaround this resolver is designed to make removable.
- [rustc diagnostic internals](rustc-diagnostic-internals.md) — where rustc drops information the
  resolver must recover, and the panic hazards of running compiler code inside the emitter.
- [The error pipeline](error-pipeline.md) — where this driver-side transformation sits among the
  pipeline's stages.
- [CGP check-trait failure](../../../cgp/docs/errors/checks/check-trait-failure.md) — the upstream error
  class the resolver reshapes.
