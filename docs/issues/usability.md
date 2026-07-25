# Usability issues

This document lists the ways `cargo-cgp` presents errors that carry the root cause but bury it — the
problems a reader hits even when the diagnostic contains everything needed to find the cause. What
separates these from a [hidden root cause](hidden-root-cause.md) is that the information is present:
the cause could be recovered from the output by a careful reader or a post-processor, so the work
here is re-presentation, not recovery. Every issue is backed by a fixture under
[`tests/ui/usability/`](../../tests/ui/usability), grouped into a sub-directory that names the issue
class. When a fixture's presentation reaches the bar, it graduates out of `usability/` into
[`tests/ui/acceptable/`](../../tests/ui/acceptable) and its issue is deleted from this document.

## What the tool already fixed

The check-trait-failure family — the dominant class of CGP error — is presented well, so its
fixtures have graduated into [`acceptable/`](../../tests/ui/acceptable). The driver's
[typed root-cause resolver](../implementation/typed-root-cause-resolution.md) leads with a single
coded headline (`[CGP-E001]` for an unimplemented consumer trait, `[CGP-E003]` for a field-type
mismatch), states the cause as one plain sentence (`` root cause: missing field `height` on
`Rectangle` ``), decodes the field name from its `Symbol!`, and renders the dependency path as a
compact `cargo tree`, with the `IsProviderFor` / `CanUseComponent` / `__Check…` scaffolding
suppressed throughout. Several once-open classes of this document have since cleared the bar and
their fixtures now live under `acceptable/`:

- A **missing derive reported field by field** is coalesced into one root cause: several
  present-but-underived fields on one struct merge into a single `[CGP-E108]` lead listing every
  field, over one merged tree, with the derive `help` naming the one fix
  ([`base_area_2`](../../tests/ui/acceptable/fields/base_area_2.rs); the boundaries — a lone
  underived field, genuinely absent fields — are pinned by
  [`underived_and_missing_field`](../../tests/ui/acceptable/fields/underived_and_missing_field.rs)
  and [`parallel_branches`](../../tests/ui/acceptable/fields/parallel_branches.rs)). The *same*
  field reached by several coalesced consumers is one cause, not several: the block's union of its
  members' causes is folded back by leaf before the wording runs, so a lead that once read
  `` the fields `name`, `name`, and `name` `` keeps the single-field form
  ([`coalesced_underived_field`](../../tests/ui/acceptable/duplication/coalesced_underived_field.rs)).
  The same fold keeps a chain from being *lost* at the other end: a use-site failure spanning several
  wired components that share a cause used to keep only the first component's route, so a consumer the
  header named had no chain in the note; each now renders, converging on the shared hop
  ([`use_site_shared_cause`](../../tests/ui/acceptable/duplication/use_site_shared_cause.rs)).
- A **pipeline stage routing back to the context for an unwired `Code`** is resolved rather than
  declined. A type-level DSL interprets each stage by routing through the context's own handler, so a
  program naming a fragment the language has no interpreter for fails one hop away from the stage —
  and a context that joins a namespace carries a blanket forwarding that matches *every* key, which
  used to send the walk into the namespace's own lookup machinery instead of stopping at the absent
  entry. An unmet delegation on the context is now terminal whatever nominally matches it, so the
  block leads with `[CGP-E107] … does not contain any delegate entry for @….Missing` in place of one
  `[CGP-E002]` per combinator layer naming `PipeHandlers`/`ComposeHandlers`
  ([`pipeline_unhandled_code`](../../tests/ui/acceptable/use-site/pipeline_unhandled_code.rs)).
