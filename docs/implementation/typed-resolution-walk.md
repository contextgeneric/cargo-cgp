# Typed resolution: walking to the root cause

This document covers the descent at the heart of the driver's
[typed root-cause resolution](typed-root-cause-resolution.md): from the consumer obligation an
[anchor](typed-resolution-anchors.md) seeds, down the wiring's trait obligations to every terminal
unmet bound, and the decoding, classification, and rendering that turns each terminal into a leaf
of the dependency tree [the emitter emits](typed-resolution-output.md).

## Walking the dependency graph downward

From the anchored consumer obligation the resolver walks *down* the wiring's trait obligations, because
the tree shows the transitive path to each root cause, not only the root. For a failing obligation it
finds the impl that would satisfy it, takes that impl's `where`-clause obligations as its direct
dependencies, and recurses into just the ones that do **not** already hold — a satisfied dependency (an
already present field, a wired provider that checks out) is pruned. Following *every* unmet dependency,
not only the first, is what surfaces independent causes as **separate** paths: the next-generation
solver short-circuits a conjunction at its first unmet bound, so a provider needing two absent fields
would otherwise hide one.

The descended chain is the real one: `Ctx: Consumer` → `Ctx: Provider<Ctx>` (the delegation routing,
whose label the tree drops) → the delegate's `Provider: ProviderTrait<Ctx>` (the real wired provider)
→ that provider impl's own `where` bounds → the leaf. The provider's bounds are read from its *own*
concrete impl, which is why `IsProviderFor` is never needed: it carried a copy of exactly these bounds
only for rustc's benefit. An `IsProviderFor` or `CanUseComponent` obligation that appears *beside* the
real one in a generated blanket (the provider blanket bounds `P: IsProviderFor<…>` right next to
`P::Delegate: ProviderTrait<…>`) is dropped from a node's dependencies by `is_workaround_plumbing`
before recursion, so the walk follows the real provider obligation rather than the marker's copied
bounds — and would keep working unchanged if `IsProviderFor` were removed from the generated code
entirely.

Finding *which* impl satisfies an obligation is two subtleties deep past a flat wiring. First, a
provider obligation such as `DeserializeRecordFields: ValueDeserializer<…>` unifies with *two* impls:
the provider's own `#[cgp_provider]` impl and the CGP delegation blanket
`impl<P: DelegateComponent> ValueDeserializer<…> for P`. Only the concrete-`Self` impl's `where`-clauses
lead to the real cause — the blanket's lead to a `DeserializeRecordFields: DelegateComponent`
dead-end, since a leaf provider does not delegate — so a **concrete-`Self` impl is preferred** over a
param-`Self` blanket, the blanket used only when it is the sole match (as for an obligation whose `Self`
*is* the context). Second, an impl may carry a **parameter fixed only by an associated-type
`where`-clause** — a record deserializer's `Builder`, pinned by
`Record: HasOptionalBuilder<Builder = Builder>` — that stays a free inference variable after the trait
ref unifies. The walk registers and **solves the impl's satisfiable clauses first**, binding such a
parameter, so the sibling clause that carries it (the branch to the cause) is not dropped as
inference-laden.

A clause the impl match leaves genuinely unconstrained is the subtle case, and the one that matters
most is a **later stage of a pipeline keyed on an earlier stage's output type**. Two pieces of
background make it concrete.

