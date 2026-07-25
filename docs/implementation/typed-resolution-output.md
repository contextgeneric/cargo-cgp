# Typed resolution: the transformed diagnostic

This document covers the output side of the driver's
[typed root-cause resolution](typed-root-cause-resolution.md): the two halves a resolved failure's
diagnostic is rebuilt from — the coded headline and the `root cause:` notes — and how the emitter
applies the rustc-free plan to the compiler's `DiagInner`. The recovery that produces the causes
being worded here is the subject of [the anchors](typed-resolution-anchors.md) and
[the walk](typed-resolution-walk.md).

## The coded headline

The main message is rewritten into a coded CGP class only when the resolution identifies one; the
Rust error code (`E0277`, `E0599`, `E0271`) is always kept. Five classes cover the cases:

- **`[CGP-E001]`, the consumer form**, for a context that cannot implement a consumer trait — a
  [check-trait failure](https://github.com/contextgeneric/cgp/blob/main/docs/errors/checks/check-trait-failure.md). It names the consumer
  trait and the context, as in the worked example's headline
  `the consumer trait \`CanCalculateArea\` is not implemented for context \`Rectangle\``. The consumer
  name comes straight off the consumer trait's `DefId`, so it is exact even when two components share a
  name in different modules. A consumer-method `E0599` (whose text names no wiring trait) and a
  `RedirectLookup`-provider failure (below) also take this form.
- **`[CGP-E002]`, the provider form**, for a rustc header that opens on an unsatisfied `IsProviderFor`
  bound — worded by the text rewrite as
  `the provider trait \`X\` with context \`Ctx\` for provider \`P\``. It fires when a real, wired
  provider's own dependency fails and rustc's own headline names
  that provider's `IsProviderFor` (a `#[check_providers]` layer, say). The one exception is when the
  "provider" in the bound is a **`RedirectLookup`**: that is redirect plumbing the programmer never
  wrote (the lookup resolved to *no* provider, so the wiring is simply missing), so the header follows
  the redirect through to the recovered consumer and takes the `[CGP-E001]` form instead of exposing
  `RedirectLookup<App, @…>` as a provider.
- **`[CGP-E003]`, the field-type-mismatch form**, for an `E0271` the resolver traced to a `HasField`
  projection — a field whose name matches but whose type does not.
- **`[CGP-E017]`, the abstract-type-mismatch form**, for an `E0271` the resolver traced to any *other*
  associated-type projection — most often a CGP abstract type the context binds one way while a
  provider pins it another. It is `[CGP-E003]`'s sibling in every respect but the trait the projection
  sits on, and is worded in the same shape.
- **`[CGP-E009]`, the plain-trait form**, for a hand-written wrapper trait that is *not* itself a CGP
  consumer (it has only a concrete impl, no consumer blanket) — `CanHandleApiSend`, say. It reads
  `the trait \`…\` is not implemented for \`…\``, distinguished from `[CGP-E001]` by the wrapper's
  fingerprint (see [the impl-site and wrapper-chain anchors](typed-resolution-anchors.md)).

The field-type-mismatch class is worth showing, because its headline is unusually specific. Reusing
the area example but giving `Rectangle` a `height` of the wrong type:

```rust
#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: i32, // `RectangleArea` needs `f64`
}
```

`RectangleArea` reads `height` as an `#[implicit]` argument of type `f64`, so `HasField<Symbol!("height")>`
*is* implemented for `Rectangle` (it derives it) but with `Value = i32`; the trait bound holds, and
only the associated-type projection `<Rectangle as HasField<Symbol!("height")>>::Value == f64` fails,
an `E0271`. The **root cause is the type mismatch on the `height` field**, and the resolver reads the
expected type from the failing projection and the actual type off the struct:

```text
error[E0271]: [CGP-E003] expected a `height` field of type `f64` on `Rectangle`, but found `i32`
   = note: this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
             └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
               └─ [CGP-E109] field `height` on `Rectangle` has type `i32`, but `f64` is required
```

Two cases keep rustc's own headline rather than a coded one. A main message that already states a
**genuine recovered leaf** — an ordinary bound such as `f64: Eq` the next-generation solver descended
to, which is itself the root cause — is left untouched, header and caret and all, because it already
names the cause. But a main message that states a *mid-chain symptom* — an ordinary bound that is not
one of the recovered leaves, such as a getter bound on a request whose real cause is a missing wiring
a level down — is replaced by the truer `[CGP-E001]` consumer header. And a main message that is
neither a trait bound nor a resolved class (an unrelated `E0308`) is always kept.

## The root-cause notes

Whatever happens to the headline, the sub-messages are always replaced. rustc's obligation-chain
notes, supplementary help, and structured suggestions are discarded, and each recovered root cause
becomes one `= note:` opening with a `root cause:` lead that names the leaf, followed by the
dependency chain rendered as a tree. The chain **repeats the root cause as its own terminal leaf**, so
it always bottoms out *at* the cause rather than one step before it, whatever the leaf's kind — a
branch elided against an earlier block
([cross-block elision](dependency-graph-rendering.md#eliding-across-blocks)) keeps that terminus too,
and a note with no chain at all names the cause in its lead instead. Every
lead and every tree entry carries a [`CGP-E1xx`/`CGP-E2xx` code](../error-code.md) (the wording below
omits the prefix for brevity). Paths render as a bare `@app.GreeterComponent` — the `Path!(@…)` macro
form is reserved for the resugaring fallback — and module qualifiers are stripped throughout, so
`contexts::app::MockApp` reads as `MockApp`. The chain is indented two spaces under its heading.

A failure with **several root causes** — the usual shape, since every cause descends from the one
failing obligation the walk seeded — is always rendered as a **single** note, not one per cause: every
cause's paths are folded into one [dependency graph](dependency-graph-rendering.md), which merges the
nodes they share by structural identity (a node reached again is `(*)`-referenced) and branches where
they diverge, each branch ending at its own leaf. The graph fuses exactly the shared dependencies and
no more, so causes that share a common ancestor collapse the enormous restated prefix a program-sized
context type would otherwise repeat, while genuinely independent causes render as their own root trees
stacked in the one note. The heading lists the distinct leaves: a singular `root cause:` lead when they
all bottom out on the same leaf, or a `root causes:` list — each with its own code, so a reader sees
every cause at a glance — when they differ.

The `root cause:` lead is worded by *why* the leaf is unmet, and there are six leaf shapes:

- A **genuinely absent field** reads as `root cause: missing field \`height\` on \`Rectangle\`` — the
  worked example's leaf. There is no `context` qualifier, since `HasField` can land on any struct.
- A **present-but-underived field** — one the struct carries but that has no (or an incomplete)
  `#[derive(HasField)]` — reads as
  `root cause: accessor trait \`HasField\` with field \`name\` is not implemented for \`Person\``, and
  a separate `help` names the fix: `make sure that \`#[derive(HasField)]\` is used for \`Person\``.
  When the field is only reachable through a `Deref` target, the `help` points at that target instead
  ([`missing_has_field_derive`](../../tests/ui/acceptable/fields/missing_has_field_derive.rs),
  [`field_via_deref`](../../tests/ui/acceptable/fields/field_via_deref.rs)). Several such fields on
  *one* struct are one mistake — the derive emits an impl per field — so `plan_resolved` coalesces
  them (`coalesce_underived_fields`) into a single cause reading
  `root cause: accessor trait \`HasField\` is not implemented for the fields \`height\` and \`width\` of \`Rectangle\``,
  over one merged tree whose branches still end at the per-field leaves
  ([`base_area_2`](../../tests/ui/acceptable/fields/base_area_2.rs)); a lone underived field, an
  underived field beside a genuinely missing one, and underived fields on *different* structs all
  stay apart
  ([`underived_and_missing_field`](../../tests/ui/acceptable/fields/underived_and_missing_field.rs)).
- A **missing wiring** — a component the context does not delegate to any provider — is the wiring
  counterpart of a missing field. It reads as
  `root cause: context \`App\` does not contain any delegate entry for \`BarProviderComponent\`` and
  names the component marker the programmer writes to fix it. The
  [`basic_missing_wiring`](../../tests/ui/acceptable/wiring/missing-wiring/basic_missing_wiring.rs)
  fixture is the shape: a provider `DoFooWithBar` declares `#[uses(CanUseBar)]`, so `App` can use it
  only if it also wires `BarProviderComponent`, but `App` wires only `FooProviderComponent`. The tree
  bottoms out on the `CanUseBar` capability the unwired component would have supplied:

  ```text
  error[E0277]: [CGP-E001] the consumer trait `CanUseFoo` is not implemented for context `App`
     = note: root cause: [CGP-E107] context `App` does not contain any delegate entry for `BarProviderComponent`
             this is required through the dependency chain:
               [CGP-E101] consumer trait impl `CanUseFoo` for context `App`
               └─ [CGP-E102] provider trait impl `FooProvider` with context `App` for provider `DoFooWithBar`
                 └─ [CGP-E101] consumer trait impl `CanUseBar` for context `App`
                   └─ [CGP-E107] context `App` does not contain any delegate entry for `BarProviderComponent`
  ```

  Note the nested `CanUseBar` node: because the walk descends the real capability the provider
  `#[uses]`, an intermediate consumer trait reads as `consumer trait impl` (`CGP-E101`), not the
  generic `trait impl` (`CGP-E105`) a plain getter or bound gets.

- A **missing redirect wiring** — a namespace/`open` redirect that resolves to nothing — is the
  path-keyed counterpart of a missing wiring, reading the *same* way but with the path as the key
  (`root cause: context \`App\` does not contain any delegate entry for \`@app.finance.types.QuantityTypeProviderComponent\``).
  Its chain renders each `RedirectLookup` hop as `redirect lookup to \`@…\` in \`App\``, so a
  multi-layer redirect reads as its successive hops down to the unterminated path. This leaf surfaces
  two ways that render identically: when the context *joins a namespace*, the unterminated redirect is
  an unmet namespace-lookup bound (`Path: DefaultNamespace<Ctx>`, or a user `cgp_namespace!` trait);
  when it dispatches a component with a bare `open` statement, the redirect looks the path up in the
  context's own table, so the failure is an unmet `DelegateComponent<PathCons<…>>` on the context —
  told apart from a plain missing component (whose key is a bare marker) by the `PathCons` key, and
  rendered as the whole path rather than its flattened item name. The
  [`unregistered_prefix_path`](../../tests/ui/acceptable/resolution/unregistered_prefix_path.rs),
  [`qualified_prefix_path`](../../tests/ui/acceptable/wiring/namespace-paths/qualified_prefix_path.rs),
  [`multi_redirect_missing`](../../tests/ui/acceptable/wiring/namespace-paths/multi_redirect_missing.rs),
  and [`open_missing_type_key`](../../tests/ui/acceptable/wiring/namespace-paths/open_missing_type_key.rs)
  fixtures pin the variants.

- A **missing dispatch entry** — a *non-context* delegation table missing a key — is the wiring
  counterpart for a provider table rather than the context. It reads as
  `root cause: [CGP-E110] provider \`ToTokioAsyncReadHandlers\` does not contain any delegate entry for \`GenericArray<u8, …>\``
  and names the table and the key. The owner is either an aggregate provider missing a component
  wiring or a `UseDelegate`/`UseInputDelegate` dispatch table missing a branch for the type it
  dispatches on (a `Code` fragment or an `Input` value's type); the two are recognized alike, by the
  owner carrying at least one `DelegateComponent` impl. This is the leaf a handler pipeline bottoms out
  on when a stage's output is not a type a later stage's input dispatcher handles — the
  `http_checksum_native` hypershell shape, where a raw `GenericArray` digest reaches an `AsyncRead`
  sink's input dispatcher because a byte-encoding stage is missing. The tree shows the offending type
  flowing into the stage, so a reader sees exactly what reached a stage that cannot handle it
  ([`cascade_nested_projection`](../../tests/ui/acceptable/use-site/cascade_nested_projection.rs) pins
  the shape; [`transitive_missing_wiring`](../../tests/ui/acceptable/wiring/missing-wiring/transitive_missing_wiring.rs)
  the aggregate-provider variant).

- A **non-provider** — a type wired where a provider was expected that does not implement the
  provider trait at all — reads as
  `root cause: [CGP-E111] the provider trait \`ApiHandler\` is not implemented for \`QueryBalanceRequest\``.
  The mistake is putting a non-provider (often a request or value type) into a provider slot, as the
  `money-transfer-api` example does when an endpoint's inner handler is dropped and
  `UseBasicAuth<QueryBalanceRequest>` leaves the *request* type where an `ApiHandler` belongs. It is
  the sibling of a missing dispatch entry for a type that is not a table at all: told apart from a
  valid-provider dead-end (a leaf provider reached via the blanket after an input mismatch, which
  *has* a concrete impl of the trait) by the owner having **no** concrete impl of the provider trait,
  and named against that trait rather than a wiring key
  ([`non_provider_wired`](../../tests/ui/acceptable/providers/non_provider_wired.rs) pins it).

- An **abstract-type mismatch** — an associated type the owner supplies differently from what a
  provider requires — reads as
  `root cause: [CGP-E112] abstract type \`Error\` of \`HasErrorType\` on \`App\` is \`String\`, but \`AppError\` is required`,
  and a separate `help` names the fix:
  `` wire `ErrorTypeProviderComponent` to `UseType<AppError>` in the wiring for `App`, or change the
  provider to work with `String` ``. It reads `associated type`, and carries no `help`, when the trait
  is not a CGP abstract-type component — an ordinary trait's associated type is fixed by whatever impl
  supplies it, so there is no wiring entry to name. The
  [`abstract_type_mismatch`](../../tests/ui/acceptable/types/abstract_type_mismatch.rs) fixture pins
  the shape.

A field-type mismatch is an eighth leaf, worded the same way from its `[CGP-E003]` headline. Any
other leaf simply restates its bound — `root cause: the trait bound \`f64: Eq\` is not satisfied`,
module qualifiers stripped.

**A lead is dropped when the main message already states that very leaf**, so the note does not
repeat the header — and the test is the header, not the kind of leaf. Both mismatch classes state
their leaf in full, so their notes normally carry the chain alone (as the `E0271` example above
shows), and a kept rustc header restating the ordinary bound the walk descended to drops that bound's
lead the same way. But the *same* leaf keeps its lead under a header that names something else, which
is what the emitter's coalesced block produces: its header lists the affected consumers, so the lead
is the only place above the tree where the cause appears. This matters most for an abstract type,
since one wrong binding breaks every consumer that raises through it and so almost always coalesces.

## Emitting the transformed diagnostic

The wording is decided rustc-free and only *applied* by the emitter. The emitter maps the diagnostic's
own rustc code to a rustc-free [`DiagKind`](../../crates/cargo-cgp-error-processing/src/diagnosis/plan.rs)
(`E0271` a projection mismatch, `E0599` a use-site method, everything else a plain check) and hands that,
the main-message text, the `Resolved`, and the name map to `plan_resolved`, which returns a
`DiagnosisPlan`: the rewritten header (or `None` to keep rustc's), the fix `help`s, and the
`root cause:` note as an unrendered `PendingNote`, which the emitter renders at flush — folding every
cause's paths into one [dependency graph](dependency-graph-rendering.md) against what the
compilation's earlier notes already drew. One mapping is by *anchor* rather than by code: a resolution the call-site anchor produced
plans as a use-site failure whatever its rustc code (a genuine `E0271` field mismatch excepted), so
its header names the consumer trait the call needs rather than whichever provider bound rustc's
headline stopped on — at a call that is dispatch plumbing (`PipeHandlers`, `ComposeHandlers`) the
programmer never asserted on, where the `[CGP-E002]` provider form would leak internals.

`plan_resolved`'s `categorized_header` is what picks the headline class described earlier — the
`CGP-E001` consumer form (worded from the resolution's context and consumer trait(s), pluralized when a
use-site failure spans several components, and also used for an `IsProviderFor` bound whose provider is
a `RedirectLookup`), the `CGP-E002` provider form from the text rewrite when rustc's own header names a
real wired provider's `IsProviderFor`, the two mismatch forms — `CGP-E003` for a `HasField` value type,
`CGP-E017` for any other associated type, the field form tried first as the more specific of the two —
or the `CGP-E009` plain
wrapper form. One extra case routes here: a mismatch-coded (`E0271`) failure the resolver traced
to a *non*-mismatch cause — a manual `Send`-recovery wrapper's opaque-future error, whose
`type mismatch resolving …` message is unreadable — takes the `CGP-E001` consumer form, since it is
really the consumer trait failing to be implemented. The header is `None` (rustc's kept) only when the
main message restates a genuine recovered leaf; a mid-chain symptom is replaced by the consumer header,
and an unrelated non-CGP message is kept. (This is the one place the `[CGP-E002]` provider wording still
routes through the text rewrite and its `ComponentNameMap`, because the *header* it rewrites is rustc's
own `IsProviderFor`-worded main message; the resolved *tree* beneath it is built entirely from the real
traits.)

`transform_resolved` then mutates rustc's `DiagInner`: with a header it replaces the main message and
collapses the span to the primary caret (the original labels restate the replaced message), and with
`None` it leaves the header, labels, and caret alone. Either way it replaces the children with the
plan's `help`s (one per distinct type that must derive `#[derive(HasField)]`, or the `Deref` target,
plus one per abstract-type mismatch naming the wiring entry to change; a field-type mismatch
contributes none) and the plan's single note — opening with its `root cause:` lead
(or `root causes:` list, when the causes bottom out on different leaves) over
`this is required through the dependency chain:` and the graph beneath (the lead omitted when the kept
header or the `CGP-E003`
header already states the bound). rustc's structured
suggestions are discarded with its notes, the diagnostic's Rust code is never touched, and a provider
with two absent dependencies sharing its chain yields one note over a merged tree that branches to
both. The JSON emitter regenerates every rendered and
structured field from the `DiagInner` for free, with rustc's note-continuation indentation aligning each
tree's box-drawing under its `= note:`. A final cross-diagnostic de-duplication (keyed on the recovered
cause, the rendered text, or the coded header) suppresses a transformed diagnostic that re-reports a
failure already shown.

## Tests

The wording is unit-tested rustc-free, over hand-built inputs:
[`cargo-cgp-error-processing/tests/diagnosis.rs`](../../crates/cargo-cgp-error-processing/tests/diagnosis.rs)
drives the coded headers, the `root cause:` notes, and the derive `help`s;
[`tests/coalesce.rs`](../../crates/cargo-cgp-error-processing/tests/coalesce.rs) the
underived-field coalescing and its boundaries;
[`tests/graph.rs`](../../crates/cargo-cgp-error-processing/tests/graph.rs) the graph build-and-render
(spine, branch, diamond, super-root, within-path repeat, elision) as `insta` inline snapshots; and
[`tests/tree.rs`](../../crates/cargo-cgp-error-processing/tests/tree.rs) the `cargo tree`-style
renderer. The end-to-end fixture catalog lives in the parent document's
[Tests](typed-root-cause-resolution.md#tests) section.

## Source

- [`crates/cargo-cgp-error-processing/src/diagnosis/`](../../crates/cargo-cgp-error-processing/src/diagnosis)
  — the rustc-free model (`leaf.rs`, `resolved.rs`), the structured nodes and graph (`node.rs`,
  `graph.rs`), the wording (`wording/`), the coalescing (`coalesce.rs`), and the plan (`plan.rs`).
- [`crates/cargo-cgp-error-processing/src/tree.rs`](../../crates/cargo-cgp-error-processing/src/tree.rs)
  — the `DependencyTree` and its renderer, the target the graph expands into.
- [`crates/cargo-cgp-driver/src/emitter/`](../../crates/cargo-cgp-driver/src/emitter) — the
  `try_resolve` seam and the `transform_resolved` mutation that applies the plan.

## Further reading

- [Typed root-cause resolution](typed-root-cause-resolution.md) — the pipeline overview and the
  consolidated tests and source catalogs.
- [Error processing](error-processing.md) — the rustc-free crate the wording lives in, and the
  post-processing every emitted diagnostic still passes through.
- [The driver](driver.md) — the emitter that hosts the seam, its de-duplication ledger, and the
  fallback text rewrite.
