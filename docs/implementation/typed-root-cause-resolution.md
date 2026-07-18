# Typed root-cause resolution

The driver's most valuable transformation turns a CGP check-failure diagnostic into a compact,
root-cause-first error by *asking the compiler* what really failed rather than parsing rustc's text.
When a context is wired wrong, the compiler reports the failure against generated types the
programmer never wrote, and the one fact that matters — a missing field, an unwired component — is
buried under `IsProviderFor`/`CanUseComponent` scaffolding or dropped entirely. The resolver re-runs
the failing obligation through the trait solver, walks the wiring down to the actual root cause, and
re-renders the whole diagnostic as a `cargo tree`-style dependency chain with a single coded
headline.

The resolver reads the failure from the **real consumer and provider trait obligations**, never from
the `CanUseComponent`/`IsProviderFor` scaffolding. Those two traits exist only so that plain rustc
can surface a wiring failure at all (see [check traits](../../../cgp/docs/concepts/check-traits.md)):
`IsProviderFor` is generated to carry a *copy* of a provider's `where` bounds precisely so the
compiler names the missing one. cargo-cgp does not need that copy — it re-runs the trait solver on
the real provider impl, whose own `where` clause holds the same bounds — so it treats `IsProviderFor`
and `CanUseComponent` as plumbing to resolve *around*, reading the actual traits instead. This is a
deliberate constraint, not an accident of implementation: cargo-cgp aims to make `IsProviderFor`
*removable*, so its dependency resolution must not lean on it. (The text-rewrite fallback for
diagnostics the resolver declines still recognizes `IsProviderFor`/`CanUseComponent` in rustc's
rendered output; that is a separate concern, covered in [The driver](driver.md).)

