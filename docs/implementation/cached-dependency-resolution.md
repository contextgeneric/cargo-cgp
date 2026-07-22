# Cached dependency resolution

The typed root-cause resolver re-walks the same wiring many times — across the diagnostics one CGP
mistake produces, and across the branches of a single walk — so this document specifies one cache
that resolves each distinct obligation once and reuses the result everywhere it recurs, without
changing a byte of the output.

**Status: implemented.** The cache is consulted at **every node**: `resolve_node` memoizes each
interior node on its region-erased obligation and root context, with the incomplete-subtree flag at
population and the reachable-set disjointness check at consultation, and `resolve_leaves` folds the
root node's owned sub-result into the diagnostic. It is verified output-preserving against the whole
`tests/ui/acceptable/` suite, with a dedicated diamond fixture pinning interior reuse and a mutual-cycle
fixture pinning the incomplete-subtree cut on a resolving walk. What is still not isolated by a test is
the reuse disjointness check, whose failure mode is an exotic cross-diagnostic multi-node cycle the
snapshot suite cannot express on its own (see [Tests](#tests)).

It is a companion to [The resolve context](resolve-context.md), which houses the cache and frames it
as one instance of a larger goal — a rustc-free, mockable resolution core — and to
[Typed root-cause resolution](typed-root-cause-resolution.md) and
[its walk stage](typed-resolution-walk.md), whose `resolve_leaves` / `resolve_node` descent is
the thing being cached.

## Why cache the resolution at all

The cache exists to remove redundant re-resolution, and the redundancy is real, documented, and of
two kinds. The first is across diagnostics: CGP wiring is lazy, so one mistake surfaces the same
failure at many sites — the `check_components!` entry, every hand-written `impl` that references the
broken consumer, and each call — and the
[money-transfer example](https://github.com/contextgeneric/cgp/blob/main/docs/examples/money-transfer-api.md)'s single un-wired password
type produced eighteen identical root-cause trees this way. Today the emitter resolves every one of
those to completion and only then drops the duplicates through the
[`DedupLedger`](error-processing.md), so sixteen full walks are computed and thrown away. The second
kind sits inside a single walk: a capability depended on by several providers (a shared getter,
`HasErrorType`) is a diamond in the dependency graph, and its subtree is walked once per parent. The
unified cache below closes both, because both reduce to the same primitive — resolve each distinct
obligation once.

The deeper reason to build this, though, is not speed — it is that **a cacheable query is a stateless
query, and statelessness is the property that makes the resolver possible to reason about and
eventually to test without a compiler.** A query you can cache is a pure function of its explicit
typed inputs; a query you cannot cache without an extra parameter has hidden state that the extra
parameter names. So writing the cache is a forcing function: it makes every load-bearing question the
resolver asks the compiler either provably pure or explicitly stateful, and it draws the line between
the two. [The resolve context](resolve-context.md) develops that consequence in full; this document
is the concrete cache the reasoning produces.

## What is cached, and why it is safe to keep across diagnostics

The value cached is the resolver's rustc-free output, and that is what makes the whole scheme sound
against the compiler's lifetimes. The walk's finished product is owned `String`-only data in the
compiler-free `cargo-cgp-error-processing` crate — the classified
[`Leaf`](../../crates/cargo-cgp-error-processing/src/diagnosis/leaf.rs) values and the structured
[chain-node](../../crates/cargo-cgp-error-processing/src/diagnosis/node.rs) paths that a
[`Resolved`](../../crates/cargo-cgp-error-processing/src/diagnosis/resolved.rs)'s causes hold —
carrying no `Ty<'tcx>`, no `DefId`, no compiler handle (the paths are merged and rendered into a tree
later, by the rustc-free [dependency graph](dependency-graph-rendering.md)). So a cached value can
live for the whole compilation on a struct that outlives no single `TyCtxt`, exactly as the existing
[`ComponentNameMap`](driver.md#naming-the-traits-behind-a-component-marker) and
[`DedupLedger`](driver.md) already do. This is the decisive difference from caching the *intermediate*
solver work: an `InferCtxt`'s obligations are `'tcx`-interned and cannot be stored past the
`ty::tls::with` closure that produced them, but the finished paths are owned and can.

Caching the finished paths carries no staleness risk, for the same reason the name map does not. The
resolver runs at emit time, over compiler state that is frozen — the trait set, the impls, the
`predicates_of`, the ADT field lists are all fixed once the crate is lowered, and trait solving does
not mutate them (see [why resolution runs in the emitter](typed-root-cause-resolution.md#why-it-runs-in-the-emitter)
and the [`after_analysis` unreachability](rustc-diagnostic-internals.md)). A subtree resolved once is
therefore valid for the rest of the crate's compilation.

## One cache, keyed by node

The cache memoizes the walk **at every node**, so there is one mechanism and one key space, not a
special case for whole seeds and another for interior nodes. The resolver's descent is a recursive
function over trait obligations: `resolve_node` visits an obligation, descends into the
`where`-clause obligations of the impl that satisfies it, and bottoms out on terminal leaves. Every
obligation it visits — a consumer trait, a provider trait, a plain Rust trait, a getter bound — is a
**node**, and every node is a cache entry keyed on that node's obligation. At each step of the walk
the resolver looks the node up and short-circuits on a hit; when it misses it computes the subtree,
converts it to the owned form, and files it under the node's key.

This retires the "seed versus interior node" vocabulary an earlier draft carried, and with it the
confusion between a *seed* and a *cache key*. A **seed** is now just a role — the obligation an
[anchor](typed-resolution-anchors.md) recovers and hands the walk as its **root**, always a real
consumer obligation `Ctx: ConsumerTrait<Params…>` (or a wrapper trait from the impl-site anchor). The
seed is the root node of one walk, nothing more; it is keyed and looked up exactly like every other
node. Both redundancies then close through the same lookup at different depths: the eighteen identical
trees share a root node, so the first resolves it and the other seventeen hit the cache at the root,
and the diamond's shared capability is a node reached from two parents, so the second parent hits the
cache one level down.

The stored value is the node's set of **root-cause sub-chains**, each an owned `Leaf` plus the label
chain from that node down to the leaf. The walk builds this bottom-up, so the sub-chains are stored
*node-rooted* — a child returns chains that begin at the child's own label, and its parent prepends
its own label to each. Because the value is already rooted at the node, both uses need it verbatim:
when the node is a future seed the resolver builds the `Resolved` header from the node and uses the
sub-chains directly, and when the node is spliced into a larger walk the parent prepends its label and
continues. There is no separate re-rooting pass. Only *non-terminal* nodes are cached: a terminal
leaf's owned classification reads its parent obligation, which lives outside a leaf-rooted subtree,
and a leaf is never reused as a seed and never expensive to re-derive, so caching leaves would buy
nothing and break the node-alone key. Every non-terminal node's cached sub-chains classify their
leaves against parents drawn from *within* the subtree, so the value is self-contained.

## Why a node key is not automatically a complete key

The walk is **not** a pure function of the node, and this is the one fact the whole soundness argument
turns on. `resolve_node(node, …, prefix, …)` takes the node *and* the set of its ancestors (the
`prefix`), because of the cycle guard: the guard cuts a branch the moment the current obligation
reappears among its ancestors, so the leaves below a node are a function of `(node, ancestor-set)`,
not of the node alone.
A cycle reaches the walk when wiring loops — the `UseContext` self-routing shape, normally intercepted
upstream as the `E0275` overflow the driver rewrites to `[CGP-E010]` — but the cycle guard exists
precisely so the walk does not *rely* on that interception, and the cache must not reintroduce the
reliance. Memoizing on the node and consulting it under a different ancestor set can therefore be
wrong, in two directions:

- A subtree computed under a prefix where a descendant looped back to a prefix ancestor was **cut** at
  that descendant, so it omits whatever lay past the cut. Reuse it under a prefix that lacks that
  ancestor and the walk **under-reports** — a real root cause silently missing.
- A subtree computed where no cut occurred, reused under a prefix that *would* cut, **over-reports** a
  leaf a correct walk would have severed.

Either way the tree is wrong, and a wrong tree that looks clean is the worst outcome for this tool. So
the node key is completed with two guards — one at population, one at consultation — and both are
no-ops in acyclic wiring, which is nearly all wiring, so the clean node-keyed design works for free
almost always and the machinery earns its keep only where a cycle actually reaches the walk.

### Population: cache only complete subtrees

The first guard is that **only a subtree no guard curtailed may be cached.** Two guards can cut a
subtree short and leave it incomplete: the cycle guard above, and the `MAX_DEPTH` backstop that stops
a pathological or divergent walk before it overflows the stack. Both truncate silently — a cut branch
produces no output — and both are position-dependent, because a node sitting deep in one walk has
fewer frames of depth budget left than the same node walked from a shallow position. A subtree that
hit either guard is incomplete and must not be stored, or a later reuse would under-report.

The trap that makes this subtle is that **a cut leaves no trace in the finished tree.** When a guard
cuts a branch it produces no output, and within any surviving path there are never repeats, so any
after-the-fact test computed from the completed tree always comes back clean even when a branch was
severed. Eligibility therefore cannot be derived from the output; it must be recorded *during* the
walk. The walk carries one **incomplete-subtree** flag on each node's sub-result: whenever the cycle
guard cuts on a collision, or a branch reaches `MAX_DEPTH`, the cut sub-result is flagged, and the flag
rides up through each parent to the root (`incomplete |= child.incomplete`), so every ancestor whose
result folded in the cut is flagged too. A flagged node is not cached. That flag is the only fact the
output tree cannot reconstruct.

Propagating the flag all the way to the root is a deliberate over-approximation. The tightest correct
rule would stop at the cycle's entry ancestor, since an ancestor *above* that entry contains the cycle
wholly within its own subtree — the loop is then intrinsic and position-independent, so that ancestor
is in fact safely cacheable. Distinguishing that case would mean threading the collision's depth back
up the recursion, and the payoff is only a few extra cache entries in the rare crate where a cycle
reaches the walk at all. So the walk takes the simpler path and never caches a curtailed ancestor;
never caching too much only costs a re-walk, never a wrong tree.

A node cached under this rule is therefore **complete**: every branch of its subtree bottomed out on a
natural terminator — a `HasField` leaf, an impl-less bound, a foreign bound, a projection mismatch —
and none was cut by budget or cycle. Completeness is what makes the subtree *position-independent along
the depth axis*: since no branch depended on the remaining depth budget, a fresh walk of the node with
a larger budget reaches the same terminators and produces the identical subtree. A cache hit can thus
legitimately return a fuller subtree than a fresh bounded walk from a deep position would — the cache
does no worse, and sometimes better, than re-walking.

### Consultation: reuse only where no cycle would form

The second guard is that **a cached subtree is reused at a node only when the current ancestor set is
disjoint from it.** Completeness settles the depth axis but not the cycle axis: a complete subtree of
node `N` is the true *acyclic* subtree, and reusing it under a prefix that shares an obligation with it
would splice in a branch the cycle guard should cut. Concretely — node `N` is cached from a walk whose
path never went through obligation `A`, so `N`'s subtree runs down through a node `D` (where `D`'s
obligation equals `A`) with no cut, and is cached complete. A later walk descends through `A` and
reaches `N`; a correct walk cuts at `D` because `A` is now an ancestor, but reusing `N`'s cached
subtree splices the full path through `D` back in and over-reports.

The fix is to store, alongside each cached subtree, the set of obligation fingerprints it reaches —
its **reachable set** — and to reuse the entry only when none of the current ancestors is in it. The
ordinary cycle guard already declines when the node itself is an ancestor; the reachable-set check
extends that to a *descendant* being an ancestor, which is the only remaining way reuse could form a
cycle. In acyclic wiring an ancestor can never appear in any subtree — it would be a repeat, hence a
cycle — so the check always passes and the hit rate is full, diamonds included. The check engages, and
declines a reuse, only when a genuine multi-node cycle reaches the walk, which is exactly where relying
on the node-alone key would be unsound.

## The cache key

The key is **a small struct hashed and compared by a single `HashStable` fingerprint, carrying
readable fields alongside purely so the cache store can be inspected.** Its identity — the only thing
`Hash` and `Eq` read — is a 128-bit `Fingerprint` of the region-erased obligation, the root context,
and the `ParamEnv`; the remaining fields render the obligation as text and never affect a lookup. This splits
the two jobs a key does — being a correct identity, and being legible — so each is served by the tool
best suited to it, and it rests on the completeness principle [the resolve context](resolve-context.md)
is built on: the identity *is* the node's input, so every input the walk's output depends on must be
in it.

The fingerprint is the identity because rustc supplies its encoding, and that encoding is both total
and faithful in ways a hand-written one would struggle to be. `HashStable` walks the whole structure
of a `Ty` / `GenericArgs` / `TraitRef` — every `TyKind`, including the exotic ones — and encodes each
`DefId` by its stable path identity rather than its name. Encoding by path identity is the one property
the key cannot do without: two same-named types in different modules share a name but not a
`DefPathHash`, and the `same_name_components` fixture pins the resolver on telling them apart. A
name-based *string* could not be the identity for exactly this reason, which is why "why not just
serialize the type to a string?" resolves to "because the faithful serializer already exists as
`HashStable`, and reusing it is safer than rebuilding it." The fingerprint is also `Copy` and
lifetime-free, so it lives on the store past any `TyCtxt`, where the interned `Ty<'tcx>` it summarizes
cannot. Its one cost is the one rustc itself accepts: a 128-bit collision is not *impossible*, and if
one ever occurred the result is a wrong message on an already-failing build — acceptable where a
miscompile would not be, and the standard rustc trusts for its own query and incremental caches.

Because the fingerprint carries all the correctness, **the remaining fields are free to be readable
rather than injective.** They exist so a maintainer investigating a hit or a miss can dump the cache
store and see, in plain text, which obligations were resolved and which were not — the context, the
consumer or provider trait, its parameters — without decoding a wall of fingerprints. Since they never
feed `Hash` or `Eq`, they may render ambiguous short names, resugared spines, whatever reads best; two
entries that happen to display alike are still distinct entries under their distinct fingerprints. This
is the payoff of splitting the key: nothing hand-written has to be total or injective, so the hazard of
a hand-rolled serializer silently merging two shapes — a *deterministic* wrong hit, worse than the
fingerprint's astronomically unlikely one — never arises.

A draft of the struct makes the split concrete. `Hash` and `Eq` are hand-written on the fingerprint
alone rather than derived, because deriving would fold the debug fields into equality and defeat the
whole arrangement, while `Debug` is derived so the store can be dumped:

```rust
/// Cache key for a resolved node. Identity is the fingerprint; the rest is for humans.
#[derive(Clone, Debug)]
struct NodeKey {
    /// The sole basis for `Hash`/`Eq`: a `HashStable` fingerprint of the region-erased
    /// obligation, the root context, and the `ParamEnv`. `Copy` and lifetime-free, so it
    /// outlives any `TyCtxt`.
    fingerprint: Fingerprint,
    /// Debug-only, never read by `Hash`/`Eq`: the obligation and root context rendered as
    /// text, e.g. `Rectangle: AreaCalculator<Circle>` under `Rectangle`, and the
    /// environment once it is non-empty.
    obligation: String,
    context: String,
    param_env: String,
}

impl PartialEq for NodeKey {
    fn eq(&self, other: &Self) -> bool {
        self.fingerprint == other.fingerprint
    }
}
impl Eq for NodeKey {}

impl std::hash::Hash for NodeKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.fingerprint.hash(state);
    }
}
```

The root context is the second input, because the resolver's rendering is not a function of the node's
obligation alone. `label_for` and `classify_leaf` both compare a node's `self_ty` against the *root*
context: a consumer obligation `Ctx: Consumer` renders as a consumer impl only when `Ctx` is that
context, and an unmet `DelegateComponent` classifies as a missing wiring — rather than a dispatch entry
or a not-a-provider — only when its owner is that context. The context is threaded unchanged from the
seed's `self_ty`, so it never varies *within* a tree; but an interior node with the same obligation
reached under a *different* root context — the cross-context CGP pattern, where one context's wiring
depends on an obligation about another — renders differently, so reusing a subtree cached under one
context beneath another would mislabel it. Keying on the context alongside the obligation makes that a
cache miss, which is correct: the same obligation under a different context is a different rendering.
For a seed the context is the obligation's own `self_ty` and adds nothing; it earns its place only once
interior nodes are cached across trees. The
[`cross_context_node_key`](../../tests/ui/acceptable/resolution/cross_context_node_key.rs) fixture is
the cross-context shape that exercises this — `Outer`'s walk re-roots the shared `Inner: CanCompute`
node at `Inner` (so it keys identically to `Inner`'s own walk and shares that cached subtree),
rendering it as a consumer impl for context `Inner`, while `Inner`'s own tree renders the same node
the same way; the context field is what keeps a subtree cached under one root context from being
reused under another where it would mislabel.

The parameter environment is the third, and folding it in now is cheap insurance against a latent
unsoundness. The walk currently solves every obligation in an **empty** `ParamEnv`, which is a
constant and therefore not a hidden input — the reason the obligation alone has sufficed as a key in
the draft. But the walk stage records that extending to checks carrying generic parameters will need
the impl's *own* environment (see [walking to the root cause](typed-resolution-walk.md)), and the
moment that lands the `ParamEnv` becomes a real input: two identical obligations under different
environments would collide. `ParamEnv` is `HashStable`, so it hashes into the fingerprint alongside the
obligation. Fold it in from the start, empty — an empty environment contributes a fixed, empty
increment at negligible cost — so the key is already complete before the walk ever gains a non-empty
one, rather than depending on a future agent to remember to add it exactly when the walk changes and
its omission would be silent.

One coverage ceiling rides on the key and is a limitation of hit rate, never of correctness. A
[call-site seed](typed-resolution-call-site.md) carries rigid placeholders whose identities are part
of the region-erased obligation and thus part of the fingerprint, so two different call sites of the same unknown-argument
capability key distinctly and do not cross-hit. This is correct — distinct placeholders make distinct
obligations — and even a coincidental cross-hit would be output-preserving, since both walks resolve
the unknown identically; the ceiling is only that the cache misses where it harmlessly could hit.

Keying by fingerprint is the pragmatic choice for as long as the walk runs on `Ty<'tcx>`. The deferred
[resolve-context](resolve-context.md#deferred-the-cgp-component-abstraction) work would move the walk
onto an owned type model, at which point the node is owned data and structural equality could key it
directly — the readable fields would become the identity and the fingerprint would fall away. That is a
possible future, not a decision to make now; build the fingerprint-keyed struct, and revisit only if
that refactor lands.

## How the cache relates to the de-duplication ledger

The cache and the `DedupLedger` act on different axes and compose without overlap, and seeing the
split prevents building the wrong thing. The ledger de-duplicates by *recovered cause* — the
span-independent [`cause_signature`](driver.md) of context, consumer, and leaves — computed **after**
the walk, to decide what is *shown*. The cache keys by *node obligation* — computed **before and
during** the walk — to decide what is *resolved*. They are complementary: the cache removes the
redundant walking, and the ledger still governs the redundant showing.

The cache reaches a strictly larger redundancy than the ledger, which is why both are needed. Two
diagnostics can reach the same shown cause through *different* seeds — the money-transfer wrapper impl
seeds a `[CGP-E009]` wrapper trait (deliberately its own block) while the check entry seeds the
consumer, and the `density_3` / `dependency_cascade` shapes reach one field through distinct consumers.
The ledger collapses or keeps those on the output side as before. The cache, meanwhile, saves the walk
whenever any node recurs — even when the outputs do *not* de-duplicate, as when a later diagnostic's
cause is a strict sub-cause of an earlier one: the ledger keeps both blocks (distinct signatures), yet
the overlapping subtree is walked once. That overlap is redundancy the ledger structurally cannot see.

## Where the cache lives

The cache is one store, and it lives on the [resolve context](resolve-context.md) — the
per-compilation struct that also holds the compiler-query access and the config constants. Interior
mutability (a `RefCell` around the map) matches how `DedupLedger` is already carried on `CgpEmitter`,
and the store is created once per driver invocation over one crate, with no cross-session incremental
database that could change underneath it. Until the resolve context exists, the store may sit directly
on `CgpEmitter` beside `dedup`; folding it into the context is part of that document's refactor.

The borrow discipline is worth stating because this codebase is acutely sensitive to re-entrancy — the
whole of [rustc diagnostic internals](rustc-diagnostic-internals.md) is about a lock that panics on
re-entry. The memo must never hold the `RefCell` borrow across the compute: check the cache and release
the borrow, compute on a miss with no borrow held, then take the borrow again to insert. This is safe
because the memoized descent never re-enters its own borrow — the recursion is `resolve_node`
calling itself, and the cache read and write bracket each call rather than spanning it — so the memo
boundary is clean even though the resolver runs inside a diagnostic being emitted.

## Sequencing and relation to diagnostic buffering

The cache was built in two increments, and the split between them is the one that is sound to make.
Root consultation — memoizing at a walk's seed — needs neither guard: the seed has an empty ancestor
prefix, so the cycle question never arises, and it is always resolved at full depth, so the
incomplete-subtree flag never bites. It landed first as a sound standalone step that captures the
dominant cross-site win. Interior consultation — consulting at every node — is the piece that must go
in as a whole, because the node key's context field, the incomplete-subtree flag, and the reachable-set
disjointness check are not separable: consulting a cached subtree mid-walk without them is unsound the
moment a cycle reaches the walk. Both are now in. The reachable-set guard stays even though its cost is
a no-op in acyclic wiring, because dropping it would reintroduce the reliance on upstream cycle
interception the resolver refuses.

The cache is complementary to the diagnostic-buffering work the [usability issues](../issues/usability.md)
anticipate, not in competition with it. Buffering would decide *what* to emit — coalescing even
different-consumer-same-cause blocks into one listing — while a node-memoized walk is the "resolve each
unique obligation exactly once" primitive a buffered emitter would want underneath it. So this cache is
a building block for that later work, not throwaway if it lands.

## Comparison with Clippy

Clippy has no analog to this cache, because it has no analog to the work being cached. Clippy's late
passes run only on code that type-checks (the same `after_analysis` gate that forces cargo-cgp's
resolver into the emitter), so Clippy never re-runs the trait solver from inside diagnostic emission
and never resolves the same failure at many sites. Where Clippy relies on the compiler's own query
memoization for repeated `TyCtxt` lookups, cargo-cgp inherits that same memoization for its schema
queries (see [The resolve context](resolve-context.md)) — this cache adds the layer rustc does *not*
provide: memoizing the resolver's own composite walk, whose result is the rustc-free tree. There is no
Clippy code to follow here; the design is particular to reshaping errors rather than adding them.

## Tests

The primary guard is met; the rest is coverage still to add.

- **Output-preservation regression guard** (met) — the existing `tests/ui/acceptable/` snapshot suite
  passes with no re-bless after the cache landed, both the root and the interior layer. This is the
  primary correctness check: the cache is pure memoization, so any snapshot change would be a bug. It
  also exercises the incidental paths — the cross-site re-report reuse (`cross_site_dedup`,
  `manual_supertrait_impl`) and, where a cycle reaches the walk, the incomplete-subtree cut
  (`use_context_cycle`, a self-cycle that declines).
- **Cycle cut in a resolving walk** (met) —
  [`acceptable/wiring/constraints/mutual_cycle_with_cause`](../../tests/ui/acceptable/wiring/constraints/mutual_cycle_with_cause.rs)
  wires a two-component mutual cycle (`ProviderA` depends on `CanB`, `ProviderB` back on `CanA`)
  alongside a genuinely missing field. The walk descends the `CanA → CanB → CanA` loop, the cycle
  guard cuts it (flagging that subtree incomplete), and the missing-field cause down the other branch
  is still reported as a clean `[CGP-E106]` tree — where rustc's raw error surfaces the cycle as
  repeated `App: CanA` requirements and buries the field. This is the first cyclic fixture that
  *resolves* rather than declining, so it pins the cut and the incomplete-flag propagation on a
  live tree.
- **Diamond reuse** (met) —
  [`acceptable/resolution/diamond_shared_capability`](../../tests/ui/acceptable/resolution/diamond_shared_capability.rs)
  routes two independent branches — `CanTop` depends on both `CanLeft` and `CanRight` — through one
  shared `CanShared` capability whose provider needs a `name` field the context lacks. The walk
  descends `App: CanShared` twice (once per branch), so the interior node is resolved under the first
  branch and consulted from the cache under the second; the two identical missing-field causes
  de-duplicate to one, and the tree shown is the first branch's. It pins that an interior cache hit is
  output-preserving on a minimal, purpose-built diamond rather than only incidentally through the
  serialization fixtures.

The tests below do not exist yet; they are the coverage still to add.

- **Determinism** — a unit test (or a repeated no-bless UI run) confirming the walk yields the same
  owned sub-chains for the same node, which is the purity the cache assumes; the resolver's placeholder
  canonicalization, and the branch order that decides which tree a de-duplicated cause keeps, are what
  this pins.
- **Whole-crate hit accounting** — an instrumented run over an error-heavy fixture (the money-transfer
  shape) asserting that the eighteen-tree case computes the walk once per distinct node, not once per
  site.
- **Soundness under a cut / under reuse** — the two guards (the incomplete-subtree taint and the
  reachable-set disjointness check) are exercised by the fixtures above but not *isolated* by one that
  would fail if either guard were removed. Such a fixture is hard to construct as a UI snapshot, for a
  structural reason worth recording: within a single walk an ancestor-induced cut never loses a cause
  (the ancestor's own causes were already reported where it was first visited), so a broken taint
  changes the output only when a node cut under one diagnostic's ancestor set is *reused* under
  another's where that ancestor is absent — an exotic cross-diagnostic multi-node cycle. Verifying such
  a fixture actually catches the bug also means temporarily removing the guard to watch it fail, which
  the snapshot suite alone cannot express. The honest coverage is the output-preservation guard plus
  the cycle-cut fixture above; a targeted guard-removal test is left as future work.

## Source

- [`crates/cargo-cgp-driver/src/resolve/cache.rs`](../../crates/cargo-cgp-driver/src/resolve/cache.rs)
  — the `NodeKey` (a `StableHash` `Fingerprint` of the region-erased obligation and its context for
  `Hash`/`Eq`, with readable debug fields and a derived `Debug`), the owned `SubCause` / `SubResult`
  values (the node-rooted sub-chains, the reachable-fingerprint set, and the incomplete flag), the
  `pred_fingerprint` helper for reachable entries, and the `ResolveCache` store (a `RefCell<HashMap>`
  of `SubResult`).
- [`crates/cargo-cgp-driver/src/resolve/walk/leaves.rs`](../../crates/cargo-cgp-driver/src/resolve/walk/leaves.rs)
  — `resolve_node` memoizes each node: it consults the cache (reusing a complete subtree only when the
  current ancestor prefix is disjoint from its reachable set), descends into terminal / projection /
  merge branches building owned node-rooted sub-chains, flags a cycle or depth cut incomplete, and
  caches every complete non-terminal node. `resolve_leaves` erases the seed and delegates to
  `compute_leaves`, which folds the root node's `SubResult` into a `Resolved` (de-duplicate, elide,
  render).
- [`crates/cargo-cgp-driver/src/emitter/cgp_emitter.rs`](../../crates/cargo-cgp-driver/src/emitter/cgp_emitter.rs)
  — `try_resolve` threads `&self.resolve_cache` through the seven anchors to `resolve_leaves`; the
  `ResolveCache` lives on `CgpEmitter` beside `dedup` (moving it onto the resolve context is that
  document's refactor). The `DedupLedger` it composes with is here too.
- [`crates/cargo-cgp-error-processing/src/diagnosis/`](../../crates/cargo-cgp-error-processing/src/diagnosis)
  — the owned `Leaf` and `Resolved` values, and the label rendering, that the cache stores.
