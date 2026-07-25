# Dependency-graph rendering

The dependency tree beneath a `root cause:` note is built by folding the resolver's per-cause paths
into one rustc-free **dependency graph** and rendering it `cargo tree`-style. The resolver emits, for
each way it reaches a root cause, a flat path of structured nodes; error-processing merges every node
that several paths reach in common into a directed acyclic graph, then renders that graph with a
`(*)` marker on any subtree already drawn elsewhere. All the merging and rendering is a pure function
over owned data, so every shape is unit-tested without a compiler in the loop.

## Why a graph rather than a chain per cause

A single failure's root causes do not form independent chains, so the note cannot be a stack of
linear spines — it has to be a graph, because real wiring produces three shapes a per-cause spine
cannot represent. Each shape is a way two root→leaf paths relate to one another beyond simply sharing
a prefix, and the graph is what lets the note show each correctly.

The first is a **shared dependency** — a diamond. When two providers both depend on one capability
and that capability is what fails, their two paths share a *suffix* (`… → C → missing`), not a
prefix. The note should show the shared capability and its subtree once; a spine model that keys only
on the root would repeat the whole subtree under each parent.

The second is **independent consumers converging on one leaf**. Two unrelated components can both
read the same missing field, so their paths meet only at the terminal leaf. The note names that one
cause once in its heading, yet must still show both consumers' chains, because both are real and both
need the same fix.

The third is **subsumption** — one consumer's chain running *through* another. When `CanCalculateDensity`
depends on `CanCalculateArea`, the density chain contains the area chain as a sub-path. The note
should lead with the deeper chain and show the area consumer in its rightful place inside it, not as a
second top-level entry.

A graph whose nodes have structural identity handles all three: a node several paths reach in common
is one node with several parents or children, so a shared dependency is stored once, a converging leaf
is one node, and a subsumed consumer is a node that is both a path head and another node's child. The
rest of this document is how that graph is built, ruled, and rendered.

## Structured nodes

A dependency node is structured data, not a pre-rendered string, so the graph can compare nodes for
identity. The interior hops are a
[`DepNode`](../../crates/cargo-cgp-error-processing/src/diagnosis/node.rs) enum with one variant per
`CGP-E1xx` chain-hop class, each carrying the names that class renders from:

```rust
pub enum DepNode {
    Consumer { trait_ref: String, context: String },              // CGP-E101
    Provider { trait_ref: String, context: String, provider: String }, // CGP-E102
    Redirect { path: String, context: String, key: String },      // CGP-E104
    Trait { trait_ref: String, self_ty: String },                 // CGP-E105
}
```

There is no `CGP-E103` hop: a `HasField` obligation is always a terminal root cause in the walk, never
a mid-chain hop, so the code that would have carried it is retired (see
[error-code.md](../error-code.md)). The terminal root cause is the existing
[`Leaf`](../../crates/cargo-cgp-error-processing/src/diagnosis/leaf.rs), unchanged — it already carries
the field, wiring, dispatch, and bound classifications the leads and codes are worded from. A graph
node is therefore either a hop or a leaf:

```rust
pub enum ChainNode {
    Hop(DepNode),
    Leaf(Leaf),
}
```

Each variant renders to exactly the label its `CGP-E1xx` template dictates — `` consumer trait impl `…` for context `…` `` and the rest — with the trait reference stored *with* its generic arguments
(`CanCalculateArea<f64>`), since rendering them is a rustc-free concern. Node identity is
whole-node structural equality (`Eq`/`Hash` derived on the enum), which is faithful even where the
rendered label is not: the `Redirect` node holds the dispatched `key` (`Left` versus `Right`) though it
renders only the route (`@ValueBuilderComponent`), so two lookups along one route for different keys
stay distinct nodes rather than merging into a false diamond.

## A cause is a leaf and the paths that reach it

A [`Cause`](../../crates/cargo-cgp-error-processing/src/diagnosis/resolved.rs) is the leaf it names for
the note heading plus every root→leaf path that reaches it:

