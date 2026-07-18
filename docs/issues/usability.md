# Usability issues

This document lists the ways `cargo-cgp` presents errors that carry the root cause but bury it — the
problems a reader hits even when the diagnostic contains everything needed to find the cause. What
separates these from a [hidden root cause](hidden-root-cause.md) is that the information is present:
the cause could be recovered from the output by a careful reader or a post-processor, so the work
here is re-presentation, not recovery. Every issue is backed by a fixture under
[`tests/ui/usability/`](../../tests/ui/usability), grouped into a sub-directory that names the issue
class. When a fixture's presentation reaches the bar, it graduates out of `usability/` into
[`tests/ui/acceptable/`](../../tests/ui/acceptable) and its issue is deleted from this document.

## What the typed resolver already fixed

The check-trait-failure family — the dominant class of CGP error — is now presented well, so its
fixtures have graduated into [`acceptable/`](../../tests/ui/acceptable). The driver's
[typed root-cause resolver](../implementation/typed-root-cause-resolution.md) leads with a single
coded headline (`[CGP-E001]` for an unimplemented consumer trait, `[CGP-E003]` for a field-type
mismatch), states the cause as one plain sentence (`` root cause: missing field `height` on
`Rectangle` ``), decodes the field name from its `Symbol!`, and renders the dependency path as a
compact `cargo tree`, with the `IsProviderFor` / `CanUseComponent` / `__Check…` scaffolding
suppressed throughout. The worked examples this document once walked through in prose all clear that
bar now and live under `acceptable/`: the encoded missing field
([`base_area_1`](../../tests/ui/acceptable/fields/base_area_1.rs)), the buried dependency path
([`density_1`](../../tests/ui/acceptable/providers/density_1.rs)), the higher-order layer
([`scaled_area_1`](../../tests/ui/acceptable/providers/scaled_area_1.rs) versus
[`scaled_area_2`](../../tests/ui/acceptable/providers/scaled_area_2.rs)), and the misleading
consumer-call error whose `E0599` "use associated function syntax" advice is now dropped
([`unsatisfied_dependency`](../../tests/ui/acceptable/use-site/unsatisfied_dependency.rs)). What
remains below are the classes the resolver does not yet reshape.

## One mistake reported as many errors

The emitter now de-duplicates *the same failing consumer* re-reported at several sites, which was the
dominant form of this class. CGP wiring is lazy, so one missing dependency surfaces at the
`check_components!` entry, at every hand-written `impl` that references the broken consumer, and at
each call — the transfer example's single un-wired password type produced eighteen identical
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

## A resolved dispatch chain repeats the full program type at every node

