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
  and [`parallel_branches`](../../tests/ui/acceptable/fields/parallel_branches.rs)).
- A **resolved dispatch chain restating the full program type at every node** is elided: a hop that
  exactly repeats its predecessor's trait and parameters renders as `Handler<…>`, so a DSL-sized
  chain reads as its meaningful steps
  ([`deep_dispatch_chain`](../../tests/ui/acceptable/verbosity/deep_dispatch_chain.rs)).
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
  — the only point after every diagnostic has arrived — grouping consumer failures by a
  consumer-independent cause signature, so
  [`density_3`](../../tests/ui/acceptable/duplication/density_3.rs) (two components, one missing
  `height`), [`dependency_cascade`](../../tests/ui/acceptable/duplication/dependency_cascade.rs)
  (three chained providers), and `missing_normal_bound` (two consumers sharing an `App: Clone` bound)
  each collapse to one block, and cargo's re-count keeps the "N errors" summary honest.

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

## What good presentation looks like

Taken together, these issues define the tool's presentation target for the classes it does not yet
reshape: collapse the `E0207` unconstrained-generic pair so only the fix that matches the mistake
survives; and recover the untransformed lowering class into a coded, root-cause-first diagnostic, or
at least name the offending construct instead of the macro attribute. The bar is the one the check-trait-failure family already meets:
lead with the cause as one plain sentence, name the decoded construct, give a short dependency path,
and never let a misleading `rustc` heuristic outrank the real cause. The
[CGP-side tooling notes](https://github.com/contextgeneric/cgp/blob/main/docs/errors/checks/check-trait-failure.md#how-cargo-cgp-presents-it)
describe the same extraction from the catalog side and are the reference to build against.
