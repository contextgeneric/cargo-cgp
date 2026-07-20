# The resolve context

The typed resolver threads a bare `TyCtxt` and a scatter of config constants through every stage, and
this document specifies replacing that with one **resolve context** — a struct that hosts the
dependency caches, the config, and the compiler-query access behind a pure interface — so the
resolution core can eventually run, and be tested, without a real compiler.

**Status: blueprint ahead of implementation.** Nothing here is built yet. The document records the
motivation, the query taxonomy, and the design decisions agreed for the work, so a later agent can
carry it out. It is the companion to [Cached dependency resolution](cached-dependency-resolution.md),
which specifies the caches this context houses, and it builds on
[Typed root-cause resolution](typed-root-cause-resolution.md) and
[its walk stage](typed-resolution-walk.md), whose queries are the surface being abstracted.

## Why a resolve context

The immediate reason is cohesion: the caches need a home, and so do the config constants and the
compiler access they all share. Passing `tcx` plus loose parameters into every `resolve/` function
works, but it gives the new caches nowhere natural to live and leaves the resolver's dependencies
implicit. A single `ResolveCtx` that carries the compiler-query access, the
[dependency caches](cached-dependency-resolution.md), and the config names (`CGP_COMPONENT_CRATE`,
`DELEGATE_COMPONENT_TRAIT`, the rest) makes those dependencies explicit and threads one value where a
handful travel today.

The larger reason is the payoff that cohesion unlocks: **making the resolution core rustc-free and
mockable.** The `cargo-cgp-error-processing` crate already proves the resolver's *output* half — the
`Resolved` model, the wording, the tree, the dedup — can live without a compiler and be unit-tested on
any toolchain. The *input* half — the anchoring and the walk that fill in `Resolved` — is still bound
to `rustc_private`, so its dense, subtle decision logic (which leaf is reportable, which owner is a
dispatch table, how a `Deref` chain classifies a field) is pinned only by whole-program UI fixtures
that must run a real compiler. If the resolver's compiler interactions sit behind an interface hosted
by the context, that decision logic can be exercised against a hand-built stand-in — fast, hermetic,
and able to reach corners that are awkward to reproduce as real CGP programs. The mechanism for that
is **deferred to later work and is deliberately not designed here**: the resolver's compiler
interactions will become **CGP components**, which this rustc-backed `ResolveCtx` implements, and a
**separate context type** implements the same components as a rustc-free stand-in — the driver
dogfooding CGP to abstract its own use of the compiler. This document specifies only the rustc-backed
context and the properties that make that later abstraction possible; the components themselves, and
the stand-in context, are out of scope.

## The organizing equivalence: cacheable is stateless is mockable

The single idea that makes this tractable is that **cacheability, statelessness, and mockability are
one property seen three ways.** A query you can cache is a pure function of its explicit typed inputs;
a pure function has no hidden state; and a pure function is exactly what a rustc-free stand-in
implements. The cache key *is* the stand-in's input and the cache value *is* its output. So the
[caching work](cached-dependency-resolution.md) is not a prerequisite to be gotten out of the way — it
is the discovery procedure for where the later component boundary falls. Proving each query cacheable
proves it stand-in-able and names its complete input; a query that resists caching without an extra
parameter has hidden state, and that parameter is the thing the later boundary must make explicit or
keep on the rustc side. Building the caches now is therefore how the component boundary is found, even
though the components themselves are designed later.

## Emit-time frozen state is what licenses purity