An *associated-type projection* is a type written `<T as SomeTrait<…>>::Assoc`: not named directly,
but computed from a trait impl. The solver *normalizes* it to a real type by selecting the impl of
`SomeTrait` for `T` and reading off the value that impl declares for `Assoc` — which requires the
impl to actually apply, i.e. its `where` clauses to hold. CGP's [handler family](https://github.com/contextgeneric/cgp/blob/main/docs/concepts/handlers.md)
composes computations into pipelines, and a composition provider wires each stage's *input* to the
previous stage's *output type*, which is exactly such a projection. Its archetype
`ComposeHandlers<ProviderA, ProviderB>` (introduced under
[the failure shape](typed-resolution-call-site.md#the-failure-shape-wiring-that-matches-unconditionally)) has two asymmetric
dependencies: `ProviderA: Handler<Ctx, Code, Input>` on the pipeline's own input, and
`ProviderB: Handler<Ctx, Code, ProviderA::Output>` on whatever the first stage *produces*. Here
`ProviderA::Output` is the projection — the handler trait's associated type happens to be named
`Output`, but nothing below turns on that name.

The problem appears when the pipeline's input is unknown. When the walk's own input is a
[call-site placeholder](typed-resolution-call-site.md) — a rigid stand-in for a call
argument the code did not type — `ProviderA`'s own `where` clause is *false* against it (a rigid
placeholder satisfies no `Input: Send`-style bound), so the solver rejects `ProviderA`'s impl and
`ProviderA::Output` cannot normalize. The next-generation solver leaves it a fresh inference variable,
so the `ProviderB` clause reads as `Handler<Ctx, Code, _>`. Dropping it as inference-laden would
silently discard **every root cause living in a stage past the first** — a bound a later stage places
on the value the earlier stage feeds it, which is where a great many real pipeline mistakes sit.

So the walk first tries to **recover** the earlier stage's output rather than give up on it, because
that output type is very often *fixed* — declared by the provider independently of its input — and
stalled only because the placeholder falsified an unrelated `where` clause. `resolve_fixed_projections`
re-normalizes each stalled projection in a fresh `InferCtxt` with the placeholders turned back into
ordinary inference variables (`placeholders_to_infer`, the inverse of `unknowns_to_placeholders`). An
inference variable makes the blocking bound *deferrable* rather than false, so the solver commits to
the sole impl candidate and reads off the associated type it declares. The recovered type is kept only
when it comes out **fully concrete** — no inference variable or placeholder left, and no longer an
alias — so an output that genuinely depends on the input never concretizes and falls through to the
fold below; and since the seed obligation is still gated on actually failing, a recovered concrete
input can never fabricate a cause. With the earlier stage's output recovered, the later stage's input
becomes that concrete type and the bound it fails becomes the reported root cause — for example an
`AsRef<[u8]>` requirement a byte-consuming stage places on a producing stage whose output type is not
a byte slice, the shape [`cascade_later_stage_input`](../../tests/ui/acceptable/use-site/cascade_later_stage_input.rs)
pins.

The step is **structural**: it recovers `<T as SomeTrait<…>>::Assoc` for *any* trait and *any*
associated type, at any argument position and to any depth — a three-stage pipeline nests one stage's
output projection inside the next — matching on the projection kind alone and never on an
associated-type name, trait, or `DefId`. `ComposeHandlers`'s `Output` is only the shape that
motivated it.

When a stalled projection *cannot* be recovered this way — its value genuinely depends on the unknown
input — the walk falls back to **folding each stray inference variable into a rigid placeholder** (the
same `unknowns_to_placeholders` the call-site anchor seeds unknown call arguments with) and descends
the stage anyway. The placeholder is an unknown the walk resolves *around*: it reaches `ProviderB`'s
context-side dependencies — a `Self: HasField` a later stage reads — while the
[placeholder-leaf filter](typed-resolution-output.md#the-root-cause-notes) keeps any leaf that genuinely depends on the unknown
input (`_: Send`) from being reported. Both the recovery and the fold are a no-op when a clause
carries no placeholder — the ordinary concrete-input walk, where each stage's projection normalizes to
a real type — so only the unknown-input case is affected.

One descent step is not about the root context at all. A **cross-context** consumer obligation — one
whose `Self` is a *different* local context (`Inner: CanCompute` reached while resolving `Outer`, the
shape where one context's wiring depends on a concrete other context) — is re-rooted at that context
before it is walked: the node and everything below it are labeled and classified against `Inner`, not
`Outer`. So it reads as a consumer node `for context Inner`, its delegation-routing hop is recognized
as routing and dropped, and its getter leaf decodes to a missing field rather than an opaque foreign
bound. Re-rooting also keys the node identically to `Inner`'s own walk, so the two share the
[cache](cached-dependency-resolution.md). Without it, the descent would carry the outer root context
down and mislabel the whole foreign-context subtree.

A branch ends at a **terminal leaf**, and which obligations count as terminal is what keeps the tree
honest. The descent follows only the CGP wiring vocabulary — any provider trait (a
`ProvideFoo: Foo<App>` bound routes on to the provider's own dependencies), `DelegateComponent`, and
any obligation whose `Self` is the context (its consumer, getter, and capability traits) — and treats
everything else as a leaf. It deliberately does *not* follow `IsProviderFor`/`CanUseComponent`, which
are dropped as plumbing above. The leaf shapes are:

- An unmet **`HasField`** is the field leaf.
- An unmet **`DelegateComponent<Marker>` on the context** is the missing-wiring leaf: the context
  delegates that component to no provider. (When the key is a `PathCons` path rather than a bare
  marker — an `open`-dispatched value the context never wired — it is the missing-*redirect*-wiring
  leaf, named by its whole path.)
- An unmet **`DelegateComponent<Key>` on a *non-context* delegation table** is the missing-dispatch-entry
  leaf (`[CGP-E110]`): the owner is a provider that delegates — an aggregate provider missing a
  component wiring, or a `UseDelegate`/`UseInputDelegate` table missing a branch for the type it
  dispatches on (a `Code` fragment or an `Input` value's type) — and it does not wire this key. It is
  told apart from a higher-order-provider dead-end (dropped, below) two ways, so it is reported when
  *either* holds (`is_reportable_leaf` / `is_dispatch_lookup` / `is_delegation_table` in `classify.rs`):
  - **structurally**, when the obligation is a *dispatch lookup into a separate table* — its owner is a
    proper part of the parent obligation's `Self`, as `Components` is of `UseDelegate<Components>` /
    `UseInputDelegate<Components>` (or any custom dispatcher that holds its table as a parameter). Such
    a `where`-clause is unambiguously a table lookup, so an unmet one is a missing entry *regardless of
    whether that table wires any other key* — this is what reaches an **empty** dispatch table.
  - **by owner property** otherwise (the generic-blanket case, where the owner *equals* the parent
    `Self`): the owner wires at least one other key, i.e. carries a `DelegateComponent` impl. This is
    the aggregate-provider case, where the blanket keys on the provider itself so no separate-table
    structure is visible.

  This is the leaf a handler pipeline bottoms out on when a stage's output type is not one a later
  stage's input dispatcher handles — the shape the `http_checksum_native` hypershell example produces
  once a byte-encoding stage is removed and a raw `GenericArray` digest reaches an `AsyncRead` sink's
  input dispatcher (distilled in
  [`cascade_nested_projection`](../../tests/ui/acceptable/use-site/cascade_nested_projection.rs), with
  the empty-table variant in
  [`empty_dispatch_table`](../../tests/ui/acceptable/use-site/empty_dispatch_table.rs)).
- An unmet **`DelegateComponent<Marker>` on a type that is neither the context nor any table** is the
  **not-a-provider** leaf (`[CGP-E111]`): a type wired where a provider was expected that does not
  implement the provider trait at all (`UseBasicAuth<QueryBalanceRequest>`, a request type in a
  handler slot). It arises via the generic blanket for the parent provider trait `T` (owner == parent
  `Self`), and is split from the **dead-end** by whether the owner has a *concrete* impl of `T`
  (`owner_has_impl_of`): a leaf provider reached via the blanket only because its concrete impl did
  not unify (a pipeline stage fed the wrong input — the `HandleShout` shape) *has* one and is dropped,
  its real cause running through that impl; a genuine non-provider has *none* and is reported, named
  against `T` rather than a wiring key ([`non_provider_wired`](../../tests/ui/acceptable/providers/non_provider_wired.rs)
  pins it; [`cascade_after_use_site`](../../tests/ui/acceptable/use-site/cascade_after_use_site.rs) is
  the dead-end that must stay dropped).
- An unmet **namespace-lookup bound** is a missing-redirect-wiring leaf too. It is recognized not by
  name but by the trait's *fingerprint* — a single `Delegate` associated type, which `DefaultNamespace`,
  the `DefaultImpls*` traits, and every user `cgp_namespace!` trait share — so a same-named user
  namespace is caught without a `DefId` anchor.
- An **ordinary bound on a foreign type** (`f64: Eq`) is a leaf, and the descent must not walk into
  whatever unrelated `std` blanket impl happens to match its `Self` (an `impl<F: FnPtr> Eq for F` would
  otherwise fabricate a misleading `f64: FnPtr` step). A **constrained `DelegateComponent` key** whose
  own `where`-clause is unmet surfaces here: when a dispatcher provider's wiring only delegates under a
  bound — `PipeHandlers<Providers>`'s `delegate_components!` generic list is
  `Providers: ComposeProviders<Provider = Provider>` — an argument that fails that bound (an empty
  `PipeHandlers<Product![]>`, whose `Nil` list has no `ComposeProviders` impl) makes the walk descend
  the delegation impl and bottom out on the unmet composition bound (`Nil: ComposeProviders`), which is
  the real cause, with the `DelegateComponent` node itself dropped as a plumbing label.

Two foreign bounds are exceptions the descent *does* follow, both for the same reason: they reach the
context only deeper. The first is a **getter or capability trait on a non-context type whose satisfying
impl depends on the context** — a request struct's `HasBasicAuthHeader<Ctx>`, whose `#[cgp_auto_getter]`
blanket impl requires `Ctx: HasPasswordType`. There the walk looks into that blanket impl and follows
only its **context-side dependencies**, so the real cause on the context surfaces (and de-duplicates
with the same cause reached down another branch) instead of the opaque `Request: HasBasicAuthHeader<Ctx>`
bound being reported as a second, misleading root cause. Following only context-side dependencies is
what preserves the `f64: Eq` guarantee — a foreign `f64: FnPtr` step is not context-side, so it is never
followed — and it also skips the getter's own `Ctx::Assoc`-typed `HasField` clause on the request,
which is present but a projection mismatch a plain descent would misreport as a missing field. The
second is a **same-trait recursion over a type-level list**: a record's field list
`Cons<Field<.., V0>, Cons<Field<.., V1>, Nil>>: HandleMapEntry<.., Ctx, ..>` handles its head field here
but its later fields through the **tail** `Cons<.., Nil>: HandleMapEntry<..>`, a same-trait bound on
another foreign list node. Following a same-trait recursion lets the walk reach the field whose
dependency is the real cause; following only the *same* trait keeps the `f64: Eq` guarantee, since a
foreign leaf's `impl` dependency is a *different* trait.

Two further rules finish the terminal cases. An obligation whose satisfying impl's trait-clause
`where`-obligations **all hold**, yet is itself unmet, is failing for an associated-type mismatch the
trait-clause walk cannot see: `Self: Trait<Assoc = T>` desugars into *two* predicates, and the walk
follows only the trait half, so a satisfied `Ctx: HasErrorType` beside a failing
`<Ctx as HasErrorType>::Error == AppError` looks, to the trait-clause walk, like an impl with nothing
wrong. The resolver therefore looks among that impl's own predicates for an unmet **projection**
(`projection_mismatch`) and completes the branch with that projection's own trait ref, tagging the
path with the required type. Two leaf shapes come out of it, differing only in which trait the
projection is on: a `HasField` value (`<Ctx as HasField<Symbol!("f")>>::Value == T`, a field present
with the wrong type) renders as the field-type mismatch, and any other associated type renders as the
[abstract-type mismatch](typed-resolution-output.md#the-root-cause-notes), whose *actual* side is read
by normalizing the projection rather than by reading a struct field. A `HasField` projection wins when
an impl carries both, since a field's value type is the more specific classification and the one whose
leaf names a struct field to fix.

The projection search *also* prefers the concrete-`Self` impl over the delegation blanket
(`projection_mismatch` / `impl_projection_mismatch`): the projection lives on the provider's own impl,
and matching the blanket first — which carries no projection — would wrongly report no mismatch; but a
getter trait whose *only* impl is a blanket (`impl<C: HasField<..>> HasName for C`) still has its
projection read from that blanket, so a blanket is deferred, not skipped outright. A branch with no
such projection yields nothing and declines to the fallback.

Finally, a branch that bottoms out on a `DelegateComponent` is kept only when the owner
genuinely lacks an entry it *should* carry, and dropped as **pure wiring plumbing** otherwise (a
delegation that *holds* is pruned before it can be a leaf, so bottoming out unmet always means the key
is genuinely unwired — the question is only whether that owner is meant to be a table). It is kept in
three shapes: a `DelegateComponent` **on the context** (the missing-wiring leaf); a **dispatch lookup
into a separate table**, recognized structurally because the owner is a proper part of the parent
obligation's `Self` (`Components` inside `UseDelegate<Components>` / `UseInputDelegate<Components>`, or
any custom dispatcher holding its table as a parameter), which reaches even an *empty* table; and a
**non-context table reached via the generic blanket** (owner *equals* the parent `Self`), recognized
because the owner carries at least one `DelegateComponent` impl — the aggregate-provider case. The two
non-context shapes are both the missing-dispatch-entry leaf. A `DelegateComponent` that is *none* of
these — owner is not the context, not a dispatch lookup, and not a table — splits one final way, by
whether the owner has a **concrete impl of the parent provider trait** (`owner_has_impl_of` against
that trait, not `DelegateComponent`): with no such impl it is a genuine non-provider wired where a
provider was expected (the **not-a-provider** leaf, `[CGP-E111]`); with one, it is a leaf provider
reached via the blanket only because that impl failed to unify (a pipeline stage fed the wrong input —
the `HandleShout` shape), the routing dead-end that is dropped, its real cause running through that
impl down another branch.

The walk crosses inference-context boundaries carefully, because a stray variable from one `InferCtxt`
panics another. It finds the satisfying impl with the `fresh_args_for_item`-plus-unification dance
rather than `SelectionContext` (which asserts against the next-generation solver the driver runs
under), and each matched impl's predicates are instantiated, normalized, and region-erased before they
cross into the fresh inference context that checks whether they hold — the cross-context contamination
hazard in
[rustc diagnostic internals](rustc-diagnostic-internals.md#contaminating-one-inference-context-with-anothers-variables).
The obligation being matched crosses the same way: its binder is instantiated with **placeholders**
(via `enter_forall_and_leak_universe`) before it is related, never reached through a bare
`skip_binder()`. A **higher-ranked** obligation makes this load-bearing — the modular-serialization
example's `SerializeIterator` carries `Self: for<'a> CanSerializeValue<<&'a Value as IntoIterator>::Item>`,
and reaching its trait ref with `skip_binder()` would relate a term with an escaping bound variable and
hit the
[generalizer panic](rustc-diagnostic-internals.md#feeding-escaping-bound-variables-to-inference-has_escaping_bound_vars).
Placeholders rather than fresh inference variables are also what let a *nested* higher-ranked hop
resolve: a projection through the bound lifetime (`<&'a Value as IntoIterator>::Item`) normalizes
deterministically against a rigid placeholder region but stalls against an unconstrained inference
region. The instantiation is a no-op for an ordinary binder-free obligation, so only the higher-ranked
case changes.

Two safeguards keep the descent bounded. A **cycle guard** stops a branch as soon as an obligation
reappears among its own ancestors — a `UseContext` loop routes `Ctx: Consumer` straight back to itself
— so a cyclic wiring bottoms out at its first repeat rather than spinning. The depth cap `MAX_DEPTH` is
the backstop for what the guard cannot catch: a *divergent* wiring whose obligations keep growing
without ever exactly repeating. The cap is set well above a genuine chain's depth — each logical hop
expands to a consumer obligation, the delegation-routing provider obligation, a `RedirectLookup`, the
real provider obligation, then the next consumer, so a deeply nested data type reaches its root cause
only tens of frames down — but not arbitrarily high, because the walk re-runs the solver at every
frame, so the cap also bounds a divergent wiring's worst-case work.

The descent is memoized at every node, so a distinct obligation reached from many sites or many
branches is resolved once and reused; the cycle guard and `MAX_DEPTH` cut above are what decide whether
a subtree is complete enough to cache. That mechanism — the node key, the incomplete-subtree flag, and
the reachable-set reuse check — is [Cached dependency resolution](cached-dependency-resolution.md).

## Decoding, classifying, and rendering a leaf

**Decode the field name.** A `HasField` leaf carries the field name as a type-level `Symbol!`, a nested
`Chars<'h', Chars<'e', …>>` spine. The resolver decodes it structurally — walking the spine and reading
each `char` const argument until `Nil` — rather than un-sugaring the printed type. Reading the name from
the type is why the replacement never needs the `--verbose` un-eliding the
[text path depends on](driver.md#un-eliding-the-diagnostic): the characters are in the `Symbol`
arguments whether or not the diagnostic would have printed them.

**Classify why the field is unmet.** A "missing" `HasField` bound does not always mean an absent field,
so the resolver inspects the struct the bound lands on and its `Deref` chain to tell three cases apart.
The field is **missing** when neither the struct nor any `Deref` target carries it; **present** (the
underived case) when the struct itself carries the field, so only the `#[derive(HasField)]` is missing;
and **present-via-`Deref`** when the field lives on a `Deref` target that has not derived it (CGP's
`HasField` forwards across `Deref` by a blanket impl, so the bound *would* hold if that target derived
the field), in which case the resolver records the target so the fix can point at it. The inspection
reads named struct fields directly and follows `Deref` by reading each `impl Deref`'s `Target`
associated type, so it needs no inference context, and it is bounded against a cyclic `Deref`. A field
present with a mismatched **type** is a fourth case reached differently — through the projection rule in
the walk above; from the failing projection the resolver reads the expected type, and it queries the
struct by `DefId` (with the struct's own generic arguments substituted, so a same-named struct in
another module is never queried) for the actual type, yielding the `[CGP-E003]` mismatch leaf. A
non-field leaf carries no struct, so it is simply restated as `self: Trait` (`f64: std::cmp::Eq`) for
its note lead and for de-duplicating a leaf reached by several paths.

**Read the type an owner supplies for any other associated type.** A projection mismatch on a trait
other than `HasField` has no struct field to query, so its *actual* side is read by asking the solver
to normalize the projection itself (`projected_type`). That one query covers every way an owner can
supply the type — an abstract type wired to `UseType<T>` resolves through the blanket `#[cgp_type]`
generates, a hand-written `impl HasErrorType for App` through that impl — so no wiring shape is
special-cased, and a result that does not reduce to a concrete type is rendered `_` rather than
half-resolved. The leaf also records whether the trait is a **CGP abstract-type component**, which is
what earns it the `abstract type` wording and a `help` naming the wiring entry to change. That is
recognized structurally by the three things
[`#[cgp_type]`](https://github.com/contextgeneric/cgp/blob/main/docs/reference/macros/cgp_type.md)
generates and nothing else does — a consumer trait, whose only associated item is a type, whose
provider trait carries the `UseType` blanket — with the last condition anchored by `DefId` to the crate
defining `UseType`, since it is what makes `UseType<T>` a valid fix to suggest
(`abstract_type_component_marker`). A behavioral component, or a plain trait with an associated type,
matches none of the three and reads as a plain `associated type` with no `help`.

**Render each root cause.** A root-cause path is a list of typed predicates, and rendering it is where
each real trait obligation becomes the concept it stands for — with every name read straight off the
obligation, no component marker and no [`ComponentNameMap`](error-processing.md) in the loop. A
consumer-trait obligation `Ctx: Consumer<Params…>` (its `Self` the context) becomes the consumer-trait
impl (`consumer trait impl \`CanCalculateArea\` for context \`Rectangle\``), named by the consumer
trait's own `DefId`; a provider-trait obligation `Provider: ProviderTrait<Ctx, Params…>` becomes the
provider-trait impl, naming the provider trait's `DefId`, the context (the trait's leading argument),
and the provider struct (the obligation's `Self`); a `HasField` becomes the field-trait impl; and a
user's own capability or getter trait — or a terminal ordinary bound — renders as
`trait impl \`Trait\` for \`Self\``. Several details make the labels read well:

- **Plumbing is dropped.** A provider-trait obligation *for the context itself* (the delegation
  routing, as opposed to the real provider), the `DelegateComponent` table lookup, a namespace lookup,
  and any residual `CanUseComponent`/`IsProviderFor` obligation carry no information and return no
  label, keeping the chain legible without losing a real step. A `RedirectLookup<Ctx, Path>` provider,
  by contrast, renders as `redirect lookup to \`@Path\` in \`Ctx\``, so a chain of redirects reads as
  its successive hops.
- **Generic parameters are reattached.** A generic component's parameters are read from the trait
  obligation's own type arguments — the consumer's arguments after `Self`, the provider's after the
  leading context — so the trait reads as written (`CanCalculateArea<u32, u64, bool>`). The context
  and the parameters are indexed by *type* position, because a component's lifetime parameters sort
  ahead of the context in a provider trait's argument list (`ReferenceGetter<'a, Ctx, T>`): indexing
  by raw argument position would land on the region and abort the compiler, while the type-position
  read skips lifetimes in the label the way ordinary Rust elision does (`ReferenceGetter<str>`).
- **Type-level spines are resugared.** A rendered label's `Self` type has its `Cons`/`Either` spine read
  back as the `Product![A, B]` or `Sum![A, B]` the programmer wrote — and one step further to
  `Struct! { name: Type, … }` or `Enum! { Name(Type), … }` when every element is a named field — so a
  field- or variant-list handler (the modular-serialization example's `FieldsSerializer` over a record's
  field list) names the list rather than a chain of cells. The rendering is `render_ty`, the **typed**
  one of the tool's three resugaring implementations, anchored by `DefId` to the CGP crate that defines
  each cell so a same-named type elsewhere is never resugared. What each construct folds back to, what
  makes a fold decline, and how this implementation differs from the text and syntax-tree ones is
  [Resugaring](resugaring.md).

Each hop is a structured [`DepNode`](error-processing.md) variant — one per template, each stamped
with its own [`CGP-E1xx` code](../error-code.md) when rendered — so `consumer trait impl`
(`CGP-E101`), `provider trait impl` (`CGP-E102`), `redirect lookup` (`CGP-E104`), and the general
`trait impl` (`CGP-E105`) each carry a distinct tag, and a terminal leaf takes a leaf code
(`CGP-E106`–`CGP-E109`), except a pass-through ordinary bound, which stays uncoded. The walk emits
one flat path of these nodes per way a cause is reached, and the rustc-free
[dependency graph](dependency-graph-rendering.md) merges the paths and renders them as
`cargo tree`-style text, every construct named in full — a dispatch pipeline's plumbing hops all restate the
same program-sized `Code` type, and each spells it out, since the type a reader is tracing is the
point of the chain
([`deep_dispatch_chain`](../../tests/ui/acceptable/verbosity/deep_dispatch_chain.rs) pins the
resulting shape). All of the merge and render is in the rustc-free `cargo-cgp-error-processing`
crate, unit-tested on any toolchain.

## Tests

The walk and its leaf classification carry most of the resolver's fixtures — the leaf classes
(fields, wiring, redirects, dispatch tables, non-providers), the harder mechanics (parallel
branches, deep nesting, higher-ranked descents, the mid-`predicates_of` decline), and the
stalled-projection recoveries. The consolidated catalog, one entry per pinned behavior, lives in
the parent document's [Tests](typed-root-cause-resolution.md#tests) section.

## Source

- [`crates/cargo-cgp-driver/src/resolve/walk/`](../../crates/cargo-cgp-driver/src/resolve/walk) —
  the descent: `leaves.rs` (the recursion), `vocabulary.rs` (what it walks into),
  `impl_match.rs` (the satisfying impl and its dependencies), `unknowns.rs` (placeholders and
  stalled projections), `projection_mismatch.rs` (the unmet projection an impl carries, `HasField`
  or otherwise), and `holds.rs` (the solver query).
- [`crates/cargo-cgp-driver/src/resolve/classify/`](../../crates/cargo-cgp-driver/src/resolve/classify)
  — `reportable.rs` (root cause versus routing dead-end), `leaf.rs` (the rustc-free `Leaf`),
  `field.rs` (the struct and `Deref`-chain inspection), and `assoc_type.rs` (the normalization that
  reads the type an owner supplies for any other projected associated type).
- [`crates/cargo-cgp-driver/src/resolve/label/`](../../crates/cargo-cgp-driver/src/resolve/label) —
  `predicate_label.rs` (each hop as a structured `DepNode`, read off the obligation) and
  `render_ty.rs` (the type-level spine resugaring).

The per-file map of the whole resolver lives in the parent document's
[Source](typed-root-cause-resolution.md#source) section.

## Further reading

- [Typed root-cause resolution](typed-root-cause-resolution.md) — the pipeline overview, its
  boundaries, and the consolidated tests and source catalogs.
- [rustc diagnostic internals](rustc-diagnostic-internals.md) — the panic hazards the walk's
  inference-context discipline exists to avoid.
