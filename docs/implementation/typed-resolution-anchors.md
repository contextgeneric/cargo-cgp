# Typed resolution: anchoring the starting obligation

This document covers the first stage of the driver's
[typed root-cause resolution](typed-root-cause-resolution.md): recovering, from a failing
diagnostic, the **real consumer-trait obligation** `Ctx: ConsumerTrait<Params…>` that seeds the
[walk](typed-resolution-walk.md). Seven anchors recover it from seven failure shapes, tried in order;
the first that succeeds wins, and each produces the same thing — the consumer obligation — from a
different shape. Six are described here; the remaining one — the re-read of the failing call
expression, tried sixth — has [its own document](typed-resolution-call-site.md), and how the
recovered causes are worded and emitted is
[the transformed diagnostic](typed-resolution-output.md).

**From a `check_components!` entry, by span.** A `check_components!` entry expands to a concrete impl
of a generated check trait — `impl __CheckRectangle<AreaCalculatorComponent, ()> for Rectangle {}` —
whose check trait carries `CanUseComponent<Marker, Params>` as a supertrait. The macro re-spans the
context type in that impl onto the entry the user wrote, so the impl's `Self`-type span equals the
failing diagnostic's primary span. `resolve_check_failure` walks the crate's check traits (those with
a `cgp_component::CanUseComponent` supertrait) and picks the impl whose `Self` span matches the caret —
tying *this* diagnostic to *this* entry without reading either one's text. It reads the entry's
`CanUseComponent<Marker, Params>` assertion only to learn *which* component the check names: it maps
the marker to its consumer trait (`marker_to_consumer`) and ungroups the `Params` slot back into the
consumer's own arguments (`can_use_to_consumer_obligation` over `consumer_obligation`), yielding the
real obligation, e.g. `Rectangle: CanCalculateArea`. The ungrouping is decided by the consumer's own
generics, not by the slot's shape: the slot carries the parameters as all-types data (none as `()`,
one bare, several as a tuple, a lifetime lifted into `Life<'a>`), so the trait's parameter count
decides whether a tuple is *the* single tuple-typed parameter or several to spread, and a lifetime
parameter takes its region back out of the `Life<'a>` lift. Trusting the slot's shape instead would
hand the solver a malformed obligation — a `Life<'a>` *type* where a region belongs aborts the
compiler when related — so any mismatch declines to the fallback rather than build one (the
[`lifetime_component`](../../tests/ui/acceptable/generic/lifetime_component.rs) and
[`tuple_param_component`](../../tests/ui/acceptable/generic/tuple_param_component.rs) fixtures pin the
two shapes). `CanUseComponent` is the user's own check assertion, legitimately read here to find the
component; it is the marker map, not the walk, that then routes to the consumer.

**From a hand-written `impl Trait for Context` block.** A wiring failure often surfaces inside an impl
the programmer wrote rather than at a check entry — the money-transfer example's per-endpoint wrapper,
which adds a `Send` bound the component cannot express:

```rust
pub trait CanHandleApiSend<Api>: CanHandleApi<Api, Request: Send, Response: Send> + Send + Sync {
    fn handle_api_send(&self, _api: PhantomData<Api>, request: Self::Request)
        -> impl Future<Output = Result<Self::Response, Self::Error>> + Send;
}

impl CanHandleApiSend<QueryBalanceApi> for MockApp {
    async fn handle_api_send(&self, api: PhantomData<QueryBalanceApi>, request: Self::Request)
        -> Result<Self::Response, Self::Error> {
        self.handle_api(api, request).await
    }
}
```

`CanHandleApiSend` carries the CGP consumer trait `CanHandleApi<Api>` as a supertrait and is
implemented directly on `MockApp`. When the underlying `CanHandleApi` wiring is broken, the caret
lands on this impl — its header, a method signature, or the forwarding call — never on `MockApp`'s own
type definition, so the use-site anchor cannot recover the context from a struct-definition span.
`resolve_impl_site` handles it: it finds the enclosing trait impl whose *full* HIR span (not
`def_span`, which for an impl covers only the header) contains a diagnostic span, takes its `Self` type
as the context, and instantiates the impl's supertraits for that `Self`. A supertrait on that context
that does not hold and is either a CGP consumer trait **or** a `#[cgp_fn]` / `#[blanket_trait]`
blanket-impl trait (`impl<Context> Trait for Context where Self: HasField<…>`, a local trait with a
blanket impl but no provider — recognized by `is_local_blanket_trait`) **is** the obligation to walk —
the resolver seeds it directly (`wrapper_consumer_causes`), with its concrete component parameter
intact (`CanHandleApi<QueryBalanceApi>`, not the `()` a parameterless re-check would substitute), so no
marker detour is needed. The blanket-impl case is what reshapes a `#[cgp_fn]` capability check
(`pub trait CheckGetUser: GetUser {}` + `impl CheckGetUser for App {}`, the tutorial idiom for
asserting a capability holds) whose real cause is a field the context is missing: the walk descends
the blanket's `where` clause — `Self: HasField<…>`, or a `#[uses]`-chained sibling capability — to the
`` missing field `…` `` leaf, in place of rustc's misleading "`#[derive(HasField)]` is required" note.
A plain supertrait such as `Send` is neither, so it is left alone.