Every "this query is pure" claim rests on one invariant, and the design should state it plainly: the
resolver runs at emit time, over compiler state that is frozen. The trait set, the impls, the
`predicates_of`, the ADT field lists, the `Deref` targets — everything the resolver reads is fixed once
the crate is lowered and is not mutated by the trait solving in progress (see
[why resolution runs in the emitter](typed-root-cause-resolution.md#why-it-runs-in-the-emitter) and
the [`after_analysis` unreachability](rustc-diagnostic-internals.md)). Frozen inputs are what make the
schema queries observationally pure and therefore cacheable and mockable. Had the resolver run in an
earlier phase where the graph could still change, none of this would hold — the placement forced by
`after_analysis` being unreachable is the same placement that makes the abstraction possible.

## The query taxonomy

Sorting every compiler interaction in `resolve/` by the cacheability lens yields three classes, and
each maps to a different place in the target architecture. The classes below are grounded in the
current query surface, not hypothetical.

**Class A — pure typed queries.** These are functions of `DefId`/`Ty` inputs against the frozen graph,
with no inference context and no hidden parameter; they are directly cacheable and directly mockable
(a table over a hand-built graph). This is the bulk of the surface and the most valuable to mock,
because it is where the subtle *decisions* live. It includes all of
[`cgp_item.rs`](../../crates/cargo-cgp-driver/src/resolve/cgp_item.rs) — the DefId-anchored
recognition (`is_cgp_item`, `provider_blanket_marker`, `consumer_provider_trait`, `marker_to_consumer`,
`is_namespace_lookup_trait`, `is_path_cons`, `decode_symbol`); the field inspection in
[`classify/field.rs`](../../crates/cargo-cgp-driver/src/resolve/classify/field.rs) (`adt_has_field`,
`deref_target`, `field_type`); the reportability decisions in
[`classify/reportable.rs`](../../crates/cargo-cgp-driver/src/resolve/classify/reportable.rs)
(`is_dispatch_lookup`, `owner_has_impl_of`, and thus `is_reportable_leaf`); the descent vocabulary in
[`walk/vocabulary.rs`](../../crates/cargo-cgp-driver/src/resolve/walk/vocabulary.rs); and the label
rendering in [`label/`](../../crates/cargo-cgp-driver/src/resolve/label). Underneath, these bottom out
on rustc queries that are themselves memoized and pure — `item_name`, `crate_name`, `all_impls`,
`trait_impls_of`, `predicates_of`, `impl_trait_ref`, `type_of`, `associated_items`, and the ADT/const
structural reads.

**Class B — pure signature, stateful implementation.** These are the three solver-driven queries:
`holds` in [`walk/holds.rs`](../../crates/cargo-cgp-driver/src/resolve/walk/holds.rs),
`impl_where_obligations` in
[`walk/impl_match.rs`](../../crates/cargo-cgp-driver/src/resolve/walk/impl_match.rs), and
`has_field_projection_mismatch` in
[`walk/projection_mismatch.rs`](../../crates/cargo-cgp-driver/src/resolve/walk/projection_mismatch.rs).
Each builds a fresh `InferCtxt` internally, so the implementation is stateful — but that state is fully
encapsulated behind a pure typed signature, which is exactly what makes the query cacheable at its
result boundary and mockable by a table. The obligation to discharge here is **canonical output**:
`impl_where_obligations` mints placeholders via `unknowns_to_placeholders`
([`walk/unknowns.rs`](../../crates/cargo-cgp-driver/src/resolve/walk/unknowns.rs)), and both the cache
value and the mock's answer must be stable modulo those placeholder identities.

**Class C — genuinely stateful or contextual.** Two different kinds of state live here, and they
belong in two different places. The first is the **cycle-guarded subtree**: `collect_leaf_paths` in
[`walk/leaves.rs`](../../crates/cargo-cgp-driver/src/resolve/walk/leaves.rs) carries the ancestor
`prefix` as a real parameter — the non-cacheable parameter that
[Stage 2 of the cache](cached-dependency-resolution.md#stage-2-the-interior-node-cache) must handle
with a cut-taint flag. But this is *not a compiler query*; it is the resolver's own traversal state,
so it stays in the rustc-free core and is tested directly, never entering the compiler interface. The
second is the **anchoring**: the anchors in
[`anchor/`](../../crates/cargo-cgp-driver/src/resolve/anchor) and
[`call_site/`](../../crates/cargo-cgp-driver/src/resolve/call_site) read spans, the diagnostic, and
raw HIR (the receiver's binding, the written argument types, HIR type lowering). This is per-diagnostic
contextual state coupled to the compiler's diagnostic and HIR representation — the least pure and least
mockable part of the resolver.

## The context boundary

The taxonomy draws the boundaries of the context type, and the key decision is to **not put every
dependency in one bag.** There are two lifetimes of state, and conflating them muddies the
abstraction:

- The **per-compilation** context — `ResolveCtx` — holds the compiler-query access (the Class A and
  Class B operations that become components later), the
  [dependency caches](cached-dependency-resolution.md), and the config constants. This is the shape a
  rustc-free stand-in context will later mirror.
- The **per-diagnostic** anchoring inputs — the diagnostic, its spans, HIR access — are passed as
  *inputs* to a resolution call, not stored in the context, because they are Class C contextual state
  and the rustc-coupled edge.

The walk and classification hang off the context, while anchoring is the thin rustc-coupled step that
turns a diagnostic into a **seed** (`Ctx: ConsumerTrait<Params…>`). The later stand-in tests bypass
anchoring and feed a seed directly — sound and desirable, because the seed is already the natural
rustc-free hand-off point and is what `resolve_leaves`, the part most worth testing, consumes. So the
honest scope of the later abstraction is: **the walk, the classification, and the labeling can be made
fully rustc-free; the anchoring cannot, need not, and should not — the seam between them is the seed
the resolver already produces.**

## The struct outline and its lifetimes

The lifetime question has a definite answer, and it settles the struct's shape: there cannot be one
reused context that carries the `TyCtxt`, but there is one reused *store*, so no cache is ever lost.
The reason there cannot be a single long-lived context holding the compiler is the same fact that
shaped [`ComponentNameMap`](driver.md#naming-the-traits-behind-a-component-marker). `CgpEmitter` is
constructed at session setup, *before any `TyCtxt` exists*, so it cannot name `'tcx`; and the tcx is
only reachable during emission through `rustc_middle::ty::tls::with`, which hands out a
`TyCtxt<'a>` for a fresh lifetime bounded by the closure. A `TyCtxt<'tcx>` therefore cannot be stored
past the `with` closure that produced it — the borrow checker forbids it, not merely discourages it.
Constructing a fresh context per resolution is thus forced, not a choice.

That construction is cheap, and it does not lose the cache, because the two kinds of state are split
by lifetime. The **long-lived, owned, lifetime-free state** — the [caches](cached-dependency-resolution.md)
and the config anchors — lives on the emitter and is reused across every resolution in the
compilation. The **short-lived, `'tcx`-scoped state** — the `TyCtxt` handle (or the rustc-backed query
provider that holds it) — is bundled into a per-resolution `ResolveCtx` built *inside* each
`ty::tls::with` closure, which **borrows** the long-lived store rather than owning it. So each
resolution gets a fresh binding to the compiler while the cache entries persist on the emitter behind
the borrow. Building the per-resolution context is a copy plus two borrows — `TyCtxt` is `Copy` — so
the per-resolution cost is negligible.

The rough shape is two structs, one per lifetime class:

```rust
// Long-lived, owned, no compiler lifetime. Lives on `CgpEmitter`, reused across every
// resolution in the compilation. Interior mutability so it is reachable both from the
// `&self` emitter methods and from inside a `ty::tls::with` closure.
struct ResolveCache {
    // Both cache stages share one map. The key is tcx-free — a `StableHash` fingerprint
    // of the region-erased seed/node, or an owned structural key under the owned-IR route
    // — and the value is the owned, rustc-free `Resolved` (`None` records a decline).
    trees: RefCell<HashMap<CacheKey, Option<Resolved>>>,
}

// The crate/trait-name anchors from `config.rs`, carried for the query methods.
struct ResolveConfig { /* CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT, … */ }

// Short-lived, `'tcx`-scoped. Built per resolution *inside* `ty::tls::with`, borrowing the
// long-lived store so its entries persist. Cheap: `TyCtxt` is `Copy`, the rest are borrows.
struct ResolveCtx<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    cache: &'a ResolveCache,
    config: &'a ResolveConfig,
}
```

The later CGP-component work does not change this split — it only changes what stands in the `tcx`
field's place. The rustc-backed context keeps the `TyCtxt<'tcx>` (staying `'tcx`-scoped and
per-resolution), while the separate stand-in context needs no compiler lifetime at all; the cache and
config are borrowed the same way in both. That component design is out of this document's scope; what
matters here is that the lifetime split already accommodates it.

Reuse across `ty::tls::with` entries is sound for the same reason the name map's is: one crate
compilation is one `GlobalCtxt`, so every entry yields a handle to the *same* underlying tcx, and the
cache's keys are tcx-free while its values are owned — an entry written under one entry is valid, and
correct, under any later one. The store outlives the individual `ResolveCtx` values that borrow it, and
the compilation outlives the store.

## Deferred: the CGP-component abstraction

The rustc-free stand-in and the CGP components it implements are designed later, so this section only
records what the current work must carry forward for that later design — it does not specify the
components. Three facts are worth handing off.

First, **the compiler-query surface is small and enumerable, which is what will make the abstraction
tractable.** The resolver already restricts itself to the CGP wiring vocabulary and DefId-anchors
everything — the same discipline that made it independent of `IsProviderFor` (see
[the walk](typed-resolution-walk.md)) — so the Class A and Class B operations are a bounded set, not
"all of rustc." The [query catalog](#the-first-artifact-the-query-catalog) the cache work produces is
the concrete enumeration.

Second, **the load-bearing difficulty is the type vocabulary, not the operation count.** The
operations traffic in `Ty<'tcx>`, `DefId`, `TraitRef<'tcx>`, and `GenericArgs<'tcx>`, and those types
cannot be inhabited without a live `TyCtxt` — a rustc-free stand-in cannot construct a `Ty<'tcx>` — so
the later components must be defined over an *abstracted* vocabulary (an owned model, or CGP abstract
types) rather than over rustc's types directly. That same choice decides the
[cache key](cached-dependency-resolution.md#the-cache-key): while the resolver walks `Ty<'tcx>` the key
is a `StableHash` fingerprint, and only if the later work moves the walk onto an owned model does the
key become an owned structural value. Until then, build the cache with the fingerprint key.

Third, **a stand-in validates the resolver's logic, not rustc's behavior.** For the Class A schema
operations a hand-built graph is a faithful substitute. For the Class B solver operations it is a
ceiling: the subtle paths — `resolve_fixed_projections` recovering a stalled projection, the
higher-ranked binder instantiation, the placeholder fold for later pipeline stages, all documented in
[the walk](typed-resolution-walk.md) — exist *because* rustc's solver does something non-obvious, and
a stand-in faithful enough to reproduce them is reproducing the very behavior in question. So the UI
snapshot suite stays ground truth for solver fidelity; the later stand-in tests cover the decision
logic and the corner cases awkward to build as whole programs, and must not breed confidence on the
paths that most need the real compiler.

## Sequencing

The work this document specifies is the first two steps; the CGP components and the stand-in context
are the deferred later work, out of scope here.

1. **Land the caches with the purity taxonomy made explicit.** This produces the query catalog and the
   per-query proof of complete input — the artifact the later component boundary is drawn from. See
   [Cached dependency resolution](cached-dependency-resolution.md).
2. **Introduce `ResolveCtx` as a rustc-backed struct** homing `tcx`, the caches, and the config,
   threaded in place of the loose parameters. A mechanical, low-risk, output-preserving refactor.
3. **(Later, out of scope.)** Define the resolver's compiler operations as CGP components over an
   abstracted type vocabulary, and add the separate stand-in context that implements them, enabling
   rustc-free tests of the walk, classification, and labeling.

### The first artifact: the query catalog

Before any code, produce the exhaustive per-call query catalog: every `tcx.*` call in `resolve/`,
tagged with its class (A / B / C), its complete input, its output type, and — for Class B — its
canonicalization obligation. That catalog is the cache-key specification now and the input to the
later component boundary, and building it will flush out any query classed as pure that secretly
carries a hidden parameter. It turns "abstract the compiler" from a judgment into a reviewable list.

## Comparison with Clippy

Clippy does not abstract the compiler for testing, so there is no Clippy design to follow here, and the
divergence is instructive. Clippy's own tests are UI tests over real compilation — it never needs a
mock `TyCtxt` because its lint passes run only on type-checking code and it has no rustc-free core to
exercise in isolation. cargo-cgp is pushed the other way by two facts particular to it: it re-runs
compiler work from inside the emitter (so its logic is unusually dense and error-prone, and worth
unit-testing), and it already keeps half of that logic rustc-free in `cargo-cgp-error-processing`. This
context type extends that existing rustc-free discipline leftward over the resolver's input half.
Where Clippy leans entirely on the compiler's query memoization, cargo-cgp inherits that same
memoization for its Class A queries and adds, on top, the resolver-level
[cache](cached-dependency-resolution.md) and — in the deferred later work — the CGP-component boundary
that a rustc-free stand-in can implement: layers Clippy has no need for because it adds diagnostics
rather than reshaping them.

## Tests (planned)

The tests below do not exist yet. The first is the near-term coverage for the work this document
specifies; the rest belong to the deferred component work and are listed so the later agent inherits
the intent.

- **Output-preservation guard (near-term)** — introducing `ResolveCtx` must leave the
  `tests/ui/acceptable/` snapshots unchanged; the refactor is behavior-preserving.
- **Stand-in decision-logic unit tests (later)** — once the components exist, exercise
  `is_reportable_leaf`, `is_dispatch_lookup`, `owner_has_impl_of`, the `field_issue` `Deref`-chain
  classification, and the label rendering over a hand-built graph, covering the branch corners (empty
  dispatch table, not-a-provider vs dead-end, present-via-`Deref`) that are awkward to reproduce as
  whole CGP programs today.
- **Parity spot-checks (later)** — a handful of fixtures whose stand-in-predicted resolution is
  compared against the real rustc-backed resolution, guarding that the stand-in's assumptions about
  Class B answers match the compiler on the shapes it claims to cover.

## Source (existing and planned)

Existing modules the context reorganizes:

- [`crates/cargo-cgp-driver/src/resolve/`](../../crates/cargo-cgp-driver/src/resolve) — the whole
  resolver, whose stages currently take `TyCtxt` directly; `cgp_item.rs`, `classify/`, `walk/`, and
  `label/` are the Class A + Class B surface the later components would cover, while `anchor/` and
  `call_site/` stay the rustc-coupled edge.
- [`crates/cargo-cgp-driver/src/config.rs`](../../crates/cargo-cgp-driver/src/config.rs) — the crate
  and trait-name constants the context carries.
- [`crates/cargo-cgp-error-processing/`](../../crates/cargo-cgp-error-processing) — the existing
  rustc-free crate whose discipline this extends; the natural home for the abstracted vocabulary and
  the decision core once the later work moves them off `rustc_private`.

Planned additions (near-term):

- The two-struct split from [the outline](#the-struct-outline-and-its-lifetimes): a long-lived,
  lifetime-free `ResolveCache` (plus a `ResolveConfig` of the name anchors) owned by `CgpEmitter` and
  reused across all resolutions, and a per-resolution `ResolveCtx<'a, 'tcx>` built inside `ty::tls::with`
  that carries the `TyCtxt` and borrows the store.
- The query catalog artifact.

Deferred (later work, out of scope here): the CGP components over an abstracted type vocabulary and the
separate rustc-free stand-in context that implements them.
