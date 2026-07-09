# Usability issues

This document lists the ways `cargo-cgp` presents errors that carry the root cause but bury it — the
problems a reader hits even when the diagnostic contains everything needed to find the cause. What
separates these from a [hidden root cause](hidden-root-cause.md) is that the information is present:
the cause could be recovered from the output by a careful reader or a post-processor, so the work
here is re-presentation, not recovery. Every issue is backed by a fixture under
[`tests/ui/usability/`](../../tests/ui/usability).

The hand-curated fixtures at the top level of that directory are the worked examples this document's
prose walks through — all instances of one upstream class, a missing context field surfaced through
`check_components!` (the catalog's
[check-trait failure](../../../cgp/docs/errors/checks/check-trait-failure.md) and its
[verbose cascade](../../../cgp/docs/errors/checks/verbose-cascade.md) form), except
[`unsatisfied_dependency`](../../tests/ui/usability/unsatisfied-dependency/unsatisfied_dependency.rs), the consumer-call form
whose cause the next-gen solver recovers. The fixtures are sorted into kind subdirectories —
[`checks/`](../../tests/ui/usability/checks), [`wiring/`](../../tests/ui/usability/wiring),
[`lowering/`](../../tests/ui/usability/lowering), and
[`unsatisfied-dependency/`](../../tests/ui/usability/unsatisfied-dependency) — mirroring the upstream
catalog's own sections. Alongside the curated examples, those directories hold a verbatim mirror of
the *whole* upstream compile-fail suite, one fixture per reproducible CGP error class; importing it
confirmed that every reproducible class carries its root cause in cargo-cgp's output, which is why the
entire mirror sits under this category rather than under [hidden root cause](hidden-root-cause.md). The
class-by-class findings and the four fixtures not importable in a single-crate harness are recorded in
the [usability fixtures README](../../tests/ui/usability/README.md).

## Overly verbose error messages

The dominant usability problem is sheer volume: a single mistake produces far more output than there
is to fix. A CGP error interleaves the actual cause with generated-type scaffolding the user never
wrote — `CanUseComponent`, `IsProviderFor`, the `__CheckRectangle` check trait, and `N redundant
requirement hidden` notes — so even a single-field mistake spans a screen of `required for …` frames
around one relevant line. The tool should suppress the scaffolding and lead with the cause.

Worse, the error *count* reflects the depth of the wiring graph rather than the number of mistakes.
[`density_3.rs`](../../tests/ui/usability/checks/density_3.rs) checks both `AreaCalculatorComponent` and
`DensityCalculatorComponent`, and the one missing `height` field produces *two* full `E0277`
cascades that a reader must get through before realizing they describe the same fix
([`.stderr`](../../tests/ui/usability/checks/density_3.stderr)). The tool should deduplicate — coalesce
every block whose cause is the same unmet bound into one headline and report the count of affected
components rather than repeating the cascade.

## The primary error can be misleading, not just verbose

When broken wiring is exercised by a direct method call, the loudest line points the reader the wrong
way. [`unsatisfied_dependency.rs`](../../tests/ui/usability/unsatisfied-dependency/unsatisfied_dependency.rs) calls
`greet()` on a context that cannot satisfy the provider's `Self: HasName`, and the
[`.stderr`](../../tests/ui/usability/unsatisfied-dependency/unsatisfied_dependency.stderr) leads with `E0599` "method
`greet` not found … this is an associated function, not a method" and even suggests "use associated
function syntax instead" — advice that is wrong for a wiring error. The real cause is present further
down, because `cargo-cgp`'s next-gen solver surfaces the unmet `HasField<…name…>` bound and an "add
`#[derive(HasField)]`" hint (which is exactly why this fixture is a usability case and not a
[hidden root cause](hidden-root-cause.md)). The tool should promote that recovered cause to the
headline and drop the method-versus-associated-function misdirection.

## The field name is an encoded type-level string

The single most important fact in each error — which field is missing — is present but written as a
type the reader must decode character by character. In
[`base_area_2.rs`](../../tests/ui/usability/checks/base_area_2.rs) a missing field appears as
`HasField<Symbol<5, Chars<'w', Chars<'i', Chars<'d', Chars<'t', Chars<'h', Nil>>>>>>>`, which spells
the name out but demands the reader (or an IDE hover) reassemble it.
[`base_area_1.rs`](../../tests/ui/usability/checks/base_area_1.rs) shows the same encoding through the
two-line "similar impl" hint — `HasField<Symbol<6, Chars<'h', …, Chars<'t', Nil>>>>` for the missing
`height` — the shape that reports one near-miss field against another. The tool should render either
back as `Symbol!("height")`, or better as the plain field name `height`.

