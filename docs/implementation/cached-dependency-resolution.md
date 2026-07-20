# Cached dependency resolution

The typed root-cause resolver re-walks the same wiring more than once — across the many diagnostics
one CGP mistake produces, and across the branches of a single walk — so this document specifies a
two-stage cache that resolves each distinct sub-problem once and reuses the result, without changing
a single byte of the output.

**Status: blueprint ahead of implementation.** Nothing here is built yet. The document records the
motivation and the design decisions agreed for the work, so a later agent can implement it directly.
It is a companion to [The resolve context](resolve-context.md), which houses the caches and frames
them as one instance of a larger goal — a rustc-free, mockable resolution core — and to
[Typed root-cause resolution](typed-root-cause-resolution.md) and
[its walk stage](typed-resolution-walk.md), whose `resolve_leaves` descent is the thing being cached.

## Why cache the resolution at all

The cache exists to remove redundant re-resolution, and the redundancy is real and documented. CGP
wiring is lazy, so one mistake surfaces the same failure at many sites — the `check_components!`
entry, every hand-written `impl` that references the broken consumer, and each call — and the
[money-transfer example](../../../cgp/docs/examples/money-transfer-api.md)'s single un-wired password
type produced eighteen identical root-cause trees this way. Today the emitter resolves *every one of
those* to completion and only then drops the duplicates through the
[`DedupLedger`](error-processing.md), so sixteen full walks are computed and thrown away. A second,
smaller redundancy sits inside a single walk: a capability depended on by several providers (a shared
getter, `HasErrorType`) is a diamond in the dependency graph, and its subtree is walked once per
parent.

The deeper reason to build this, though, is not speed — it is that **a cacheable query is a stateless
query, and statelessness is the property that makes the resolver possible to reason about and
eventually to test without a compiler.** A query you can cache is a pure function of its explicit
typed inputs; a query you cannot cache without extra parameters has hidden state that the extra
parameters name. So writing the cache is a forcing function: it makes every load-bearing question the
resolver asks the compiler either provably pure or explicitly stateful, and it draws the line between
the two. [The resolve context](resolve-context.md) develops that consequence in full; this document
is the concrete cache the reasoning produces.

## What is cached, and why the value is safe to keep across diagnostics