- A **use-site failure the resolver declines** no longer keeps rustc's misleading method advice:
  the "this is an associated function, not a method" framing and the actively wrong "use associated
  function syntax instead" suggestion — both artifacts of CGP's `self`-less provider methods — are
  stripped, leaving the unmet wiring bound as the first note
  ([`generic_consumer_unwritten_arg`](../../tests/ui/acceptable/use-site/generic_consumer_unwritten_arg.rs);
  recovering the dispatch parameter itself remains a documented
  [resolver boundary](../implementation/typed-root-cause-resolution.md#boundaries-and-open-ends)).
- The **wiring coherence conflicts** are largely reshaped: the duplicate delegate-key `E0119` family
  carries its `[CGP-E004]`–`[CGP-E008]` codes, a duplicate `cgp_namespace!` `@`-path now reshapes
  into `[CGP-E008]` naming both redirect targets
  ([`namespace_duplicate_path_key`](../../tests/ui/acceptable/wiring/namespace-paths/namespace_duplicate_path_key.rs)),
  a duplicate provider *name*'s redundant `IsProviderFor` half is suppressed so only the `E0428` and
  the provider-trait conflict remain
  ([`duplicate_provider_name`](../../tests/ui/acceptable/wiring/duplicate-keys/duplicate_provider_name.rs)),
  and the `UseContext` cycle's `E0275` is rewritten into a `[CGP-E010]` headline with a `help`
  naming the usual cause
  ([`use_context_cycle`](../../tests/ui/acceptable/wiring/constraints/use_context_cycle.rs)).
- A **cross-context dependency** — one context's wiring depending on a *concrete* other context, so
  one obligation appears in two contexts' trees — is resolved cleanly on both sides. A provider's own
  `where Inner: CanCompute` clause is recovered as that consumer obligation (de-duplicating into
  `Inner`'s own block rather than leaking `__Context__`/`IsProviderFor` or declining to rustc's raw
  bound), and the same node inside the *outer* context's tree is re-rooted at `Inner` so it decodes to
  the missing field instead of an opaque bound
  ([`cross_context_node_key`](../../tests/ui/acceptable/resolution/cross_context_node_key.rs)).
- An **orphan-rule namespace registration** — registering wiring into a namespace the crate does not
  own, keyed on a component it does not own either — is reshaped from rustc's `E0210`/`E0117` (which
  names the machinery parameter `__Components__`/`__Table__` and frames the mistake as a bare coherence
  rule) into a `[CGP-E011]` header naming the foreign namespace and key, with the ownership-based fix in
  a `help`. The three triggers now live under
  [`acceptable/wiring/orphan/`](../../tests/ui/acceptable/wiring/orphan): a `#[default_impl]` on a bare
  foreign component marker
  ([`default_impl_foreign_component`](../../tests/ui/acceptable/wiring/orphan/default_impl_foreign_component.rs)),
  on a foreign prefix path
  ([`default_impl_foreign_prefix_path`](../../tests/ui/acceptable/wiring/orphan/default_impl_foreign_prefix_path.rs)),
  and a `cgp_namespace!` re-open whose `__Table__` trigger selects the inherit-a-new-namespace fix
  ([`reopen_foreign_namespace`](../../tests/ui/acceptable/wiring/orphan/reopen_foreign_namespace.rs)).
- **One mistake reported as many errors** is collapsed on both axes. CGP wiring is lazy, so one
  missing dependency surfaces at the `check_components!` entry, at every hand-written `impl` that
  references the broken consumer, and at each call. The *same* consumer re-reported at many sites
  de-duplicates to one block, keyed on a span-independent cause signature — the transfer example's
  single un-wired password type collapses from eighteen identical trees to two
  ([`cross_site_dedup`](../../tests/ui/acceptable/duplication/cross_site_dedup.rs),
  [`manual_supertrait_impl`](../../tests/ui/acceptable/use-site/manual_supertrait_impl.rs)). And
  *different* consumers that share one root cause coalesce into a single `[CGP-E001]` headline
  listing every affected consumer trait, with a caret per failing entry and the shared cause shown
  once. The emitter holds each compilation's diagnostics in arrival order and flushes them at `Drop`
  — the only point after every diagnostic has arrived — grouping together the consumer failures that
  *share* a root cause, so
  [`density_3`](../../tests/ui/acceptable/duplication/density_3.rs) (two components, one missing
  `height`), [`dependency_cascade`](../../tests/ui/acceptable/duplication/dependency_cascade.rs)
  (three chained providers), and `missing_normal_bound` (two consumers sharing an `App: Clone` bound)
  each collapse to one block, and cargo's re-count keeps the "N errors" summary honest. Grouping on a
  shared cause rather than on one whole-failure key is what reaches the *partial-overlap* form of this
  class, where one omission is reached at several instantiations and so surfaces as several root
  causes: each `check_components!` entry stops at the first unmet leaf on its own branch while a
  use-site call walks every wired component and reaches them all, so no two of those cause sets are
  equal and none used to group. The union block fared worst of them, since every one of its roots had
  already been drawn and its chain was then elided away entirely, leaving a bare `root causes:` list
  with no dependency chain
  ([`overlapping_cause_sets`](../../tests/ui/acceptable/duplication/overlapping_cause_sets.rs)).
- A **capability used but not declared** — a `#[cgp_fn]`/`#[cgp_impl]` body that calls a CGP
  capability (a consumer or `#[cgp_fn]`/`#[blanket_trait]` trait) on `self` without declaring it via
  `#[uses(…)]`, so the method cannot resolve on the generated `__Context__` generic — is reshaped
  from rustc's vague `E0599` (which names `__Context__` and points at a transitive `HasField` bound,
  the wrong fix) into a `[CGP-E012]` header naming the capability, with the `#[uses(…)]` fix in a
  `help`
  ([`undeclared_uses_capability`](../../tests/ui/acceptable/lowering/undeclared_uses_capability.rs)).
  Any `[T]: Sized` cascade the unresolved return type trails is left as rustc wrote it — those errors
  can land off the failing expression, where suppressing them reliably would risk hiding an unrelated
  error.