This is a readability burden, not an insufficiency: the full string is present, so it *can* be read.
That is only reliably true because the driver injects `--verbose`. Without it, the two-line hint in
`base_area_1` diffs the two symbols and elides the character they share, dropping the `'h'` out of
both names and leaving `height` genuinely unreadable — which is why that fixture used to be a
[hidden root cause](hidden-root-cause.md#defeated-a-field-name-the-printer-elided-a-character-from)
and, now that the flag defeats the elision, is a usability case here. The mechanism is documented in
[rustc diagnostic internals](../implementation/rustc-diagnostic-internals.md#matching-generic-arguments-are-elided-to-_).

## The root cause is never stated plainly

No line in any fixture says, in words, what the mistake is. In
[`base_area_2.rs`](../../tests/ui/usability/checks/base_area_2.rs) the reader assembles "`Rectangle` is
missing an accessible `width` field, which `RectangleArea` needs through `HasRectangleFields`" out of
three separate fragments: the `help:` note naming the unmet `HasField` bound, the `note: required
for Rectangle to implement HasRectangleFields`, and the caret on the struct definition. The tool
should emit that one sentence as the headline and demote the fragments to supporting detail, the way
Clippy leads with a plain statement of a lint before its span.

## The dependency path is not summarized

When the checked component is several hops from the failing field, the path connecting them is
present but scattered through scaffolding. In [`density_1.rs`](../../tests/ui/usability/checks/density_1.rs)
the check names `DensityCalculatorComponent`, yet the missing `height` field belongs to a transitive
`AreaCalculator` dependency, and the [`.stderr`](../../tests/ui/usability/checks/density_1.stderr) traces
the connection only through a stack of `required for …` notes punctuated by `1 redundant requirement
hidden`. [`density_2.rs`](../../tests/ui/usability/checks/density_2.rs) adds a `ScaledArea` layer and shows
the chain growing longer with no new cause. The tool should collapse either into a short, readable
path — `DensityCalculatorComponent → AreaCalculator → missing field height` — reconstructed from the
chain rather than dumped.

## The failing layer of a higher-order provider is not spelled out

When a provider wraps another, the output identifies the failing layer but does not say so outright,
so the reader must infer it. [`scaled_area_1.rs`](../../tests/ui/usability/checks/scaled_area_1.rs) and
[`scaled_area_2.rs`](../../tests/ui/usability/checks/scaled_area_2.rs) both wire `ScaledArea<RectangleArea>`
and look nearly identical, but in the first the *inner* `RectangleArea` is missing `height` and in
the second the *outer* `ScaledArea` is missing its own `scale_factor`. The distinguishing signal is
present — which provider's `where` clause the "introduced here" caret sits on, and how deep the
`required for …` chain runs — but the reader has to know to read it. The tool should name the layer,
the way the upstream `#[check_providers(...)]` form does by hand (see the catalog's
[higher-order provider layer failure](../../../cgp/docs/errors/checks/higher-order-provider-layer.md)).

## A missing derive reads like a missing field

The absence of `#[derive(HasField)]` altogether looks, at a glance, like a single missing field, even
though the fix is entirely different. [`base_area_2.rs`](../../tests/ui/usability/checks/base_area_2.rs)
omits the derive, so `Rectangle` has *no* `HasField` impls, and the
[`.stderr`](../../tests/ui/usability/checks/base_area_2.stderr) reports only the first field (`width`) and —
unlike every other fixture — carries no "but trait `HasField<…>` is implemented for it" landmark.
That absent landmark is the signal, and it is in the output, so a tool can detect it: a context with
zero `HasField` impls behind a `#[derive(HasField)]`-shaped requirement has most likely forgotten the
derive. The tool should say so, rather than sending the user to add one field at a time.

## What good presentation looks like

Taken together, these issues define the tool's presentation target for this class: lead with the
root cause as one plain sentence, name the decoded field, give a short dependency path, name the
failing provider layer, deduplicate a cascade down to its distinct causes, and never let a
misleading `rustc` heuristic outrank the real cause — with the
`IsProviderFor`/`CanUseComponent`/`__Check…` scaffolding suppressed throughout. The
[upstream tooling notes](../../../cgp/docs/errors/checks/check-trait-failure.md#notes-for-tooling)
for this class describe the same extraction from the CGP side and are the reference to build against.
When a fixture here reaches that bar, it graduates from `tests/ui/usability/` into `tests/ui/ok/` and
its issue is deleted from this document.