The tree and headline are then **headed by the impl's own trait** — the wrapper the programmer wrote —
so the failure reads `CanHandleApiSend → CanHandleApi → …` and points at their code rather than
dropping the wrapper. The headline wording turns on the wrapper's **fingerprint**: a wrapper that is
itself a CGP consumer trait (has a consumer blanket routing to a provider) reads
`[CGP-E001] the consumer trait …`, while a plain wrapper such as `CanHandleApiSend` — with only a
concrete impl — reads `[CGP-E009] the trait …`. Because the wrapper is a distinct trait from the CGP
supertrait it reduces to, its error is reported on its own rather than de-duplicating into the
`check_components!` entry for that supertrait. This anchor is tried *before* the wrapper-chain and
use-site ones, and it fires only for an impl on a *local* struct or enum — an impl on a foreign type or
a provider struct carries no consumer supertrait on a context and is skipped.

**From a foreign wrapper chain.** The routing glue can put the failure one level further out: a
hand-written `impl Trait for Foreign` block whose `Self` is a foreign type holding the context, where
the CGP consumer sits several ordinary-trait `where`-clause hops beneath the impl. The money-transfer
example's routing layer is the case — `impl CanAddApiRoutes for Router<Arc<MockApp>>`, whose supertrait
descends through `CanAddMainApiRoutes<MockApp>` and `CanAddRoute<MockApp, …>` before reaching
`MockApp: CanHandleApi<…>`, with the real context `MockApp` appearing only as a type *argument* of each
hop and never as the impl's `Self`. Neither the impl-site anchor (whose `Self` must be a local context)
nor the use-site anchor (whose context comes from a struct-definition span the caret never touches) can
recover it.

`resolve_wrapper_chain` descends the impl's own unmet supertrait through the ordinary trait obligations
beneath it — each impl's `where`-clause bounds — until one lands on a CGP consumer whose `Self` is a
local context, the handoff (`consumer_handoff_causes`) it then seeds and walks directly. Every ordinary
hop becomes a `trait impl` node, so the tree reads from the code the programmer wrote down to the root
cause. Two subtleties make the descent work. First, it **re-evaluates each obligation with the trait
solver** rather than trusting rustc's cascade-suppressed diagnostic: the direct
`MockApp: CanHandleApiSend<…>` bound is *assumed to hold* off its own ill-formed impl, so the descent
reaches the consumer instead through the **base trait of a projection `where`-clause** (a
`Ctx::Response: Send` bound over the broken `CanHandleApi`, whose base `Ctx: CanHandleApi<…>` is what
genuinely fails), which requires reading the impl's predicates *un-normalized* so the projection's base
survives. Second, the tree and headline are headed by the impl's own trait, fingerprinted for the
`[CGP-E001]`/`[CGP-E009]` wording as the impl-site anchor does — but because `Self` is a foreign
wrapper rather than the context, the headline names it **plainly**
(`the trait \`CanAddApiRoutes\` is not implemented for \`Router<Arc<MockApp>>\``, with no `context`
qualifier), carried by the `Resolved::subject_is_context` flag. Only a genuine CGP consumer is ever
reported as a cause, so a descent into unrelated `where`-clauses contributes nothing. This anchor is
tried after the impl-site anchor and before the use-site ones.

**From a use site, by wired component.** When no impl matches the caret, the failure is often a
consumer-method call — CGP wiring is lazy, so a broken dependency surfaces where the method is *called*
rather than at a check:

```rust
let person = Person { /* … */ };
person.greet(); // `Person` cannot satisfy `CanGreet`'s wiring
```