The use-site failure that used to expose combinator plumbing now *resolves*, through the
[call-site anchor](../implementation/typed-root-cause-resolution.md#recovering-from-the-call-expression-itself):
what used to be several `[CGP-E002]`-plumbing blocks with no cause is now one `[CGP-E001]` block led
by the root cause (the graduated
[`cascade_after_use_site`](../../tests/ui/acceptable/use-site/cascade_after_use_site.rs) pins the
class, its `?`-operator cascade still suppressed and its await-site re-report de-duplicated).

What remains is how that success *reads* on a program-sized `Code` type. Every `Handler` node of the
recovered chain restates the entire `Prog<Product![…]>` code type, and every namespace or dispatch
level adds a `redirect lookup` hop, so a realistic DSL context (the
[shell-scripting DSL](../../../cgp/docs/examples/shell-scripting-dsl.md)'s dynamic-argument example)
yields a tree of ~30 nodes whose lines differ only in the provider column — the cause leads, but the
chain buries the wiring path a reader might actually want in sheer repetition.
[`deep_dispatch_chain`](../../tests/ui/usability/verbosity/deep_dispatch_chain.rs) distills the
shape. The open work is presentation: elide a code parameter that is unchanged from the node above
(`Handler<…, _>` on first appearance, `Handler<…>` or an ellipsis after), or fold a
redirect-hop/provider pair that adds no branching into one line, so a deep chain reads as its
handful of meaningful steps.

## A use-site failure whose arguments write no types keeps rustc's misleading method advice

A generic consumer that fails at a direct call now resolves whenever the call *writes* the dispatch
parameter's type somewhere the
[call-site anchor](../implementation/typed-root-cause-resolution.md#recovering-from-the-call-expression-itself)'s
signature unification can read it (the graduated
[`generic_consumer_use_site`](../../tests/ui/acceptable/use-site/generic_consumer_use_site.rs) pins
the value-argument case). What still declines — the anchor's documented boundary — is a call whose
parameter-carrying argument the call does not type syntactically, a plain variable or an unsuffixed
literal, since typing it would need the typeck results the emitter can never force.
[`generic_consumer_unwritten_arg`](../../tests/ui/usability/use-site/generic_consumer_unwritten_arg.rs)
pins the class: the declined `E0599` keeps rustc's method-probe framing, whose "this is an associated
function, not a method" caret and "use associated function syntax instead" suggestion — both
artifacts of CGP's `self`-less provider methods, and the second actively wrong — outrank the real
`HasField<Symbol!("separator")>` cause buried in the first note. (The same failure at a
`check_components!` entry resolves cleanly.) The plausible recovery is to re-check the wired
delegate's *implemented* parameter values — the provider's concrete impls name them — instead of the
meaningless `()` form; short of that, the fallback should at least drop the method-syntax advice the
way the resolver already does for the shapes it reshapes.

## One missing derive reported field by field

A struct missing its `#[derive(HasField)]` has no `HasField` impl for *any* field, so the resolver
recovers each field the providers read as a separate root cause even though the single fix is one
derive. [`base_area_2`](../../tests/ui/usability/duplication/base_area_2.rs) reports both `height`
and `width` as distinct `root cause:` notes, each with its own dependency chain, on top of the one
`help` that already names the fix. The deduplication key here is not the unmet bound — the bounds
differ (`HasField<Symbol!("height")>` versus `HasField<Symbol!("width")>`), so the same-bound rule
above cannot catch it — but the shared *fix*: when every recovered cause is an underived field on the
same struct, they should collapse to one "add the derive" note. The subtlety is that a struct whose
fields are genuinely absent (`empty_field_struct`, `parallel_branches`, both now in
[`acceptable/fields`](../../tests/ui/acceptable/fields)) correctly stays several causes — several real
fixes — so the coalescing must key on the derive being present-but-empty, not merely on there being
more than one field cause.


## Some wiring coherence conflicts still fan out or expose internal traits

A structural wiring mistake surfaces as a coherence conflict (`E0119`/`E0207`/`E0275`), and rustc
often reports it *twice* — one block keyed on `IsProviderFor<…>` and one on `DelegateComponent<…>`,
both internal traits the user never wrote. The **duplicate delegate-key** case is now handled: the
tool recognizes the `DelegateComponent`/`IsProviderFor` `E0119` pair, drops the redundant
`IsProviderFor` half, and rewrites the `DelegateComponent` half into a coded
`[CGP-E004]`–`[CGP-E008]` headline that names the colliding key(s) — one code per shape: the same key
twice, an overlapping generic, multiple namespaces, a duplicated `@`-path, or a redirect collision —
so those fixtures have graduated to
[`tests/ui/acceptable/wiring`](../../tests/ui/acceptable/wiring). What remains here is the conflicts
that are *not* a duplicate delegate key.

Two of them still pass through with only light post-processing. A duplicate provider *name*
([`duplicate_provider_name`](../../tests/ui/usability/wiring/duplicate-keys/duplicate_provider_name.rs))
is an `E0428` plus a `Greeter`/`IsProviderFor` `E0119` pair on the provider *struct* — not a
`DelegateComponent` conflict, so the `CGP-E004`–`CGP-E008` handling does not reach it and its pair
still fans out. A duplicate `cgp_namespace!` `@`-path
([`namespace_duplicate_path_key`](../../tests/ui/usability/wiring/namespace-paths/namespace_duplicate_path_key.rs))
is a *single* `E0119` on the user's own namespace trait (`MyNamespace<_>`); its path resugars to the
readable `Path!(@foo.bar.*)`, but the header still exposes the raw namespace-trait conflict, uncoded.

Two adjacent constraint failures round out the class
([`tests/ui/usability/wiring/constraints`](../../tests/ui/usability/wiring/constraints)): the
unconstrained-generic `E0207` emits two errors with contradictory auto-fixes ("add the parameter"
versus "remove it"), and the `UseContext` cycle `E0275` exposes `CanUseComponent` / `__Check…` and
never names the cycle. The tool should coalesce or suppress these remaining paired blocks and name the
wiring mistake behind each code, as it now does for the duplicate delegate-key `E0119`.

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
reshape: deduplicate one cause down to a single headline whether it fans out across diagnostics (the
cascade) or within one (the missing derive); recover the untransformed lowering class into a coded,
root-cause-first diagnostic, or at least strip the generated `__…__` names and misleading suggestions
it leaks; surface the generic consumer's use-site failure once its value-carried parameter becomes
recoverable; compact the deep dispatch chains the call-site anchor now resolves; and coalesce the
coherence conflicts that still fan out — the
duplicate provider *name*, the `E0207` unconstrained generic, and the `E0275` `UseContext` cycle —
and name the wiring mistake behind each code, the way the duplicate delegate-key `E0119` is now
reshaped into the `[CGP-E004]`–`[CGP-E008]` family. The bar is the same one
the check-trait-failure family already meets: lead with the cause as one plain sentence, name the
decoded construct, give a short dependency path, and never let a misleading `rustc` heuristic outrank
the real cause. The
[upstream tooling notes](../../../cgp/docs/errors/checks/check-trait-failure.md#notes-for-tooling)
describe the same extraction from the CGP side and are the reference to build against.