```rust
pub struct Cause {
    pub leaf: Leaf,
    pub paths: Vec<Vec<ChainNode>>,
}
```

A leaf reached one way has a single path — the common case; a leaf reached through a shared capability
several providers depend on has several, one per parent. Holding several paths on one cause rather
than one cause per path is deliberate: it preserves the **one cause per distinct leaf** invariant that
the rest of the pipeline relies on. The de-duplication ledger's `cause_signature` and
`cause_keys`, the consumer coalescing, and `derive_help_messages` all read `resolved.causes`
expecting each leaf once; only the *rendering* consumes the extra paths.

Every place that *builds* a cause list has to establish that invariant, and there are four: the
walk's own `compute_leaves`, the impl-site and wrapper-chain anchors (each prepending a hop to the
paths of the sub-walks it collects), the by-component use-site anchor (unioning a walk per wired
component), and the emitter's coalesced block (unioning a resolution per affected consumer). All four
go through one function,
[`merge_causes_by_leaf`](../../crates/cargo-cgp-error-processing/src/diagnosis/merge.rs), which folds
causes naming the same leaf into one carrying every path.

Getting it wrong fails in two directions, and both were live before the function existed. **Merging
nothing** states one mistake once per contributor: three consumers failing on one underived field
produced `` accessor trait `HasField` is not implemented for the fields `name`, `name`, and `name` ``
— the underived-field coalescing below, faithfully reporting three causes it should never have been
given. **De-duplicating by leaf but dropping the duplicate's paths** loses a chain instead: a
use-site failure across several wired components that share a cause kept only the first component's
route, so the header named a consumer whose chain appeared nowhere in the note. Keeping the leaf
*and* accumulating the paths is what avoids both.

Coalescing several present-but-underived fields on one struct is the one case where a cause's heading
leaf differs from its paths' terminal leaves.
[`coalesce_underived_fields`](../../crates/cargo-cgp-error-processing/src/diagnosis/coalesce.rs) merges
those causes into one whose heading `leaf` is a `Leaf::UnderivedFields` naming every field, while its
`paths` keep every field's own path — each still terminating at that field's individual
`Leaf::Field`. The heading then reads as one fix (`` the fields `height` and `width` ``) while the
graph still branches to each per-field leaf.

## Building the graph

[`DependencyGraph::from_paths`](../../crates/cargo-cgp-error-processing/src/diagnosis/graph.rs) folds a
set of paths into a DAG. It stores every distinct node once, keyed by structural identity, and records
each node's children in first-seen order, whether the node is ever some node's child, and the head
(first node) of each input path. A node equal to one already seen in *another* path reuses its id, so a
node several paths reach in common becomes one node with several parents and/or children.

Node identity is **cross-path only**. A label that repeats *within a single path* is kept a distinct
node, because a linear descent can pass through two hops that render identically yet mean different
things — a recursive `RedirectLookup` resolving `Outer` then `Inner`, whose rendered label omits the
key — and merging those would fold the spine into a false cycle. `from_paths` enforces this by tracking
the ids already placed on the current path and registering only a label's first occurrence in the
lookup index: a within-path repeat gets a fresh, unregistered node, while a later path still finds the
canonical one.

## Roots and subsumption

A path head is a **top-level root only if it is not also some other node's child.** This one rule
gives subsumption for free. When one consumer's chain passes through another —
`CanCalculateDensity → DensityCalculator → CanCalculateArea → …` — the head `CanCalculateArea` from the
shorter path is also a child inside the longer one, so it is not rendered as a second top-level entry;
it still appears, once, in its place inside the deeper chain. When neither consumer subsumes the other,
both heads stay roots and both render. If every head is a child (a pathological all-cyclic input),
`roots` falls back to every head, so rendering never yields nothing.

## Rendering