- A **`#[cgp_impl]` header naming the wrong trait** — the component's *consumer* trait where its
  *provider* trait belongs (`#[cgp_impl(new P)] impl CanCalculateArea` instead of `impl AreaCalculator`),
  or a trait that is not a CGP component at all — is reshaped from the burst of cryptic macro-lowering
  errors it produces (`E0425` on a `…Component` marker the user never wrote, `E0107`, `E0186`, `E0207`,
  plus a downstream check failure) into a single `[CGP-E013]`/`[CGP-E014]` error on the misused trait
  name, naming the fix, with the whole cascade suppressed. The recognition uses the consumer/provider
  fingerprints, so it generalizes over a component's generic parameters and tells a wrong-half mistake
  (`[CGP-E013]`, which names the provider trait to use) apart from a non-component trait (`[CGP-E014]`,
  which has no provider to suggest)
  ([`consumer_trait_in_provider_impl`](../../tests/ui/acceptable/lowering/consumer_trait_in_provider_impl.rs),
  [`consumer_trait_in_provider_impl_generic`](../../tests/ui/acceptable/lowering/consumer_trait_in_provider_impl_generic.rs),
  and [`cgp_impl_on_non_cgp_trait`](../../tests/ui/acceptable/lowering/cgp_impl_on_non_cgp_trait.rs)).
- A **higher-order provider's `#[use_provider]` mistake** is reshaped the same way. Forgetting to
  import the inner provider — calling `InnerCalculator::area(self)` without
  `#[use_provider(InnerCalculator: AreaCalculator)]`, so the parameter is unbounded — is reshaped from
  rustc's vague `E0599` (which leaks the generated `__Context__` and suggests the wrong consumer-trait
  bound) into a `[CGP-E016]` error naming the inner provider and the `#[use_provider(…)]` fix
  ([`higher_order_missing_use_provider`](../../tests/ui/acceptable/lowering/higher_order_missing_use_provider.rs)).
  Naming the *consumer* trait in the `#[use_provider]` bound — the inner-bound sibling of the header
  mistake above — is reshaped into a `[CGP-E015]` error pointing at the provider trait to use, with
  the `E0308` body cascade suppressed
  ([`higher_order_use_provider_consumer_trait`](../../tests/ui/acceptable/lowering/higher_order_use_provider_consumer_trait.rs)).

- An **abstract type the context binds one way while a provider pins it another** is resolved into a
  coded mismatch of its own. A context chooses an abstract type's concrete form by wiring its
  component to `UseType<T>`; a provider can pin the same type with the
  `#[use_type(Trait.{Assoc = Concrete})]` equality form, and when the two disagree the trait bound
  still holds and only the projection fails. The resolver used to recognize only a `HasField`
  projection there and decline everything else, leaving rustc's `type mismatch resolving
  <Ctx as HasErrorType>::Error == AppError` under its `IsProviderFor` scaffolding — with the type the
  context actually supplies absent from the message and the caret on the `#[cgp_type]` attribute. Its
  projection recovery is now general, so the failure becomes a `[CGP-E017]` header naming both types
  over a root-cause tree, with a `help` naming the wiring entry to change
  ([`abstract_type_mismatch`](../../tests/ui/acceptable/types/abstract_type_mismatch.rs)).

- A **capability published by a library** is reshaped like a local one. A
  `#[cgp_fn]`/`#[blanket_trait]` capability is recognized by its blanket impl over a bare context,
  and that signal alone is too broad (`ToString` and `Into` share it), so recognition was gated to
  traits the checked crate defines — which excluded every *published* capability along with the std
  blankets it was aimed at, and stopped the `[CGP-E009]` reshaping at the crate boundary. A foreign
  trait now qualifies on evidence that its blanket depends on a CGP construct instead
  ([`upstream_capability_use_site`](../../tests/ui/acceptable/use-site/upstream_capability_use_site.rs)).