This is an `E0599` "the method `greet` exists … but its trait bounds were not satisfied", with no check
impl to anchor on. `resolve_use_site` recovers the context from the diagnostic's own spans instead: it
scans every local struct/enum whose definition span contains one of the diagnostic's spans (the
receiver's type is one such — the "method not found for this struct" span lands on `Person`'s
definition) and, for each candidate, reads the `DelegateComponent<Key>` impls that context wires, maps
each key to its consumer trait, seeds that consumer obligation, and keeps the ones that do not hold.
Because it walks every wired component, several of them can bottom out on one shared cause, so the
collected causes go through `merge_causes_by_leaf`: the shared cause is named once and keeps *every*
component's route to it, so each consumer the header lists has a chain in the note
([`use_site_shared_cause`](../../tests/ui/acceptable/duplication/use_site_shared_cause.rs)). A
diagnostic span can also land on a *provider* struct, so a candidate that wires no failing component is
discarded, which selects the real context. The transformed error is the same `[CGP-E001]` consumer form
over a root-cause note, and the misleading "this is an associated function… use associated function
syntax instead" advice — which the method probe emits for CGP's `self`-less provider methods — is
dropped with the rest of rustc's sub-notes. The anchor is not limited to method calls: any failure
whose spans land on the context's struct definition reaches it, which is how a
**`#[check_providers(...)]` per-layer assertion** — whose `IsProviderFor`-supertraited check impl no
other anchor matches — still resolves to the failing layer's root cause, because rustc's "not
implemented for `Rectangle`" note spans the struct
([`check_providers_layer`](../../tests/ui/acceptable/providers/check_providers_layer.rs)).