The renderer walks the graph depth-first from each root in first-seen order, expanding a node's
children the first time it is reached and marking any later reach with a `(*)` suffix — the convention
`cargo tree` uses for a dependency already shown elsewhere. One `expanded` set spans all the roots, so
a node expanded under one root is `(*)`-referenced under the next, and because each node is expanded at
most once the walk terminates even if the data ever contains a cycle. A **leaf** carries no subtree to
hide, so a leaf reached by several paths is drawn in full each time rather than marked — the root cause
reads the same wherever a chain bottoms out on it. The render produces one
[`DependencyTree`](../../crates/cargo-cgp-error-processing/src/tree.rs) per root, stacked into the note.

**Every CGP construct is rendered in full.** A hop whose trait reference exactly repeated its
parent's once printed its generic list as `<…>`, which shortened a dispatch chain that restates a
program-sized `Code` type at every step. That is no longer done, deliberately: the elision hid the
very type the reader is tracing, and left them unable to tell a genuine repeat from a hop whose
parameters differ without re-deriving it. A chain step now always names its trait and its parameters
as written. The verbosity that buys back is real on a DSL-sized program and is accepted — the chain
is there to be read precisely, and a reader who wants it shorter is better served by the
[cross-block elision](#eliding-across-blocks) below, which drops whole subtrees a previous block
already drew rather than obscuring individual types.

## Worked shapes

The following shapes cover the cases the graph exists to render; each is a real UI fixture, and the
leaf it bottoms out on is the root cause.

A **linear spine** — one consumer, one chain, one leaf — is the base case
([`base_area_1`](../../tests/ui/acceptable/fields/base_area_1.rs), a `Rectangle` missing its `height`
field):

```text
[CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
└─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
  └─ [CGP-E105] trait impl `HasRectangleFields` for `Rectangle`
    └─ [CGP-E106] missing field `height` on `Rectangle`
```

A **shared-root branch** — one consumer whose provider has two unmet dependencies — branches at the
divergence point ([`parallel_branches`](../../tests/ui/acceptable/fields/parallel_branches.rs), a
`Person` missing both name fields):

```text
[CGP-E101] consumer trait impl `CanGreet` for context `Person`
└─ [CGP-E102] provider trait impl `Greeter` with context `Person` for provider `GreetFullName`
  ├─ [CGP-E105] trait impl `HasFirstName` for `Person`
  │ └─ [CGP-E106] missing field `first_name` on `Person`
  └─ [CGP-E105] trait impl `HasLastName` for `Person`
    └─ [CGP-E106] missing field `last_name` on `Person`
```

A **subsuming cascade** — `CanCalculateDensity` depends on `CanCalculateArea`, both checked
([`density_3`](../../tests/ui/acceptable/duplication/density_3.rs)) — renders as the single deeper
chain, because the area consumer's head is a descendant of the density chain and so is not a second
root:

```text
[CGP-E101] consumer trait impl `CanCalculateDensity` for context `Rectangle`
└─ [CGP-E102] provider trait impl `DensityCalculator` with context `Rectangle` for provider `DensityFromMassField`
  └─ [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
    └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
      └─ [CGP-E105] trait impl `HasRectangleFields` for `Rectangle`
        └─ [CGP-E106] missing field `height` on `Rectangle`
```

**Two independent consumers converging on one leaf**
([`parallel_consumers`](../../tests/ui/acceptable/duplication/parallel_consumers.rs), two unrelated
components each reading the missing `height`) keeps both roots, and the shared leaf is drawn under each
— a leaf hides no subtree, so no `(*)`:

```text
[CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
└─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
  └─ [CGP-E105] trait impl `HasRectangleFields` for `Rectangle`
    └─ [CGP-E106] missing field `height` on `Rectangle`
[CGP-E101] consumer trait impl `CanReportHeight` for context `Rectangle`
└─ [CGP-E102] provider trait impl `HeightReporter` with context `Rectangle` for provider `ReportHeight`
  └─ [CGP-E105] trait impl `HasHeight` for `Rectangle`
    └─ [CGP-E106] missing field `height` on `Rectangle`
```

A **diamond** — one shared capability reached through two branches
([`diamond_shared_capability`](../../tests/ui/acceptable/resolution/diamond_shared_capability.rs),
where `CanTop` depends on both `CanLeft` and `CanRight`, which both depend on `CanShared`, whose
provider needs a `name` field the context lacks) — expands the shared `CanShared` subtree under the
first branch and `(*)`-references it under the second, so the root cause is shown once:

```text
[CGP-E101] consumer trait impl `CanTop` for context `App`
└─ [CGP-E102] provider trait impl `TopProvider` with context `App` for provider `ProvideTop`
  ├─ [CGP-E101] consumer trait impl `CanLeft` for context `App`
  │ └─ [CGP-E102] provider trait impl `LeftProvider` with context `App` for provider `ProvideLeft`
  │   └─ [CGP-E101] consumer trait impl `CanShared` for context `App`
  │     └─ [CGP-E102] provider trait impl `SharedProvider` with context `App` for provider `ProvideShared`
  │       └─ [CGP-E105] trait impl `HasName` for `App`
  │         └─ [CGP-E106] missing field `name` on `App`
  └─ [CGP-E101] consumer trait impl `CanRight` for context `App`
    └─ [CGP-E102] provider trait impl `RightProvider` with context `App` for provider `ProvideRight`
      └─ [CGP-E101] consumer trait impl `CanShared` for context `App` (*)
```

**Distinct-key redirects** show why the redirect key is part of a node's identity
([`redirect_distinct_keys`](../../tests/ui/acceptable/wiring/missing-wiring/redirect_distinct_keys.rs),
two dependencies dispatched along one `open` route for two unwired keys). Both redirect hops render the
same label, but they hold different keys (`Left`, `Right`), so the graph keeps them distinct and the
tree branches to each key's own missing-wiring leaf rather than collapsing both under one shared
redirect:

```text
[CGP-E101] consumer trait impl `CanAssemble` for context `App`
└─ [CGP-E102] provider trait impl `Assembler` with context `App` for provider `AssembleParts`
  ├─ [CGP-E101] consumer trait impl `CanBuildValue<Left>` for context `App`
  │ └─ [CGP-E104] redirect lookup to `@ValueBuilderComponent` in `App`
  │   └─ [CGP-E107] context `App` does not contain any delegate entry for `@ValueBuilderComponent.Left`
  └─ [CGP-E101] consumer trait impl `CanBuildValue<Right>` for context `App`
    └─ [CGP-E104] redirect lookup to `@ValueBuilderComponent` in `App`
      └─ [CGP-E107] context `App` does not contain any delegate entry for `@ValueBuilderComponent.Right`
```

## Eliding across blocks

The `(*)` convention reaches past one note: one `seen` set threaded through a compilation's blocks in
emission order lets a later block truncate at a subtree an earlier one already drew. This exists
because CGP wiring is lazy, so one mistake surfaces in several diagnostics that legitimately do *not*
de-duplicate — a hand-written wrapper trait is a distinct trait from the consumer it reduces to, so it
keeps its own block — and their chains can share everything below their own first few hops. In the
money-transfer example the second block's 29 nodes were 25 of the first block's plus a four-node
routing prefix; eliding the shared remainder takes it from 38 rendered lines to six.

[`render_seen`](../../crates/cargo-cgp-error-processing/src/diagnosis/graph.rs) is `render` against a
caller-owned `seen`, and three rules keep it honest.

**An elided branch still bottoms out at the root cause.** A chain exists to lead the reader from what
they wrote down to the mistake, so stopping one step short of it is the one thing it may never do —
and a cross-block `(*)` points into *another* block, which a reader may not have to hand. So a branch
elided across renders keeps the marker on the hop and appends the distinct leaves reachable beneath it
(`leaves_below`): the intervening hops are elided, the terminus is not. A branch elided *within* one
render needs no such terminator, and keeps the bare `(*)` it always had, because the subtree it points
at — root cause included — is right above it in the same note.

**A render consults only what *earlier* renders drew.** The nodes it draws itself are collected apart
and folded in at the end, because `seen` is keyed by node value while a label repeating *within* one
path is deliberately a distinct node — a set the current render were also filling would mark the second
occurrence `(*)` and fold a linear descent into a false cycle. Within a render, only the id-keyed
`expanded` elides.

**A block that would say nothing new drops its chain.** `fully_elided_by` reports every top-level root
already drawn, and the note then keeps its `root cause:` lead alone rather than heading a lone `(*)`
with the promise of a chain. The cause is still named — by the lead instead of by a terminus.

An elided block stays actionable read on its own — its header, its fix `help`, and its `root cause:`
lead all still name the cause, so what is elided is chain *detail*, never what failed or how to fix
it. That is what makes the elision safe for a consumer that sees one diagnostic at a time, such as an
editor reading the JSON output.

## The note over the graph

A `root cause:` note is built from one graph covering every cause in the failure, not one note per
cause. [`cause_notes`](../../crates/cargo-cgp-error-processing/src/diagnosis/wording/note.rs) collects
every path of every cause, folds them into a single `DependencyGraph`, renders it beneath the
`this is required through the dependency chain:` heading, and words the heading above it. The heading
lists the distinct leaves once, in first-seen order: a singular `root cause: [code] lead` when every
path bottoms out on the same leaf, or a `root causes:` list when they differ, each entry carrying its
own code so a reader sees every cause at a glance. The singular lead is dropped when the main message
already states that very leaf — a mismatch header naming the type, or a kept rustc header restating
the ordinary bound — leaving the graph alone under its heading.

The emitter's coalesced block builds the same graph-backed note rather than assembling a tree of its
own. When several consumer failures share a cause and coalesce into one `[CGP-E001]` block (see
[The driver](driver.md)), that block's note folds *every* member's causes
into one graph, so a consumer whose chain runs through another collapses into it while independent
chains to the shared cause render side by side — and no member's chain is dropped. Its causes pass
through `merge_causes_by_leaf` first, so the heading names each distinct cause once however many
members reached it, while the merged cause still carries every member's path into the graph.

## How the resolver feeds the graph

The resolver produces the structured paths and nothing tree-shaped; every merge decision and every
glyph is the graph's. Its per-stage source lives under the driver's
[`resolve`](../../crates/cargo-cgp-driver/src/resolve) module, and the change from string labels to
structured nodes touches only what each stage *emits*, not how it reads the compiler.

`label_for` in [`resolve/label`](../../crates/cargo-cgp-driver/src/resolve/label) reads each dependency
hop off its obligation and returns a `DepNode` — a `Consumer`, `Provider`, `Redirect`, or `Trait`
variant, with the redirect variant carrying the dispatched key as identity. The walk in
[`resolve/walk`](../../crates/cargo-cgp-driver/src/resolve/walk) builds each cause's path bottom-up: a
terminal leaf produces a one-element path holding just the `ChainNode::Leaf`, a field-type-mismatch
produces the hop above its leaf, and `resolve_node` prepends its own hop to every sub-path it collects
from its children, so a stored sub-path is rooted at the node that produced it. `compute_leaves` then
groups those sub-paths by leaf into one `Cause` each — a leaf reached by several paths keeps each path,
so the diamond survives to the renderer instead of every path but the first being dropped.

The anchors that wrap a walk result in an enclosing trait prepend their hops the same way. The
impl-site and foreign-wrapper anchors (`impl_site.rs`, `wrapper_chain.rs`) insert the wrapper trait's
`DepNode` at the front of each recovered path and group by leaf, rather than nesting one
`DependencyTree` inside another — flat prepending is both simpler and impossible to get structurally
wrong. The per-node cache in [`cache.rs`](../../crates/cargo-cgp-driver/src/resolve/cache.rs) stores
these node-rooted paths (`SubCause { leaf, path }`) as its owned, rustc-free value, so a subtree resolved
once is reused verbatim wherever the node recurs (see
[Cached dependency resolution](cached-dependency-resolution.md)).

## Boundaries and known limitations

A few edges of the model are worth recording. Node identity is whole-node structural equality, which
is faithful in both directions: within a single path a repeated label stays distinct, so a recursive
descent never folds into a false cycle, and across paths the fields a node carries capture what
distinguishes it even where the rendered label does not — the `Redirect` key being the load-bearing
case. The `(*)` convention is borrowed from `cargo tree` and is pinned in the fixtures. This document covers rendering only; the resolver's walk, anchors, and cache are
described in [Typed root-cause resolution](typed-root-cause-resolution.md) and changed by this model
only in the representation they emit.

**A wide convergence repeats `(*)` once per parent, and that is not noise to collapse.** A capability
many providers share — an abstract type especially, since one binding serves the whole context — is a
node with many parents, so its reference recurs once under each. It reads at a glance like repetition
worth folding, and it is not: each `(*)` is the *terminator of a distinct branch* naming a distinct
consumer, so dropping the repeats would leave those branches dangling with no explanation of why they
are listed. Reproducing a six-way convergence makes this plain — the weight is the six two-line
branches, which are genuine information, not the six markers that end them. The length is inherent to
wiring in which six capabilities really do need one shared thing. What *is* worth collapsing is a
subtree drawn in another block, which is [the cross-block elision](#eliding-across-blocks) above.

## Comparison with Clippy

Clippy has no analog to this rendering, because it has no analog to the work behind it. Clippy adds new
lints and lets the compiler's standard emitter render them; it never reconstructs a failed obligation's
dependency chain, so it has nothing to merge into a graph or lay out as a tree. The whole graph is
particular to reshaping an existing diagnostic rather than emitting a new one, and it lives in the
rustc-free `cargo-cgp-error-processing` crate precisely so it is testable the way Clippy's own logic is
— as a pure function over owned data, with no compiler in the loop.

## Tests

Because the graph is a pure function over structured data, every shape is a unit test with no compiler,
and the end-to-end behavior is pinned by the UI suite.

- [`crates/cargo-cgp-error-processing/tests/graph.rs`](../../crates/cargo-cgp-error-processing/tests/graph.rs)
  — the build-and-render as `insta` inline snapshots: a linear spine, a shared-prefix branch, a
  subsuming cascade, converging independent roots on one leaf, a diamond, a super-root, a within-path
  label repeat kept linear, cross-path redirects distinct by key versus merged by key, a repeated
  trait and a differing one both rendered in full, a cyclic input terminating with a `(*)`
  mark, and an empty path set rendering empty. The cross-block elision has four of its own: a second
  graph truncating at what the first drew while keeping its own prefix *and still ending at the root
  cause*, the within-render reference staying bare by contrast, a wholly-redundant graph reporting
  itself `fully_elided_by`, and a leaf-only graph never doing so (a leaf hides no subtree).
- [`crates/cargo-cgp-error-processing/tests/diagnosis.rs`](../../crates/cargo-cgp-error-processing/tests/diagnosis.rs)
  — the note assembly over the graph: the singular `root cause:` lead, its drop when the header states
  that leaf (either mismatch class, or a kept-header bound) and its retention under a header that does
  not, the `root causes:` list for distinct leaves, the shared-prefix merge
  into one branching note, and independent-root causes folding into one note with stacked chains.
- [`crates/cargo-cgp-error-processing/tests/coalesce.rs`](../../crates/cargo-cgp-error-processing/tests/coalesce.rs)
  — `coalesce_underived_fields` keeping every field's path while merging the heading into one
  `UnderivedFields` cause.
- [`crates/cargo-cgp-error-processing/tests/merge.rs`](../../crates/cargo-cgp-error-processing/tests/merge.rs)
  — `merge_causes_by_leaf`: one leaf reached by three consumers folding to one cause holding all three
  paths, the underived-field lead that repetition would otherwise produce, distinct leaves staying
  apart, an exact repeat of a path dropped, and a well-formed list left untouched.
- The UI fixtures the worked shapes above cite — `base_area_1`, `parallel_branches`, `density_3`,
  `parallel_consumers`, `diamond_shared_capability`, and `redirect_distinct_keys` — plus
  [`foreign_getter_missing_wiring`](../../tests/ui/acceptable/resolution/foreign_getter_missing_wiring.rs),
  a diamond converging on one missing wiring, exercise the graph end to end through the real compiler.

## Source

- [`crates/cargo-cgp-error-processing/src/diagnosis/node.rs`](../../crates/cargo-cgp-error-processing/src/diagnosis/node.rs)
  — the `DepNode` and `ChainNode` structured nodes and their rendering (the `CGP-E1xx` label
  templates).
- [`crates/cargo-cgp-error-processing/src/diagnosis/graph.rs`](../../crates/cargo-cgp-error-processing/src/diagnosis/graph.rs)
  — `DependencyGraph`, `from_paths` (the cross-path-only merge), the root rule, the `(*)`-dedup
  renderer (every construct rendered in full), and `render_seen`/`leaves_below`/`fully_elided_by` (the
  cross-block elision and the terminus it keeps).
- [`crates/cargo-cgp-error-processing/src/diagnosis/resolved.rs`](../../crates/cargo-cgp-error-processing/src/diagnosis/resolved.rs)
  — `Cause { leaf, paths }`, one cause per leaf holding every path that reaches it.
- [`crates/cargo-cgp-error-processing/src/diagnosis/wording/note.rs`](../../crates/cargo-cgp-error-processing/src/diagnosis/wording/note.rs)
  — `cause_notes`, folding every cause's paths into one graph and wording the heading over it.
- [`crates/cargo-cgp-error-processing/src/diagnosis/coalesce.rs`](../../crates/cargo-cgp-error-processing/src/diagnosis/coalesce.rs)
  — `coalesce_underived_fields`, merging underived fields into one heading cause while keeping their
  paths.
- [`crates/cargo-cgp-error-processing/src/diagnosis/merge.rs`](../../crates/cargo-cgp-error-processing/src/diagnosis/merge.rs)
  — `merge_causes_by_leaf`, the one-cause-per-distinct-leaf invariant, applied by all four cause-list
  builders: the walk's `compute_leaves`, the `impl_site` and `wrapper_chain` anchors, the
  by-component `use_site` anchor, and the emitter's coalesced block.
- [`crates/cargo-cgp-error-processing/src/tree.rs`](../../crates/cargo-cgp-error-processing/src/tree.rs)
  — the `DependencyTree` type and its `termtree`-backed renderer, the target the graph expands into.
- [`crates/cargo-cgp-driver/src/resolve/label`](../../crates/cargo-cgp-driver/src/resolve/label),
  [`walk`](../../crates/cargo-cgp-driver/src/resolve/walk), and the `impl_site` / `wrapper_chain`
  anchors — emit `DepNode` hop-paths and group by leaf; the cache in
  [`cache.rs`](../../crates/cargo-cgp-driver/src/resolve/cache.rs) stores them.
- [`crates/cargo-cgp-driver/src/emitter/cgp_emitter.rs`](../../crates/cargo-cgp-driver/src/emitter/cgp_emitter.rs)
  — the flush that renders every resolution's note against one shared `seen`, and the coalesced block
  whose note is built from the same graph.

## Further reading

- [Typed root-cause resolution](typed-root-cause-resolution.md) — the resolver whose walk fills the
  paths this document renders, and the anchors that prepend to them.
- [Error processing](error-processing.md) — the rustc-free crate this rendering lives in, alongside the
  wording, plan, and de-duplication it feeds.
- [The driver](driver.md) — the emitter that applies the plan and builds the coalesced block's note
  from the same graph.