The value cached is the resolver's rustc-free output, and that is what makes the whole scheme sound
against the compiler's lifetimes. `resolve_leaves` returns
[`Option<Resolved>`](../../crates/cargo-cgp-error-processing/src/diagnosis/resolved.rs), and
`Resolved` is owned `String`-only data in the compiler-free `cargo-cgp-error-processing` crate — no
`Ty<'tcx>`, no `DefId`, no compiler handle. So a cached `Resolved` can live for the whole compilation
on a struct that outlives no single `TyCtxt`, exactly as the existing
[`ComponentNameMap`](driver.md#naming-the-traits-behind-a-component-marker) and
[`DedupLedger`](driver.md) already do. This is the decisive difference from caching the *intermediate*
solver work: an `InferCtxt`'s obligations are `'tcx`-interned and cannot be stored past the
`ty::tls::with` closure that produced them, but the finished tree is owned and can.

Caching the tree carries no staleness risk for the same reason the name map does not. The resolver
runs at emit time, over compiler state that is frozen — the trait set, the impls, the `predicates_of`,
the ADT field lists are all fixed once the crate is lowered, and trait solving does not mutate them
(see [why resolution runs in the emitter](typed-root-cause-resolution.md#why-it-runs-in-the-emitter)
and the [`after_analysis` unreachability](rustc-diagnostic-internals.md)). A tree resolved once is
therefore valid for the rest of the crate's compilation.

## Stage 1: the whole-seed cache

Stage 1 memoizes `resolve_leaves` keyed on its seed obligation. Every one of the six
[anchors](typed-resolution-anchors.md) funnels its recovered obligation `Ctx: ConsumerTrait<Params…>`
into `resolve_leaves`, so wrapping that single function with a memo covers all anchor kinds at once.
The key is the region-erased seed obligation (`resolve_leaves` already erases it on entry); the value
is the `Option<Resolved>` it returns.

Stage 1 is **output-preserving**: it is pure memoization of a pure function, so a hit returns
byte-identical `Resolved` and no UI snapshot changes. That is the property to hold onto through
implementation — the regression guard is that the whole `tests/ui/acceptable/` suite re-blesses to
*nothing*.

Its soundness rests on one fact about where the cache boundary sits. `resolve_leaves` always starts
its descent with an **empty ancestor prefix**, so the seed is the one point in the whole walk where
the cycle guard's ancestor set — the hidden parameter that Stage 2 must contend with below — is empty
and therefore not a hidden input. Caching at the seed is caching at the only boundary where the
obligation alone is the complete key.

Two further decisions round out Stage 1. **Negative results are cached**: a seed that declines
(`None`) declines deterministically, so caching `None` avoids re-walking a declining seed, and the
anchor chain in `try_resolve` keys each anchor's seed independently. And a **hit-rate ceiling** is
accepted rather than fought: a [call-site seed](typed-resolution-call-site.md) carries rigid
placeholders whose identities are minted per anchor, so two different call sites of the same
unknown-argument capability hash to different keys and will not cross-hit. This is a limitation of
coverage, never of correctness — distinct placeholders make distinct obligations, so they *should*
key distinctly.

### How Stage 1 relates to the de-duplication ledger

Stage 1 and the `DedupLedger` partition the work along different axes, and understanding the overlap
prevents building the wrong thing. The ledger de-duplicates by *recovered cause* — the
span-independent [`cause_signature`](driver.md) of context, consumer, and leaves — computed **after**
the walk. Stage 1 keys by *seed* — computed **before** the walk. Same seed always yields the same
cause, so every Stage 1 hit corresponds to a diagnostic the ledger would then drop: Stage 1 removes
the *walk* for those re-reports, and the ledger still governs what is *shown*. They compose, and
neither replaces the other.

The caveat is that Stage 1 catches a strict subset of what the ledger suppresses. Two diagnostics can
reach the same cause through *different* seeds — the money-transfer wrapper impl seeds a `[CGP-E009]`
wrapper trait (deliberately its own block) while the check entry seeds the consumer, and the
`density_3` / `dependency_cascade` shapes reach one field through distinct consumers. Those have
different keys, so Stage 1 misses them and they still walk in full; the ledger handles the output side
as before. The big collapsing group — one consumer method checked or called at many sites — does share
a seed, so Stage 1 lands on it.

## Stage 2: the interior-node cache

Stage 2 caches each *step* of a completed walk, so a later diagnostic whose seed equals an interior
node of an earlier diagnostic's tree reuses that node's subtree instead of re-walking it. This is a
strictly larger hit set than Stage 1, which only hits when two whole seeds coincide; a common CGP
shape makes the difference concrete — a context that checks a high-level component (whose tree
descends through capability `C`) and also uses `C` directly seeds a second resolution *at* `C`, an
interior node of the first tree. Crucially Stage 2 saves the walk **even when the outputs do not
de-duplicate**: if the second diagnostic's cause is a strict sub-cause of the first, the ledger keeps
both blocks (distinct cause signatures), yet the overlapping subtree need not be walked twice. Stage 2
captures redundancy the ledger structurally cannot.

The cache is **populated as a post-pass after each full resolution**, not consulted mid-recursion.
This keeps `resolve_leaves` pure and its cycle guard untouched: the walk builds the tree exactly as it
does today, and a separate pass files each eligible interior node's subtree under its key. Populating
at the edge rather than inside the traversal matches the codebase's "side effects at the thin edges"
discipline and is the right seam — but the *timing* of the write is orthogonal to the soundness of the
*read*, which is where the real work is.

### The cycle-guard soundness problem

The reason Stage 2 cannot naively key a subtree on its obligation alone is the cycle guard.
`collect_leaf_paths` cuts a branch the moment the current obligation reappears among its ancestors, so
the leaves below an obligation `X` are a function of `(X, ancestor-set)`, not of `X` alone. Cache the
subtree under `X` and reuse it under a different ancestor set, and the result is wrong — not merely
conservative:

- Computed under a prefix where one of `X`'s descendants loops back to a prefix ancestor, the guard
  cut that branch, so the cached subtree **omits** whatever lay past the cut. Reuse it under a prefix
  that lacks that ancestor and you **under-report** — a real root cause silently missing.
- Cache it where no cut occurred and reuse it under a prefix that *would* have cut, and you
  **over-report** a leaf a correct walk would have severed.

Either way the tree is wrong, and a wrong tree that looks clean is the worst outcome for this tool.
The failure is confined to cyclic wiring reaching the walk — the `UseContext` self-routing shape,
normally intercepted upstream as the `E0275` overflow the driver rewrites to `[CGP-E010]` — but the
cycle guard exists precisely so the walk does not *rely* on that interception, and Stage 2 must not
reintroduce the reliance.

### The invisible-cut trap, and the one bit that fixes it

The trap that makes this subtle is that **an ancestor-cut leaves no trace in the finished tree.** When
the guard cuts a branch it produces no output, and within any surviving path there are never repeats,
so any after-the-fact test computed from the completed tree — intersecting a node's subtree
obligations against its ancestors — always comes back empty and *looks* safe even when a branch was
severed. Deriving eligibility purely from the output tree is therefore unsound: it would cache a node
whose subtree was silently cut and under-report on reuse.

The fix is small and is the one piece that cannot be deferred to the post-pass: **the cycle guard must
flag, during the walk, every node on the stack from the colliding ancestor down as "cut-tainted."**
That flag is the only fact the output tree cannot reconstruct. The post-pass then caches only
*untainted* interior nodes — a node whose subtree provably never depended on an ancestor, so its
stored form equals what an empty-prefix `resolve_leaves` of that node would produce. Cyclic and
near-cyclic regions simply go uncached, which is correct and, because cycles are rare, cheap.

### Reuse only at seed boundaries, and re-rooting the fragment

Two rules make reuse sound and faithful. First, **reuse is restricted to seed boundaries** — a new
diagnostic's anchored seed, which has an empty prefix by construction — so there are no ancestors to
conflict with an untainted cached subtree. (Consulting the interior cache *mid-walk* would reintroduce
the ancestor-set question and is out of scope; Stage 2 is a cross-diagnostic cache keyed at seeds,
sharing storage with Stage 1.) Second, the cached value must be a **node-rooted `Resolved`** — its
header names that node as the subject and its trees start there, i.e. exactly the `Resolved` an
empty-prefix walk of the node yields, not a raw slice of the parent tree whose labels are relative to
the original root. Producing it is cheap: re-run only the *rendering* half over the node's sub-paths —
`label_for`, `elide_repeated_generics`, and the `Resolved` assembly, all compiler-free — while
skipping the expensive impl-selection-and-`holds` descent that Stage 2 exists to avoid. This means the
post-pass must retain each eligible node's sub-paths (the predicate chains below it), not only the
rendered strings, so it can re-root them.

## The cache key

The key for both stages must make different seeds provably unable to collide, because a wrong result
now comes from a key collision rather than from the walk. Because the resolver walks `Ty<'tcx>` today,
the key to build now is **a stable fingerprint of the region-erased obligation**. rustc's own query and
incremental caches key this way: `Ty`, `GenericArgs`, and `TraitRef` implement `HashStable`, and
`tcx.with_stable_hashing_context(|hcx| …)` yields a 128-bit `Fingerprint` that is `Copy` and
lifetime-free, so it can live on the emitter/context across diagnostics. At 128 bits the collision
probability is negligible to the standard rustc trusts for its own memoization. A raw
`(DefId, GenericArgsRef<'tcx>)` key does **not** work: `DefId` is lifetime-free but
`GenericArgsRef<'tcx>` is an interned `'tcx` reference that cannot be stored past its `TyCtxt`.

The one case where the key would change is if the deferred
[resolve-context](resolve-context.md#deferred-the-cgp-component-abstraction) work later moves the walk
onto an owned type model: the seed would then be owned data and structural equality could key it
directly, with no fingerprint. That is a possible future, not a decision to make now — build the
fingerprint key, and revisit only if that refactor lands.

Either way the placeholder caveat from Stage 1 holds: a call-site seed's placeholder identities are
part of the obligation and part of the key, so those seeds do not cross-hit — a coverage ceiling, not
a soundness gap.

## Where the caches live

Both stages share one store, and it lives on the [resolve context](resolve-context.md) — the
per-compilation struct that also holds the compiler-query access and the config constants. Interior
mutability (a `RefCell` around the map) matches how `DedupLedger` is already carried on `CgpEmitter`,
and the store is created once per driver invocation over one crate, with no cross-session incremental
database that could change underneath it. Until the resolve context exists, the store may sit directly
on `CgpEmitter` beside `dedup`; folding it into the context is part of that document's refactor.

## Sequencing and relation to diagnostic buffering

Build Stage 1 first, in full, and only add Stage 2 if a profile shows sub-obligation re-walks
dominating the residual after Stage 1 is in. Stage 1 is simple, sound, and output-preserving; Stage 2
adds the cut-taint bit and the re-rooting machinery for a second-order win. Instrument `resolve_leaves`
call counts, distinct-seed counts, and would-be hit rates against a real error-heavy project before
committing to Stage 2 — the tool runs only on failing builds, so the bar for the extra machinery is
whether the residual is real.

Both stages are complementary to the diagnostic-buffering work the
[usability issues](../issues/usability.md) anticipate, not in competition with it. Buffering would
decide *what* to emit — coalescing even different-consumer-same-cause blocks into one listing — while
a memoized `resolve_leaves` is the "resolve each unique seed exactly once" primitive a buffered
emitter would want underneath it. So this cache is a building block for that later work, not throwaway
if it lands.

## Comparison with Clippy

Clippy has no analog to this cache, because it has no analog to the work being cached. Clippy's late
passes run only on code that type-checks (the same `after_analysis` gate that forces cargo-cgp's
resolver into the emitter), so Clippy never re-runs the trait solver from inside diagnostic emission
and never resolves the same failure at many sites. Where Clippy relies on the compiler's own query
memoization for repeated `TyCtxt` lookups, cargo-cgp inherits that same memoization for its Class-A
schema queries (see [The resolve context](resolve-context.md)) — this cache adds the layer rustc does
*not* provide: memoizing the resolver's own composite walk, whose result is the rustc-free `Resolved`
tree. There is no Clippy code to follow here; the design is particular to reshaping errors rather than
adding them.

## Tests (planned)

The tests below do not exist yet; they are the coverage the implementation should add.

- **Output-preservation regression guard** — the existing `tests/ui/acceptable/` snapshot suite must
  re-bless to nothing after either stage lands. This is the primary correctness check: the caches are
  pure memoization, so any snapshot change is a bug.
- **Determinism** — a unit test (or a repeated no-bless UI run) confirming `resolve_leaves` yields the
  same `Resolved` for the same seed, which is the purity the cache assumes; the resolver's placeholder
  canonicalization is what this pins.
- **Stage 1 hit accounting** — an instrumented run over an error-heavy fixture (the money-transfer
  shape) asserting that the eighteen-tree case computes the walk once per distinct seed, not once per
  site.
- **Stage 2 soundness under a cut** — a fixture whose wiring makes the cycle guard cut a branch on one
  path and not another for the same interior node, asserting the cut-tainted node is not cached and the
  later resolution still reports the full cause (guarding the invisible-cut trap directly).

## Source (existing and planned)

Existing modules the cache attaches to:

- [`crates/cargo-cgp-driver/src/resolve/walk/leaves.rs`](../../crates/cargo-cgp-driver/src/resolve/walk/leaves.rs)
  — `resolve_leaves` (Stage 1's memo boundary) and `collect_leaf_paths` (whose cycle guard gains the
  cut-taint flag for Stage 2).
- [`crates/cargo-cgp-driver/src/emitter/cgp_emitter.rs`](../../crates/cargo-cgp-driver/src/emitter/cgp_emitter.rs)
  — `try_resolve` / `transform_resolved`, and the `DedupLedger` the cache composes with; the interim
  home of the cache store before the resolve context exists.
- [`crates/cargo-cgp-error-processing/src/diagnosis/resolved.rs`](../../crates/cargo-cgp-error-processing/src/diagnosis/resolved.rs)
  — the owned `Resolved` value the cache stores.

Planned additions:

- A cache store (both stages, one map) on the [resolve context](resolve-context.md), keyed per the
  key decision above, with the re-rooting helper for Stage 2's node-rooted fragments.
- The cut-taint annotation threaded through `collect_leaf_paths` and consumed by the Stage 2 post-pass.