A `DelegateComponent` key comes in three shapes, and each is handled differently — the distinction
matters for an `open`-dispatched context, whose per-value entries are redirect *paths*, not markers. A
**bare component marker** maps to `Ctx: Consumer` (its parameterless form). An **`open`-dispatch
redirect path** — `PathCons<Component, PathCons<Value, …>>`, the key an `@Component.Value:` entry emits
— is decomposed, and the real dispatch value re-checked as `Ctx: Consumer<Value>`, so the failure is
traced with the value the context actually wired (re-checking the raw `PathCons` key would report the
internal spine as a bogus consumer bottoming out on `T: Sized` noise). Three keys are skipped: a bare
marker that is *also* `open`-dispatched (its `()` form would report a spurious `@Component.()` redirect,
while its real values are covered by the path entries); a generic catch-all whose recovered value still
carries a free type parameter (`<'a, T> &'a T: SerializeDeref` yields `&T`, whose re-check produces only
`T: Sized`); and a **blanket-forwarding key** — a bare type parameter (`__Key__`), the impl a
`namespace …;` join emits (`impl<__Key__> DelegateComponent<__Key__> for Ctx`) to forward *every* lookup
to the namespace. That key names no concrete component, and re-checking a free parameter as one bottoms
out on `__Key__: Sized` noise; skipping it means this anchor yields nothing for a pure namespace join
(whose concrete wiring lives in the namespace, not the context's own impls), leaving that case to the
next anchor. The [`open_dispatch_use_site`](../../tests/ui/acceptable/use-site/open_dispatch_use_site.rs)
fixture pins the path re-check.

**From a use site, by consumer trait.** The next anchor closes the namespace-joined gap the previous
one leaves. When a use-site failure names a **local, non-generic CGP consumer trait** in the diagnostic
— an `E0599` note such as `` `CanGreet` defines an item `greet` `` points its span at the trait
definition — `resolve_use_site_consumer` recovers that consumer trait and the context ADT from the
diagnostic's spans and seeds `Ctx: CanGreet` directly, no marker involved. The walk then descends
through the context's joined namespace to the real provider and its missing dependency: a namespace
join gives the context only a blanket `DelegateComponent<__Key__>` forwarding, so its concrete wiring
is invisible to the per-component anchor above, but the trait solver resolves the delegate *through*
the namespace's `RedirectLookup` when the walk normalizes it. This anchor is restricted to a consumer
whose only generic is `Self` — so the obligation forms without the component parameters a use site does
not carry — and it reaches not only namespace-joined method calls
([`namespace_join_use_site`](../../tests/ui/acceptable/use-site/namespace_join_use_site.rs)) but any
failure that names a local consumer and its context in its spans, including a manual supertrait bound in
a trait definition or `where` clause (the `use_type_*_unsatisfied` fixtures under
[`acceptable/use-type/`](../../tests/ui/acceptable/use-type)). A directly-wired
context keeps the more precise per-component recovery.

**From the call expression itself.** The sixth anchor, `resolve_call_site`, handles the use-site
failure whose spans touch *nothing* the other anchors can read. Two shapes reach it. In the first,
a context's wiring matches the called component unconditionally, so the method is *found*, the
failure is an `E0277` rather than an `E0599`, and its spans never leave the call. In the second, the
called method belongs to a `#[cgp_fn]`/`#[blanket_trait]` **capability trait** rather than a CGP
consumer — a local blanket-impl trait that is not a component, so the by-consumer anchor (restricted
to CGP consumers) declines its `E0599`; the anchor finds it by method name and heads the result
`[CGP-E009] the trait …`. Either way it re-reads the failing call expression from
HIR — the context from the method's *receiver*, the component's parameters by *unifying the call's
written argument types against the method's own declared signature*, and every parameter the call
leaves to inference seeded as a rigid placeholder the walk resolves around. It is the one anchor
whose recovery works from the code the programmer wrote rather than from the diagnostic's spans, and
its rationale, mechanics, and worked example have their own document:
[Typed resolution: the call-site anchor](typed-resolution-call-site.md).

**Recognizing a capability trait.** Three of the anchors below reach a
`#[cgp_fn]`/`#[blanket_trait]` **capability** — a trait consumed like a consumer but which is not a
component, so no marker or provider trait identifies it. The only structural mark it carries is a
blanket impl over a bare context, and that alone is far too broad to key on: `ToString`, `Into`, and
`Borrow` all have one, and reshaping their failures into CGP errors would be an over-reach. So
`is_capability_trait` accepts a trait two ways. One the **checked crate defines** qualifies outright,
since cargo-cgp runs on CGP workspaces and a failing local blanket trait is the shape `#[cgp_fn]`
produces. A **foreign** one must show that its blanket genuinely depends on CGP — on a trait from
cgp's own crates (`HasField` above all), on a CGP consumer trait, or on another capability that does,
followed a few links through composed capabilities. That is what lets the reshaping reach a
capability a *library* publishes, which is where capabilities normally live
([`upstream_capability_use_site`](../../tests/ui/acceptable/use-site/upstream_capability_use_site.rs)),
while still excluding the std blankets the rule is aimed at.

**From a use site, by capability trait.** The seventh and last anchor, `resolve_use_site_capability`,
is the by-consumer anchor's counterpart for a `#[cgp_fn]`/`#[blanket_trait]` **capability trait** — a
blanket-impl trait that is not a CGP component. It reaches the shape a capability required
through a `where` **bound** or supertrait produces (`fn greet_all<Context: GetName>(…)` called with a
context missing the field): an `E0277` naming the capability, with no method call on a concrete
context for the call-site anchor to read. It recovers the capability trait from the diagnostic's
spans (as the by-consumer anchor recovers a consumer), and the context from the **failing expression
itself** — the call argument whose type fails (`app`, read off its binding by the call-site anchor's
`contexts_at_spans`) — because rustc puts its "not implemented for `App`" span on the context's
`#[derive(HasField)]` attribute, *outside* the struct's item span, so no struct-definition span
carries it. The walk then descends `Ctx: Capability` to the cause, and the result is headed
`[CGP-E009] the trait …` (not `[CGP-E001] the consumer trait …`) since a capability trait is not a
component. It is gated to the `E0277` shape and tried **after** the call-site anchor deliberately: an
`E0599` method call belongs to the call-site anchor, and a *generic-consumer* method call whose deep
capability bound is unrecoverable (`generic_consumer_unwritten_arg`) must stay declined rather than
latch onto that transitive capability.

## Tests

Every anchor is pinned end to end by the UI snapshot suite; the consolidated fixture catalog, with
one entry per pinned behavior, lives in the parent document's
[Tests](typed-root-cause-resolution.md#tests) section. The check-entry seed's params-slot
ungrouping, the impl-site and wrapper-chain paths, and the span-matching use-site paths are the
groups anchored here.

## Source

- [`crates/cargo-cgp-driver/src/resolve/anchor/`](../../crates/cargo-cgp-driver/src/resolve/anchor)
  — one file per anchor (`check_failure.rs`, `impl_site.rs`, `wrapper_chain.rs`, `use_site.rs`, and
  `use_site_consumer.rs`, which holds both the by-consumer `resolve_use_site_consumer` and its
  by-capability sibling `resolve_use_site_capability` over one shared helper) over the shared
  `seed.rs` (the consumer-obligation builder) and `spans.rs` (the local items a diagnostic's spans
  land on). The by-capability anchor recovers its context from the failing expression through the
  call-site anchor's `contexts_at_spans`.
- [`crates/cargo-cgp-driver/src/resolve/cgp_item.rs`](../../crates/cargo-cgp-driver/src/resolve/cgp_item.rs)
  — the DefId-anchored, `IsProviderFor`-free trait recognition every anchor relies on.

The per-file map of the whole resolver lives in the parent document's
[Source](typed-root-cause-resolution.md#source) section.

## Further reading

- [Typed root-cause resolution](typed-root-cause-resolution.md) — the pipeline overview, its
  boundaries, and the consolidated tests and source catalogs.
- [Typed resolution: the call-site anchor](typed-resolution-call-site.md) — the sixth anchor.
- [Typed resolution: walking to the root cause](typed-resolution-walk.md) — the descent every
  anchor's seed feeds.