This is the second, deeper transformation the driver's emitter performs, and it builds on the first.
[Naming the traits behind a component marker](driver.md#naming-the-traits-behind-a-component-marker)
edits a diagnostic in place, renaming its wording; the resolver instead reconstructs the failure from
compiler state and replaces the diagnostic wholesale. It realizes the compiler-state enrichment that
[The driver](driver.md) and [The error pipeline](error-pipeline.md) anticipated, and everything
downstream of its rustc-free [`Resolved`](../../crates/cargo-cgp-error-processing/src/diagnosis/resolved.rs)
model — the wording, the tree — is unit-tested without a compiler.

## A worked example

The clearest way in is one failure end to end, from the [area-calculation
example](../../../cgp/docs/examples/area-calculation.md). A `Rectangle` computes its area through a
wired `RectangleArea` provider that reads the rectangle's fields, but the struct is missing one of
them:

```rust
#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_auto_getter]
pub trait HasRectangleFields {
    fn width(&self) -> f64;
    fn height(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator
where
    Self: HasRectangleFields,
{
    fn area(&self) -> f64 {
        self.width() * self.height()
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    // the `height` field is missing
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}
```

`RectangleArea` reads `width` and `height` through the `HasRectangleFields` getter, whose
`#[cgp_auto_getter]` blanket impl requires the context to have both fields. `Rectangle` derives
`HasField` but declares only `width`, so the wiring cannot be satisfied — the **root cause is the
absent `height` field** — and `check_components!` fails. Left to rustc, the failure reads as an unmet
`HasField<Symbol!("height")>` bound (often with the field name itself compressed to an unreadable
`Symbol<6, Chars<'h', …>>` spine), routed through `IsProviderFor` and `CanUseComponent`. The resolver
replaces all of that with:

```text
error[E0277]: [CGP-E001] the consumer trait `CanCalculateArea` is not implemented for context `Rectangle`
  --> src/main.rs:61:9
   |
61 |         AreaCalculatorComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: root cause: [CGP-E106] missing field `height` on `Rectangle`
           this is required through the dependency chain:
               [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
               └── [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
                   └── [CGP-E105] trait impl `HasRectangleFields` for `Rectangle`
                       └── [CGP-E106] missing field `height` on `Rectangle`
```

Every element of that output is reconstructed from the compiler, not read from rustc's message: the
headline names the *consumer trait* the reader called (`CanCalculateArea`) and the context that
cannot implement it; the `root cause:` note names the actual mistake in one sentence; and the tree
shows the transitive path from the check entry down to the missing field, with each node named
straight off the trait it stands for. Read the chain as the real obligation chain the walk descended:
`Rectangle: CanCalculateArea` (the consumer) needs `RectangleArea: AreaCalculator<Rectangle>` (the
wired provider), which needs `Rectangle: HasRectangleFields` (the getter), which needs the `height`
field. No `IsProviderFor` node appears because the walk never went through one. The `[CGP-Exxx]` codes
are catalogued in [error-code.md](../error-code.md). The rest of this document explains how each piece
is produced.

## When the resolver engages, and when it declines

The resolver treats a diagnostic as a candidate whenever it plausibly stems from a CGP component
failure, then either traces it to a root cause or steps aside. Concretely, a diagnostic is a
candidate when it **names a CGP wiring or field trait** (`CanUseComponent`, `IsProviderFor`, or
`HasField` — matched in rustc's rendered text, the one place these still serve as a *signal* that the
diagnostic is CGP-related), or carries code **`E0271`**, **`E0277`**, or a **method-bounds `E0599`** —
the "the method `…` exists … but its trait bounds were not satisfied" shape. The breadth past the
wiring-worded cases is deliberate, because a failure that names no CGP construct can still be one a
CGP component caused: a hand-written `Send`-recovery wrapper's `async fn` fails with an `E0271`
opaque-future mismatch, a downstream bound needs a method the context cannot supply. The resolver
traces the dependency chain and treats the error as CGP-related exactly when a CGP component failure
sits in that chain; a candidate whose chain reaches no CGP cause **declines** and passes through to
the fallback text rewrite untouched.

The `E0599` arm is narrowed to the method-bounds shape for a reason beyond relevance: a
*resolution*-class `E0599` (`no variant named …`, `no associated item …`) is emitted while type
lowering is still mid-flight, and running the resolver's trait solver on it re-enters the diagnostic
context and aborts the compiler. Declining such an `E0599` before any solving is both crash-safe and
correct — the resolver has nothing to say about a name-resolution error — whereas `E0271`/`E0277` are
trait-solving failures reported after collection and do not hit the hazard. This is the
re-entrant-emission panic catalogued in
[rustc diagnostic internals](rustc-diagnostic-internals.md#re-entering-the-diagnostic-context-lock-was-already-held),
where the phase a diagnostic is emitted in is what decides whether the solver may safely run on it.

A candidate the resolver accepts is transformed in two independent halves — the coded headline and
the root-cause notes — described next. A candidate it declines keeps rustc's diagnostic, cleaned only
by the fallback [post-processing](error-processing.md).

## The two halves of a transformed diagnostic

### The coded headline

The main message is rewritten into a coded CGP class only when the resolution identifies one; the
Rust error code (`E0277`, `E0599`, `E0271`) is always kept. Four classes cover the cases:

- **`[CGP-E001]`, the consumer form**, for a context that cannot implement a consumer trait — a
  [check-trait failure](../../../cgp/docs/errors/checks/check-trait-failure.md). It names the consumer
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
- **`[CGP-E009]`, the plain-trait form**, for a hand-written wrapper trait that is *not* itself a CGP
  consumer (it has only a concrete impl, no consumer blanket) — `CanHandleApiSend`, say. It reads
  `the trait \`…\` is not implemented for \`…\``, distinguished from `[CGP-E001]` by the wrapper's
  fingerprint (see [the impl-site and wrapper-chain anchors](#anchoring-the-starting-obligation)).

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
           └── [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
               └── [CGP-E109] field `height` on `Rectangle` has type `i32`, but `f64` is required
```

Two cases keep rustc's own headline rather than a coded one. A main message that already states a
**genuine recovered leaf** — an ordinary bound such as `f64: Eq` the next-generation solver descended
to, which is itself the root cause — is left untouched, header and caret and all, because it already
names the cause. But a main message that states a *mid-chain symptom* — an ordinary bound that is not
one of the recovered leaves, such as a getter bound on a request whose real cause is a missing wiring
a level down — is replaced by the truer `[CGP-E001]` consumer header. And a main message that is
neither a trait bound nor a resolved class (an unrelated `E0308`) is always kept.

### The root-cause notes

Whatever happens to the headline, the sub-messages are always replaced. rustc's obligation-chain
notes, supplementary help, and structured suggestions are discarded, and each recovered root cause
becomes one `= note:` opening with a `root cause:` lead that names the leaf, followed by the
dependency chain rendered as a tree. The chain **repeats the root cause as its own terminal leaf**, so
it always bottoms out *at* the cause rather than one step before it, whatever the leaf's kind. Every
lead and every tree entry carries a [`CGP-E1xx`/`CGP-E2xx` code](../error-code.md) (the wording below
omits the prefix for brevity). Paths render as a bare `@app.GreeterComponent` — the `Path!(@…)` macro
form is reserved for the resugaring fallback — and module qualifiers are stripped throughout, so
`contexts::app::MockApp` reads as `MockApp`.

The `root cause:` lead is worded by *why* the leaf is unmet, and there are four leaf shapes:

- A **genuinely absent field** reads as `root cause: missing field \`height\` on \`Rectangle\`` — the
  worked example's leaf. There is no `context` qualifier, since `HasField` can land on any struct.
- A **present-but-underived field** — one the struct carries but that has no (or an incomplete)
  `#[derive(HasField)]` — reads as
  `root cause: accessor trait \`HasField\` with field \`name\` is not implemented for \`Person\``, and
  a separate `help` names the fix: `make sure that \`#[derive(HasField)]\` is used for \`Person\``.
  When the field is only reachable through a `Deref` target, the `help` points at that target instead
  ([`missing_has_field_derive`](../../tests/ui/acceptable/fields/missing_has_field_derive.rs),
  [`field_via_deref`](../../tests/ui/acceptable/fields/field_via_deref.rs)).
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
                 └── [CGP-E102] provider trait impl `FooProvider` with context `App` for provider `DoFooWithBar`
                     └── [CGP-E101] consumer trait impl `CanUseBar` for context `App`
                         └── [CGP-E107] context `App` does not contain any delegate entry for `BarProviderComponent`
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

A field-type mismatch is a fifth leaf, but its own `[CGP-E003]` headline already states it in full (as
the `E0271` example above shows), so its note drops the `root cause:` lead and carries the chain
alone. Any other leaf simply restates its bound —
`root cause: the trait bound \`f64: Eq\` is not satisfied`, module qualifiers stripped — except when
the kept headline already states that very bound, where the lead is likewise dropped.

## Why it runs in the emitter

The natural home for whole-crate typed analysis would be an `after_analysis` callback, where the
compiler hands the driver a `TyCtxt` directly — but that door is closed for exactly the crates that
matter. The `analysis` query raises a fatal error the moment type-checking reports any non-lint error
(`rustc_interface`'s `analysis` calls `has_errors_excluding_lint_errors().raise_fatal()`), and that
unwind happens *before* `after_analysis` runs. A crate with a CGP check failure has an error by
definition, so `after_analysis` never sees it — the same reason Clippy's late passes only run on code
that type-checks.

The one place that executes *while the error exists but before the fatal unwind* is the diagnostic
emitter, which the compiler calls as it emits each error during trait solving. A `TyCtxt` is in
thread-local scope there (the driver already relies on this for the trait-renaming rewrite), so the
resolver reaches the compiler through `rustc_middle::ty::tls` from inside `emit_diagnostic`. The
subtlety this design must be sound against is that it re-enters the trait solver *from within a
diagnostic being emitted mid-solve*. Building a fresh `InferCtxt` and `ObligationCtxt` and solving a
concrete obligation there works cleanly, and that re-entrancy is the load-bearing assumption of the
whole approach — proven on the area example before any of the machinery was built.

Running compiler code from this position is also the source of every panic the tool has hit. The
constraints it imposes — never force a query that emits, instantiate a binder before relating it, keep
each fresh `InferCtxt`'s variables to itself — are catalogued together in
[rustc diagnostic internals](rustc-diagnostic-internals.md#panic-hazards-running-compiler-code-inside-the-emitter),
and the boundaries at the end of this document note where a hazard puts a case out of reach.

## How the root cause is recovered

The recovery is a pipeline of typed lookups with no string parsing until the very last step decodes a
field name. It runs in the driver's [`resolve`](../../crates/cargo-cgp-driver/src/resolve) module —
stage files `anchor`, `walk`, `classify`, `label`, and `cgp_item` behind a re-exporting `mod.rs` — and
fills the rustc-free `Cause`/`Leaf`/`FieldIssue`/`Resolved` types with owned `String`s, so the wording
that consumes them needs no compiler. Every stage is anchored by `DefId` to the CGP crate that defines
the trait or type it matches, so a same-named item from an unrelated crate can never drive a
replacement. The stages run in order: anchor the starting obligation, walk it down to the leaves,
decode and classify each leaf, render the chain, and emit.

Two facts hold across every stage, and everything below assumes them. **The obligation the walk works
on is always a real consumer-trait obligation** `Ctx: ConsumerTrait<Params…>` — never a
`CanUseComponent` wrapper — and **the traits are recognized structurally, without `IsProviderFor`**:

- A **provider trait** is identified by the delegation blanket `#[cgp_component]` generates —
  `impl<Ctx, P> ProviderTrait<Ctx> for P where P: DelegateComponent<Marker>, …` — whose `Self` is a
  bare type parameter bounded by `DelegateComponent` (`is_provider_trait` / `provider_blanket_marker`
  in `cgp_item.rs`). That same blanket's `DelegateComponent<Marker>` bound also yields the component
  marker when an anchor needs it, in place of reading the `IsProviderFor<Marker, …>` supertrait.
- A **consumer trait** is identified by its blanket impl `impl<C> Consumer for C where C: Provider<C>`
  routing to such a provider (`consumer_provider_trait` / `is_consumer_trait`).
- The **marker → consumer** inversion the check and use-site anchors need (`marker_to_consumer`) is the
  composition of those two: the provider trait whose blanket keys on the marker, then the consumer
  whose blanket routes to that provider.

### Anchoring the starting obligation

The resolver recovers the obligation the compiler failed to prove in one of five ways, tried in order;
the first that succeeds wins. Each produces the same thing — the real consumer-trait obligation
`Ctx: ConsumerTrait<Params…>` to seed the walk — but recovers it from a different failure shape.

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
as the context, and instantiates the impl's supertraits for that `Self`. A supertrait that is a CGP
consumer trait on that context and does not hold **is** the obligation to walk — the resolver seeds it
directly (`wrapper_consumer_causes`), with its concrete component parameter intact
(`CanHandleApi<QueryBalanceApi>`, not the `()` a parameterless re-check would substitute), so no marker
detour is needed.

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
each key to its consumer trait, seeds that consumer obligation, and keeps the ones that do not hold. A
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

**From a use site, by consumer trait.** The final anchor closes the namespace-joined gap the previous
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
[`acceptable/use-type/`](../../tests/ui/acceptable/use-type)). It is tried last, so a directly-wired
context keeps the more precise per-component recovery.

### Walking the dependency graph downward

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
- An unmet **namespace-lookup bound** is a missing-redirect-wiring leaf too. It is recognized not by
  name but by the trait's *fingerprint* — a single `Delegate` associated type, which `DefaultNamespace`,
  the `DefaultImpls*` traits, and every user `cgp_namespace!` trait share — so a same-named user
  namespace is caught without a `DefId` anchor.
- An **ordinary bound on a foreign type** (`f64: Eq`) is a leaf, and the descent must not walk into
  whatever unrelated `std` blanket impl happens to match its `Self` (an `impl<F: FnPtr> Eq for F` would
  otherwise fabricate a misleading `f64: FnPtr` step).

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
`where`-obligations **all hold**, yet is itself unmet, is failing for a projection/associated-type
mismatch the trait-clause walk cannot see; the resolver looks among that impl's own predicates for the
one form it can pin down — an unmet `HasField` projection
(`<Ctx as HasField<Symbol!("f")>>::Value == T`), a field present with the wrong type — and completes
the branch with that field's `HasField` trait ref, tagging the path with the expected type so the leaf
renders as a field-type mismatch (the `E0271`
case shown earlier). This projection search *also* prefers the concrete-`Self` impl over the delegation
blanket (`has_field_projection_mismatch` / `impl_field_projection_mismatch`): the projection lives on
the provider's own impl, and matching the blanket first — which carries no projection — would wrongly
report no mismatch; but a getter trait whose *only* impl is a blanket
(`impl<C: HasField<..>> HasName for C`) still has its projection read from that blanket, so a blanket is
deferred, not skipped outright. A branch with no such projection yields nothing and declines to the
fallback. Finally, a branch that bottoms out on **pure wiring plumbing** — a `DelegateComponent` on a
type *other* than the context — is a routing dead-end and is dropped, since the real cause is found down
another branch. A `DelegateComponent` on the context is the one exception, never plumbing but the
missing-wiring leaf itself: a delegation that *holds* is pruned before it can be a leaf, so the only way
one bottoms out unmet on the context is that the context genuinely does not wire it.

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

### Decoding, classifying, and rendering a leaf

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
- **Type-level spines are resugared.** A rendered label's `Self` type has its `Cons<A, Cons<B, Nil>>`
  product spine read back as `Product![A, B]` and its `Either<A, Either<B, Void>>` sum spine as
  `Sum![A, B]`, so a field- or variant-list handler (the modular-serialization example's
  `FieldsSerializer` over a record's `Cons` field list) reads as the flat list the programmer wrote. A
  spine whose elements are all named fields `Field<Symbol!("name"), Type>` resugars one step further to
  the record or variant it describes — a product to `Struct! { name: Type, … }`, a sum to
  `Enum! { Name(Type), … }` — so a `HasFields` field list reads as
  `Struct! { message_id: u64, date: DateTime<Utc>, … }` rather than a chain of `Field` cells.
  `Struct!`/`Enum!` are not real CGP macros; like `Path!`'s `.*` wildcard they are a readability-only
  form. Each cell is anchored by `DefId` to the CGP crate that defines it (`Cons`/`Nil` in
  `cgp-base-types`, `Either`/`Void`/`Field` in `cgp-field`), so a same-named type from another crate is
  never resugared, and elements are rendered recursively so a nested list resugars in turn.

Each rendered entry is stamped with its own [`CGP-E1xx` code](../error-code.md) — one per template, so
`consumer trait impl` (`CGP-E101`), `provider trait impl` (`CGP-E102`), `redirect lookup` (`CGP-E104`),
and the general `trait impl` (`CGP-E105`) each carry a distinct tag, and a terminal leaf takes a leaf
code (`CGP-E106`–`CGP-E109`), except a pass-through ordinary bound, which stays uncoded. The cleaned
path folds into a [`DependencyTree`](error-processing.md) spine, rendered as `cargo tree`-style indented
text by the [`termtree`](https://crates.io/crates/termtree) crate in the rustc-free
`cargo-cgp-error-processing` crate, so the rendering is unit-tested on any toolchain.

### Emitting the transformed diagnostic

The wording is decided rustc-free and only *applied* by the emitter. The emitter maps the diagnostic's
own rustc code to a rustc-free [`DiagKind`](../../crates/cargo-cgp-error-processing/src/diagnosis/plan.rs)
(`E0271` a field mismatch, `E0599` a use-site method, everything else a plain check) and hands that,
the main-message text, the `Resolved`, and the name map to `plan_resolved`, which returns a
`DiagnosisPlan`: the rewritten header (or `None` to keep rustc's), the derive `help`s, and one note per
cause.

`plan_resolved`'s `categorized_header` is what picks the headline class described earlier — the
`CGP-E001` consumer form (worded from the resolution's context and consumer trait(s), pluralized when a
use-site failure spans several components, and also used for an `IsProviderFor` bound whose provider is
a `RedirectLookup`), the `CGP-E002` provider form from the text rewrite when rustc's own header names a
real wired provider's `IsProviderFor`, the `CGP-E003` field-type-mismatch form, or the `CGP-E009` plain
wrapper form. One extra case routes here: a field-mismatch-coded (`E0271`) failure the resolver traced
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
plan's `help`s (one per distinct type that must derive `#[derive(HasField)]`, or the `Deref` target; a
field-type mismatch contributes none) and the plan's notes — one per root cause, each opening with its
`root cause:` lead over `this is required through the dependency chain:` and the tree beneath (the lead
omitted when the kept header or the `CGP-E003` header already states the bound). rustc's structured
suggestions are discarded with its notes, the diagnostic's Rust code is never touched, and a provider
with two absent dependencies yields two notes. The JSON emitter regenerates every rendered and
structured field from the `DiagInner` for free, with rustc's note-continuation indentation aligning each
tree's box-drawing under its `= note:`. A final cross-diagnostic de-duplication (keyed on the recovered
cause, the rendered text, or the coded header) suppresses a transformed diagnostic that re-reports a
failure already shown.

## Boundaries and open ends

The resolver is deliberately bounded, and a few edges are worth recording. Because it anchors the five
ways above, a wiring failure that is *none* of them still declines. The consumer-trait use-site anchor
widened the reach considerably — a manual supertrait bound in a trait definition or `where` clause, and
a namespace-joined use-site call, both now resolve whenever a local CGP consumer trait and its context
appear in the diagnostic's spans (the once-declining `use_type_foreign_unsatisfied`,
`use_type_nested_unsatisfied`, and `namespace_join_use_site` fixtures, now under
[`acceptable/`](../../tests/ui/acceptable)). What still declines is a failure that names no local
consumer to anchor on: a caret only on a *provider* struct's own impl (whose `Self` is the provider,
reaching no consumer on a context), a generic component's trait definition, or a use-site failure on a
*foreign* generic consumer whose context and dispatch parameters are unrecoverable — the
shell-scripting DSL's `hello_name`, an `E0277` on `app.handle(PhantomData::<Program>, Vec::new())` whose
`Code` and `Input` come only from the call and whose combinator plumbing the text rewrite then exposes
([`cascade_after_use_site`](../../tests/ui/usability/use-site/cascade_after_use_site.rs) pins the
class). The wrapper-chain descent is itself bounded — it follows only real impl `where`-clauses, reports
a cause only at a genuine CGP consumer on a local context, and stops at a recursion bound — so it cannot
fabricate a chain from an unrelated bound.

One use-site shape is out of reach for a hard reason worth recording: a **consumer-method call whose
failure is an `E0271`, not an `E0599`** — `app.deserialize_json_string::<Payload>(…)` on a context that
cannot deserialize `Payload`, which fails as a type mismatch on the capability's output (the
modular-serialization arena test hits this). Its caret sits on the method call, naming no
context-definition span, so neither use-site anchor finds a context; and recovering the obligation would
need the compiler's **typeck results**, which the resolver cannot obtain. `tcx.typeck` replays its
cached diagnostics when forced, so forcing it from the emitter re-enters the diagnostic context and
panics — the re-entrant-emission hazard in
[rustc diagnostic internals](rustc-diagnostic-internals.md#re-entering-the-diagnostic-context-lock-was-already-held).
Only the fresh-`InferCtxt` trait solver is safe to re-enter mid-emit; a full query is not, and there is
no hook between typeck and the fatal error to precompute the result. So this failure falls through to
rustc's output, usually redundant with the `check_components!` failure for the same capability, which
the resolver *does* reshape.

A few parameter-recovery limits remain. The impl-site path recovers a generic component's concrete
parameter from the supertrait, and the by-component use-site path recovers it for an `open`-dispatched
component from the `PathCons<Component, Value>` redirect key; what neither use-site path recovers is the
parameter of a **non-dispatched generic component** — the by-component path re-checks its bare marker
with an empty `()` slot, and the by-consumer path only fires for a consumer whose sole generic is
`Self`. Such a failure declines to the fallback and keeps rustc's misleading method-syntax advice
([`generic_consumer_use_site`](../../tests/ui/usability/use-site/generic_consumer_use_site.rs) pins the
class, and the [usability issue](../issues/usability.md) records the plausible recovery: re-check the
wired delegate's *implemented* parameter values instead of the meaningless `()` form). And the walk
uses an **empty parameter environment** throughout, which suits the concrete check
impls the fixtures exercise but will need the impl's own environment to extend cleanly to checks that
carry generic parameters. The resolver renders only leaves it can trust — a `HasField` field (missing,
underived, or type-mismatched), a missing wiring, a namespace redirect the context does not terminate,
an ordinary foreign bound, or a terminal capability bound — and declines an associated-type projection
mismatch that is *not* a `HasField` one, dropping pure wiring-plumbing dead-ends, so a diagnostic whose
only recoverable leaf is one of those falls back. Parallel branches, deep nesting, and non-field leaves,
by contrast, are all handled.

How a transformed diagnostic is *marked* as CGP is settled by the [error-code scheme](../error-code.md):
a rewritten, classified main message carries its `[CGP-Exxx]` code inline, and everything else — a kept
header over rewritten sub-messages included — stays in rustc's own `error[E0277]:` form. There is no
separate header brand; the inline code is the only marking.

## Source

- [`crates/cargo-cgp-driver/src/resolve/`](../../crates/cargo-cgp-driver/src/resolve) — the typed
  resolution, split by stage behind a re-exporting `mod.rs` and building the rustc-free `Resolved`
  model. Every anchor feeds the walk the real consumer obligation `Ctx: ConsumerTrait<Params…>`, never
  a `CanUseComponent` wrapper.
  - `anchor.rs` holds the five anchors and the shared `consumer_obligation` they build the seed with —
    the `Params`-slot ungrouping decided by the consumer's own generics (a single tuple-typed
    parameter kept whole, a lifetime restored from `Life<'a>` via `life_region`, any mismatch
    declining rather than handing the solver a malformed trait ref): `resolve_check_failure` (matches
    the check impl by span, then `can_use_to_consumer_obligation` maps its
    `CanUseComponent<Marker, Params>` assertion through
    `marker_to_consumer` to the consumer obligation); `resolve_impl_site` (recovers the context and the
    consumer supertrait from an enclosing `impl Trait for Context` block, heading the tree with the
    impl's own wrapper trait — `[CGP-E001]` or `[CGP-E009]` by its blanket-impl fingerprint — through
    the shared `wrapper_consumer_causes`, which seeds the supertrait directly); `resolve_wrapper_chain`
    (the foreign-wrapper case, descending each hop's `where`-clauses via `wrapper_chain_children` read
    un-normalized so an associated-type bound descends to its base trait, until `consumer_handoff_causes`
    reaches a CGP consumer on the context, named plainly with `subject_is_context = false`);
    `resolve_use_site` (recovers the context ADT from the diagnostic's spans and its wired components
    from `DelegateComponent` impls via `delegated_check_targets`, mapping each marker to its consumer
    and recovering an `open`-dispatch value from a `PathCons` key through `open_dispatch_target`, while
    skipping a raw path key, a redundant bare marker, a free-parameter catch-all, and a `namespace …;`
    blanket `__Key__` key); and `resolve_use_site_consumer` (recovers a local, non-generic CGP consumer
    trait from the diagnostic's spans and walks `Ctx: Consumer` directly — the anchor that reaches a
    namespace-joined context).
  - `walk.rs` walks the cause chain to each terminal leaf: `resolve_leaves`/`collect_leaf_paths`, the
    descendable-vocabulary rule (`is_descendable` — provider traits, `DelegateComponent`, and context
    obligations, *not* `IsProviderFor`/`CanUseComponent`), the `is_workaround_plumbing` drop of a
    `CanUseComponent`/`IsProviderFor` dependency beside the real obligation, the cycle guard and
    `MAX_DEPTH` backstop, the placeholder instantiation of a higher-ranked binder
    (`enter_forall_and_leak_universe`), the plumbing-leaf drop, the foreign-getter descent into just
    context-side dependencies plus a same-trait list recursion, `impl_where_obligations` preferring a
    concrete-`Self` impl over the delegation blanket and solving satisfiable clauses first,
    `is_reportable_leaf` keeping an unmet `DelegateComponent` only on the context, and
    `has_field_projection_mismatch`/`impl_field_projection_mismatch` finding an unmet `HasField`
    projection on the concrete-`Self` impl (deferring the blanket).
  - `classify.rs` classifies a leaf (a field by inspecting the struct and its `Deref` chain, a
    field-type mismatch with `field_type` reading the actual type by `DefId`, a missing wiring, a
    missing redirect wiring told apart by `is_path_cons`, or a bound).
  - `label.rs` folds the inner chain into a `DependencyTree`, naming each consumer/provider node off its
    trait `DefId` and the obligation's arguments (`trait_generics`) and dropping the plumbing, with
    `render_ty` resugaring a `DefId`-anchored `Cons`/`Nil` or `Either`/`Void` self type to
    `Product![…]`/`Sum![…]`, or — when every element is a `Field` — to `Struct! { … }`/`Enum! { … }`.
  - `cgp_item.rs` holds the structural, `IsProviderFor`-free trait recognition — `is_provider_trait` /
    `provider_blanket_marker` (the `DelegateComponent`-bounded provider blanket), `consumer_provider_trait`
    / `is_consumer_trait`, and `marker_to_consumer` — plus the `Symbol!` field-name decode and
    `is_namespace_lookup_trait` (by the single-`Delegate`-associated-type fingerprint). A sibling
    `conflict.rs` handles the duplicate-key `E0119` conflict — a separate transform documented in
    [The driver](driver.md#reshaping-a-duplicate-key-conflict).
- [`crates/cargo-cgp-driver/src/emitter/`](../../crates/cargo-cgp-driver/src/emitter) — the `try_resolve`
  seam (gated by a cheap `mentions_wiring` scan, an `E0271`/`E0277` code, or a method-bounds `E0599`,
  with a resolution-class `E0599` excluded so the solver never runs on an error emitted
  mid-`predicates_of`) that tries the five anchors in turn, and the `transform_resolved` mutation it
  feeds — mapping the rustc code to a `DiagKind`, calling `plan_resolved`, and applying the plan to the
  `DiagInner`, falling back to the in-place text rewrite when resolution returns `None`. A final
  cross-diagnostic de-duplication suppresses a re-report of a failure already shown.
- [`crates/cargo-cgp-error-processing/src/diagnosis/`](../../crates/cargo-cgp-error-processing/src/diagnosis)
  — the rustc-free model and wording: `leaf.rs`/`resolved.rs` (the `Leaf`, `FieldIssue`, `Cause`, and
  `Resolved` types), `wording.rs` (the coded headers, `root cause:` notes, and derive `help`s), and
  `plan.rs` (`DiagKind`, `DiagnosisPlan`, and `plan_resolved` with its `categorized_header`),
  unit-tested in [`tests/diagnosis.rs`](../../crates/cargo-cgp-error-processing/tests/diagnosis.rs).
- [`crates/cargo-cgp-error-processing/src/tree.rs`](../../crates/cargo-cgp-error-processing/src/tree.rs) —
  the `DependencyTree` type and its `cargo tree`-style renderer over `termtree`, unit-tested in
  [`tests/tree.rs`](../../crates/cargo-cgp-error-processing/tests/tree.rs).
- [`crates/cargo-cgp-driver/src/config.rs`](../../crates/cargo-cgp-driver/src/config.rs) — the crate and
  trait-name anchors the resolution matches against.

## Tests

The resolver is exercised end to end by the UI snapshot suite. The fixtures it reshapes live under
[`tests/ui/acceptable/`](../../tests/ui/acceptable) — the `fields/`, `field-types/`, `providers/`,
`generic/`, `resolution/`, `wiring/`, `use-site/`, and `use-type/` subgroups, carrying `.cgp.stderr`
snapshots of the transformed output. The failures it still declines — a use-site `E0277` on a foreign
generic consumer, and a use-site `E0599` on a *local* generic consumer whose dispatch parameter no
span recovers — keep their fallback snapshots under
[`tests/ui/usability/use-site/`](../../tests/ui/usability/use-site) (`cascade_after_use_site`,
`generic_consumer_use_site`), so the two sides together pin both the transform and the decline
boundary. [Testing](testing.md) describes the suite
and its bless workflow. The fixtures group by what they pin.

Each **leaf class** has fixtures for its field, wiring, and redirect shapes:

- `base_area_1` — a genuinely missing field (the worked example).
- `missing_has_field_derive` — a present-but-underived field, with the derive `help`.
- `field_via_deref` — a field on a `Deref` target, with the `help` pointed at the target.
- `field_type_mismatch` and `field_type_mismatch_1` — a matching name with a mismatched type, read
  through a getter and directly via an `#[implicit]` argument.
- `field_type_mismatch_modules` — two `Rectangle` contexts in separate modules with differently-typed
  `height` fields, proving the actual-type query is `DefId`-anchored.
- `basic_missing_wiring` — a `#[uses]` dependency on an unwired component.
- `direct_missing_wiring` — a checked component wired nowhere (a single-node chain).
- `parallel_missing_wiring` — two unwired components (two notes).
- `record_field_chain` — a record provider building each field through the context over a recursive
  `Cons`/`Nil` handler (the modular-serialization `DeserializeRecordFields`/`HandleMapEntry` shape),
  whose tree entries also pin the `Cons`/`Nil` → `Struct! { … }` resugaring.
- `sum_variant_chain` — the sum counterpart over a `Sum![u64, f64]` spine of bare types, pinning the
  `Either`/`Void` → `Sum![…]` resugaring left as a plain list.
- `enum_variant_chain` — a sum of *named* variants, pinning the `Enum! { Rect(u64), … }` form.
- `unregistered_prefix_path`, `qualified_prefix_path` (a module-qualified path still folding to a clean
  `@…`), `multi_redirect_missing` (several hops), and `open_missing_type_key` — the namespace-redirect
  variants.

Several fixtures pin the **harder mechanics**:

- `parallel_branches` — two independent missing fields, two sub-errors.
- `deep_nesting` — higher-order providers nested four deep, one long spine.
- `dependency_cascade` — a chain of providers each depending on the next, its intermediate consumers
  each a `[CGP-E101]` node.
- `mixed_rust_error` — a CGP tree beside an untouched `E0308`.
- `same_name_components` — two components sharing a marker name in different modules, resolved to their
  own traits (off their `DefId`s) with no cross-over.
- `generic_area_multi` — a three-parameter component, its parameters reattached to the labels from the
  obligation's own arguments.
- `lifetime_component` — a component carrying a *lifetime* parameter (`(Life<'a>, str)` in its check
  entry), the lifetime restored from its `Life<'a>` lift to a region when the consumer obligation is
  rebuilt, and the provider label's context read by type position past the leading lifetime.
- `tuple_param_component` — a component whose single parameter is itself a *tuple* type, kept whole
  (`CanFormatPair<(u32, u64)>`) rather than spread into two parameters by the params-slot ungrouping.
- `check_providers_layer` (under [`acceptable/providers/`](../../tests/ui/acceptable/providers)) — a
  `#[check_providers(...)]` per-layer assertion, whose `IsProviderFor`-supertraited check impl no
  anchor matches directly, resolved through the use-site anchor instead (rustc's "not implemented for
  `Rectangle`" note spans the context's struct definition) into the failing layer's root-cause tree.
- `ordinary_bound_unsatisfied` — a non-field `f64: Eq` bound whose rustc header is kept over a lead-less
  chain note.
- `foreign_getter_missing_wiring` — the money-transfer `UseBasicAuth` shape, where the walk descends a
  request getter's blanket impl into its context-side dependency and the misleading second root cause
  collapses into the one missing wiring, under a promoted `CGP-E001` header.
- `higher_ranked_descent` — a recursive provider with a `Self: for<'a> CanEncodeItem<&'a Value>`
  dependency (the `SerializeIterator` shape) that used to feed an escaping bound variable into the
  solver and panic rustc, now resolved through the placeholder instantiation.
- `nested_higher_ranked_descent` — the same nested twice through the record machinery (the
  `MessagesArchive` shape), which used to decline to the raw fallback.
- `enum_hasfields_lock` — a resolution-class `E0599` emitted mid-`predicates_of`, which the resolver
  must decline rather than run its solver on and re-enter the `DiagCtxt` lock.

The **use-site paths** are pinned by the [`acceptable/use-site/`](../../tests/ui/acceptable/use-site)
and [`acceptable/use-type/`](../../tests/ui/acceptable/use-type) fixtures:

- `missing_dependency` and `unsatisfied_dependency` — a consumer-method `E0599` giving the `CGP-E001`
  header with the method-syntax advice dropped over a `missing field` note.
- `missing_wiring` — a use-site `E0599` whose provider needs an unwired component.
- `ordinary_bound_unsatisfied` — a use-site `f64: Eq`, code kept `E0599`.
- `open_dispatch_use_site` — the dispatch value recovered from the redirect key, so the header names
  `CanEncodeItem<Seq<u64>>` and the note reaches the real `@ItemEncoderComponent.u64` wiring rather than
  reporting the internal `PathCons` key as a bogus consumer trait.
- `namespace_join_use_site` — a use-site `E0599` on a namespace-joined context, anchored on the
  `CanGreet` consumer trait from the diagnostic and walked through the namespace's `RedirectLookup` to
  the missing field, with the blanket `__Key__` forwarding skipped.
- `use_type_foreign_unsatisfied` and `use_type_nested_unsatisfied` — an unsatisfiable `#[use_type]`
  abstract-type import in a trait definition, recovered by the consumer-trait anchor into a
  `[CGP-E001]` missing-wiring tree instead of leaking generated `__…__` placeholder names.

The **impl-site and wrapper-chain paths** are pinned by:

- `manual_supertrait_impl` — a wrapper carrying a generic CGP consumer supertrait implemented directly
  on the context (the `CanHandleApiSend` shape), failing at both the impl header `E0277` and its
  forwarding-call `E0599`, both collapsing to one `[CGP-E009]` block.
- `traced_send_wrapper` — an async `Send`-recovery wrapper whose opaque-future `E0271` names no CGP
  construct, traced to the wrapper-headed tree.
- `foreign_wrapper_chain` — a routing trait on a foreign `Box<App>` whose `where`-clause chain reaches a
  CGP consumer two hops down, the cause reached through a projection bound's base trait, headed by the
  `[CGP-E009]` foreign-plain form.

Finally, the leaf wording and the tree renderer are unit-tested over hand-built `Resolved` values,
independently of the compiler:

- [`cargo-cgp-error-processing/tests/diagnosis.rs`](../../crates/cargo-cgp-error-processing/tests/diagnosis.rs)
  — the coded headers, the `root cause:` notes, and the derive `help`s.
- [`cargo-cgp-error-processing/tests/tree.rs`](../../crates/cargo-cgp-error-processing/tests/tree.rs) —
  the `cargo tree`-style renderer.

## Further reading

- [The driver](driver.md) — the emitter seam this resolver extends, and the trait-renaming text rewrite
  it falls back to (which still recognizes `IsProviderFor`/`CanUseComponent` in rustc's output).
- [Error processing](error-processing.md) — the rustc-free crate that holds the `Resolved` model, the
  wording, and the fallback post-processing.
- [Check traits](../../../cgp/docs/concepts/check-traits.md) — why `IsProviderFor`/`CanUseComponent`
  exist, the workaround this resolver is designed to make removable.
- [rustc diagnostic internals](rustc-diagnostic-internals.md) — where rustc drops information the
  resolver must recover, and the panic hazards of running compiler code inside the emitter.
- [The error pipeline](error-pipeline.md) — where this driver-side transformation sits among the
  pipeline's stages.
- [CGP check-trait failure](../../../cgp/docs/errors/checks/check-trait-failure.md) — the upstream error
  class the resolver reshapes.
