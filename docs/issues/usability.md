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
mismatch), states the cause as one plain sentence (`root cause: missing field \`height\` on
\`Rectangle\``), decodes the field name from its `Symbol!`, and renders the dependency path as a
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

The tool does not deduplicate a single cause that surfaces at several wiring sites, so one mistake
produces several full error blocks. [`density_3`](../../tests/ui/usability/duplication/density_3.rs)
checks two components against one missing `height` field and gets *two* complete `E0277` cascades;
[`dependency_cascade`](../../tests/ui/usability/duplication/dependency_cascade.rs) chains three
providers and gets three. The error count reflects the depth of the wiring graph, not the number of
mistakes. This is a cross-diagnostic transform the per-diagnostic resolver cannot do on its own: it
needs the emitter to buffer the compilation's diagnostics and coalesce every block whose recovered
cause is the same unmet bound into one headline that reports the count of affected components.

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

## Abstract-type imports fail untransformed and leak generated names

A `#[use_type]` abstract-type dependency that cannot be satisfied is not recognized by the resolver,
so it falls through untransformed — no `[CGP-Exxx]` code, no root-cause note — and rustc's "the
following other types implement …" help then exposes the generated placeholder identifiers
`__Context__`, `__Components__`, `__Path__`, and `__Provider__` that the user never wrote.
[`use_type_foreign_unsatisfied`](../../tests/ui/usability/use-type/use_type_foreign_unsatisfied.rs)
shows the placeholder leak, and
[`use_type_nested_unsatisfied`](../../tests/ui/usability/use-type/use_type_nested_unsatisfied.rs)
adds a second error block for the one cause and a misleading `add #![feature(trivial_bounds)]`
suggestion — a rustc heuristic that is wrong for a wiring error, the same kind of misdirection the
resolver already strips from the consumer-call case. At minimum the post-processing should strip the
`__…__` placeholders the way it strips the CGP path prefixes; better, the resolver should learn to
recover this class into a coded root-cause note.

## Wiring coherence conflicts fan out and expose internal traits

A structural wiring mistake — a key or provider name wired twice, an overlapping generic, a namespace
path registered twice — surfaces as a coherence conflict (`E0119`) that the tool passes through with
only light post-processing, so one mistake becomes two near-identical blocks: one keyed on
`IsProviderFor<…>` and one on `DelegateComponent<…>`, both internal traits the user never wrote. The
duplicate-key family ([`tests/ui/usability/wiring/duplicate-keys`](../../tests/ui/usability/wiring/duplicate-keys))
and the namespace-path family
([`tests/ui/usability/wiring/namespace-paths`](../../tests/ui/usability/wiring/namespace-paths)) all
show this fan-out. Two adjacent constraint failures round out the class
([`tests/ui/usability/wiring/constraints`](../../tests/ui/usability/wiring/constraints)): the
unconstrained-generic `E0207` emits two errors with contradictory auto-fixes ("add the parameter"
versus "remove it"), and the `UseContext` cycle `E0275` exposes `CanUseComponent` / `__Check…` and
never names the cycle. A smaller gap rides along in the namespace-path headers: the conflicting key
prints as a raw `PathCons<Symbol!("foo"), PathCons<Symbol!("bar"), _>>` because the `Path!`
resugaring does not reach a path whose tail is an open `_`
([`namespace_duplicate_path_key`](../../tests/ui/usability/wiring/namespace-paths/namespace_duplicate_path_key.rs),
[`delegate_duplicate_path_key`](../../tests/ui/usability/wiring/namespace-paths/delegate_duplicate_path_key.rs)).
The tool should coalesce the paired blocks, suppress the internal traits, finish the `Path!`
resugaring for open-tailed paths, and name the wiring mistake behind each code.

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
cascade) or within one (the missing derive); recover the untransformed `#[use_type]` and lowering
classes into coded, root-cause-first diagnostics, or at least strip the generated `__…__` names and
misleading suggestions they leak; coalesce a coherence conflict's paired blocks and name the wiring
mistake behind its `E0119`/`E0207`/`E0275` code rather than exposing the internal traits; and finish
the `Path!` resugaring so no encoded type-level path survives in a header. The bar is the same one
the check-trait-failure family already meets: lead with the cause as one plain sentence, name the
decoded construct, give a short dependency path, and never let a misleading `rustc` heuristic outrank
the real cause. The
[upstream tooling notes](../../../cgp/docs/errors/checks/check-trait-failure.md#notes-for-tooling)
describe the same extraction from the CGP side and are the reference to build against.
