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

What remains below are the classes the tool does not yet reshape.

## One mistake reported as many errors

The emitter de-duplicates *the same failing consumer* re-reported at several sites, which was the
dominant form of this class. CGP wiring is lazy, so one missing dependency surfaces at the
`check_components!` entry, at every hand-written `impl` that references the broken consumer, and
at each call — the transfer example's single un-wired password type produced eighteen identical
root-cause trees. The emitter keeps a span-independent signature of each transformed CGP diagnostic
(the recovered cause for a resolved one, the rendered text for a declined-but-rewritten one) and shows
only the first occurrence, so those eighteen collapse to two (one per endpoint) and cargo's re-count
keeps the "N errors" summary honest. The cross-site behavior is pinned by
[`cross_site_dedup`](../../tests/ui/acceptable/duplication/cross_site_dedup.rs) (a check entry, a
wrapper `impl`, and its forwarding call for one missing field → one block) and
[`manual_supertrait_impl`](../../tests/ui/acceptable/use-site/manual_supertrait_impl.rs) (an impl
header and its call → one). See [The driver](../implementation/driver.md) for the mechanism.

What remains is coalescing *different* consumers that share one cause. The signature includes the
consumer deliberately — so a distinct capability's failure is never hidden — which means two
components failing for the same underlying mistake still produce two blocks:
[`density_3`](../../tests/ui/usability/duplication/density_3.rs) checks two components against one
missing `height` field and still gets two blocks, and
[`dependency_cascade`](../../tests/ui/usability/duplication/dependency_cascade.rs) chains three
providers and gets three. Collapsing these into a single headline that *lists the affected
components* — rather than dropping all but one — needs the emitter to buffer the compilation's
diagnostics before emitting, so it can name every affected consumer in the one surviving block; that
buffering step is the open work here.

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

## A cross-context dependency leaks internals and under-decodes its leaf

A **cross-context dependency** — one context's wiring depending on a *concrete* other context, so
the same obligation appears in two contexts' dependency trees — is resolved well in one respect and
poorly in two. [`cross_context_node_key`](../../tests/ui/usability/resolution/cross_context_node_key.rs)
wires an `Outer` whose provider carries a `where Inner: CanCompute` clause on a second context `Inner`
that lacks the field its own provider needs. The part that already works is the reason the resolution
cache keys each node on its root context: the shared `Inner: CanCompute` node renders as a
consumer-trait impl in `Inner`'s own tree but as a plain trait bound in `Outer`'s, so neither tree
borrows the other's context-specific labels.

Two presentation problems remain. First, the failure that anchors on the provider impl's own `where`
clause resolves through the impl-site anchor onto that *provider* impl, so its tree leaks the reserved
`__Context__` placeholder and an `IsProviderFor` hop — internals every other transformed diagnostic
suppresses. The [documented resolver boundary](../implementation/typed-root-cause-resolution.md#boundaries-and-open-ends)
is that a caret on a provider struct's own impl should decline; a concrete-context `where` clause
slips past it. Second, `Outer`'s tree descends past the foreign-context node and bottoms out on an
ordinary `[CGP-E201] Inner: HasName` bound with a spurious `for provider Inner` hop, rather than the
decoded `[CGP-E106] missing field` the sibling blocks show. Declining the provider-impl anchor and
decoding the foreign-context field leaf are the work here.

## What good presentation looks like

Taken together, these issues define the tool's presentation target for the classes it does not yet
reshape: coalesce *different* consumers that share one cause into a single headline that lists the
affected components (the cascade class, which needs diagnostic buffering); collapse the `E0207`
unconstrained-generic pair so only the fix that matches the mistake survives; recover the
untransformed lowering class into a coded, root-cause-first diagnostic, or at least name the
offending construct instead of the macro attribute; and keep the cross-context dependency's tree free
of `__Context__`/`IsProviderFor` internals while decoding its foreign-context field leaf. The bar is the one the check-trait-failure
family already meets: lead with the cause as one plain sentence, name the decoded construct, give a
short dependency path, and never let a misleading `rustc` heuristic outrank the real cause. The
[upstream tooling notes](../../../cgp/docs/errors/checks/check-trait-failure.md#notes-for-tooling)
describe the same extraction from the CGP side and are the reference to build against.