- A **CGP construct rustc split across styled fragments** is resugared. rustc builds its "similar
  impl" hint from fragments split at every difference between the two traits, shredding a
  `Symbol<3, Chars<'B', …>>` so no fragment matches — the header would read `Symbol!("Bar")` while
  the hint beside it showed the raw spine. The fragments are now read as the one line they render
  as, and flattened only when that recovers something
  ([`upcast_missing_variant`](../../tests/ui/usability/extensible-data/upcast_missing_variant.rs)
  shows it on a declined diagnostic).

One presentation decision was deliberately **reversed**. A dispatch chain restates a program-sized
`Code` type at every hop, and a hop repeating its parent's trait exactly used to render as
`Handler<…>` to shorten it. That elision is gone: it hid the very type a reader follows the chain to
trace, and left a genuine repeat indistinguishable from a hop whose parameters differ. Every CGP
construct is now shown as written, and the length that costs on a DSL-sized program is accepted — the
[cross-block elision](../implementation/dependency-graph-rendering.md#eliding-across-blocks), which
drops whole subtrees an earlier block already drew, is the mechanism for brevity that does not
sacrifice precision.

What remains below are the classes the tool does not yet reshape.

## An unconstrained per-entry generic emits two contradictory errors

A `delegate_components!` entry whose generic parameter is used only in the value
(`<T> GreeterComponent: GreetWith<T>`) is rejected with `E0207` *twice*, at the same caret, and the
two errors carry contradictory auto-fixes: the first suggests adding the parameter to the context
type, the second suggests removing it
([`unconstrained_generic`](../../tests/ui/usability/wiring/constraints/unconstrained_generic.rs)).
Only the second fix matches the actual mistake. Coalescing the pair — or at least suppressing the
misleading first suggestion — is the work here; it is the last of the wiring-conflict shapes that
still passes through with only light post-processing.

## Macro lowering errors point at the attribute, not the cause

When a macro lowers accepted input into ill-formed Rust, the error lands on the macro attribute and
never states the real cause.
[`option_slice`](../../tests/ui/usability/lowering/option_slice.rs) produces an unsized-type failure
— an `Option<[u8]>` generated from an auto-getter returning `&[u8]` — as two cascading errors both
anchored on the `#[cgp_auto_getter]` attribute, and
[`use_type_cyclic_context`](../../tests/ui/usability/lowering/use_type_cyclic_context.rs) reports
`cannot find type A`/`B` without ever saying the `#[use_type]` routing is cyclic. The cause is hinted
by the spans but never named; the `use_type_cyclic_context` case in particular trends toward a hidden
cause, since nothing in the output states "cycle" (its counterpart
[`use_type_unknown_assoc`](../../tests/ui/acceptable/lowering/use_type_unknown_assoc.rs) is
`acceptable/` precisely because rustc already names the typo and its fix). Recognizing the lowering
class and naming the offending construct is the work here.

## Extensible-data failures are not reshaped at all

The casts, builders, and extractors of the
[extensible-data](https://github.com/contextgeneric/cgp/blob/main/docs/concepts/extensible-records.md)
family have no reshaping and no upstream error class. A `CanUpcast` into a target missing one variant
([`upcast_missing_variant`](../../tests/ui/usability/extensible-data/upcast_missing_variant.rs))
reports an internal `FromVariant` bound, puts its caret on the *wrong* variant, exposes the
macro-generated `__PartialSmall<IsVoid, IsPresent>` extractor state, and hides a requirement — while
never stating the mistake, that one enum has a variant the other lacks. The mismatch is pure
type-level list algebra the compiler holds exactly, so the class suits the typed resolver: a leaf
naming the absent variant (or the unbuildable field) is the work here.

## What good presentation looks like

Taken together, these issues define the tool's presentation target for the classes it does not yet
reshape: collapse the `E0207` unconstrained-generic pair so only the fix that matches the mistake
survives; give the extensible-data family a root cause naming the absent variant or field; and
recover the untransformed lowering class into a coded, root-cause-first diagnostic, or
at least name the offending construct instead of the macro attribute. The bar is the one the check-trait-failure family already meets:
lead with the cause as one plain sentence, name the decoded construct, give a short dependency path,
and never let a misleading `rustc` heuristic outrank the real cause. The
[CGP-side tooling notes](https://github.com/contextgeneric/cgp/blob/main/docs/errors/checks/check-trait-failure.md#how-cargo-cgp-presents-it)
describe the same extraction from the catalog side and are the reference to build against.
