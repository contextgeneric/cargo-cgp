# Typed resolution: the call-site anchor

This document covers the sixth and last-tried anchor of the driver's
[typed root-cause resolution](typed-root-cause-resolution.md): recovering a use-site failure's
obligation from the failing call expression's own HIR, when its spans touch nothing the
[span-matching anchors](typed-resolution-anchors.md) can read.

The call-site anchor exists for the use-site failure that leaves *no usable span at all*, and its
design follows from working out what can still be known once the spans are gone. This document builds
the failure shape from a small self-contained program, shows why every span-matching anchor
declines on it, and then develops each recovery step with the reasoning behind it.

## The failure shape: wiring that matches unconditionally

The shape arises whenever a context's wiring answers a whole *family* of component parameters with
one generic entry, so that resolving the method always finds a provider and only that provider's
deeper dependencies fail. The natural home of this pattern is the
[handler family](https://github.com/contextgeneric/cgp/blob/main/docs/concepts/handlers.md) — an advanced corner of CGP whose
`CanHandle<Code, Input>` consumer turns an `Input` value into an output, with a phantom `Code`
*type* selecting which computation runs, so one context can host many computations and wire each
`Code` differently.
Nothing below is specific to handlers, though: any consumer whose wiring matches unconditionally and
fails only in its dependencies produces the same shape.

The two combinators that sequence such a pipeline are core CGP providers (from `cgp-handler`, not
from any example), and the recovered tree renders them by name, so a reader needs their shape. Both
are documented in full under [handler combinators](https://github.com/contextgeneric/cgp/blob/main/docs/reference/providers/handler_combinators.md):

- **`ComposeHandlers<ProviderA, ProviderB>`** runs two handlers back to back, feeding the *output* of
  the first as the *input* of the second. So its two dependencies are asymmetric —
  `ProviderA: Handler<Ctx, Code, Input>` on the pipeline's own input, and
  `ProviderB: Handler<Ctx, Code, ProviderA::Output>` on whatever the first stage *produces*. That
  asymmetry is what this section turns on.
- **`PipeHandlers<Product![A, B, C]>`** generalizes that to a type-level list, folding it right to
  left into `ComposeHandlers<A, ComposeHandlers<B, C>>` — so a three-stage pipeline threads its input
  through `A`, then `B`, then `C`, each stage's output type feeding the next stage's input type. A
  program written with a pipe operator (a step, then another step, then another) desugars to exactly
  this.

Both are zero-sized dispatch plumbing the programmer never writes by hand — they appear only because a
pipeline program's wiring expands to them — which is precisely why a diagnostic that stops on one
names no cause a reader can act on.

The following program (condensed from the
[`cascade_after_use_site`](../../tests/ui/acceptable/use-site/cascade_after_use_site.rs) fixture)
wires every pipeline program `Prog<Steps>` — whatever its `Steps` — to a `PipeHandlers` composition
of those steps. The first step reads the context's `name` field, which `App` does not have; that
missing field is the root cause the anchor must recover:

```rust
/// A program is a *type*: a pipeline of steps, selected by the phantom `Code` tag.
pub struct Prog<Steps>(pub PhantomData<Steps>);

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

/// First pipeline step: read the context's `name` field — the dependency `App` cannot meet.
#[async_trait]
#[cgp_impl(new HandleName)]
#[use_type(HasErrorType.Error)]
impl<Code, Input> Handler<Code, Input>
where
    Self: HasName,
    Input: Send,
{
    type Output = String;

    async fn handle(&self, _tag: PhantomData<Code>, _input: Input) -> Result<String, Error> {
        Ok(self.name().to_owned())
    }
}

// A second step, `HandleShout`, uppercases the `String` the first step produces. It is
// defined the same way, its dependencies all hold, and it plays no part in the failure.

cgp_namespace! {
    new MyNamespace: DefaultNamespace {
        @cgp.core.error.ErrorTypeProviderComponent:
            UseType<String>,

        // One generic entry serves *every* program: any `Prog<Steps>` runs as the pipeline of
        // its steps. This is what makes the wiring match unconditionally.
        @cgp.extra.handler.HandlerComponent.<Steps> Prog<Steps>:
            PipeHandlers<Steps>,
    }
}

#[derive(HasField)]
pub struct App {
    // No `name` field.
}

delegate_components! {
    App {
        namespace MyNamespace;
    }
}

async fn run_app(app: &App) -> Result<(), String> {
    app.handle(PhantomData::<Prog<Product![HandleName, HandleShout]>>, Vec::new())
        .await?;
    Ok(())
}
```

Because the `<Steps> Prog<Steps>` entry matches *any* program, rustc's method probe succeeds —
`handle` exists for `App` — and only later does the provider's transitive `Self: HasName` bound
fail. So the failure arrives as an `E0277` on the call, not the `E0599` "method not found" a
directly-missing method produces, and that difference disarms every span-matching anchor. There is
no `check_components!` entry, so no check impl's span matches the caret. The call sits in a plain
`fn`, not inside an `impl` block, so the impl-site and wrapper-chain anchors find no enclosing impl.
An `E0599` would have carried a "method not found for this struct" span on `App`'s definition — the
handle the by-component use-site anchor grabs — but this `E0277` points only at the call. And the
consumer trait `CanHandle` is foreign and generic, so the by-consumer anchor, restricted to local
consumers whose only generic is `Self`, is out too. Before this anchor existed, the diagnostic fell
to the text rewrite, which reported the failure *three times* (once per rustc re-report at the call
and its `.await`) under `[CGP-E002]` headers naming `PipeHandlers<…>` and `ComposeHandlers<…>` — the
dispatch plumbing — as the failing "provider", with the missing `name` field appearing nowhere.

## What the call still knows

The spans are useless, but the call expression itself contains almost everything the walk's seed
obligation `App: CanHandle<Code, Input>` needs — provided it is read from HIR alone.
`tcx.typeck`, the query that would answer every question at once, replays its cached diagnostics
when forced and so aborts the compiler from inside the emitter (the re-entrancy hazard in
[rustc diagnostic internals](rustc-diagnostic-internals.md#re-entering-the-diagnostic-context-lock-was-already-held));
HIR, by contrast, is fully built long before analysis, and the only queries this recovery touches
(`type_of`, `fn_sig`, `generics_of` on items the failing code already named) are cached by the very
type-checking that produced the diagnostic.

**The receiver names the context.** A consumer trait is implemented on the context, so in a
consumer-method call the receiver *is* the context by construction — no guessing is involved, only
reading the receiver's type without typeck. The anchor follows the receiver expression
syntactically: a path to a binding leads to a `let` (typed by its annotation, or by a struct-literal
initializer) or to a fn parameter (typed by the enclosing signature — the fixture's `app: &App`); a
struct literal, unit-struct value, const, or static names its type directly; a call to a
non-generic fn takes the callee's declared return type; references are peeled along the way. A
receiver whose type genuinely needs inference — a method call's result, a field access — declines,
as does a generic context, whose type arguments are exactly what the missing typeck results would
have supplied. The trait candidates come from the method *name*, and are of two kinds: every CGP
**consumer trait** (recognized structurally, in any crate) declaring a `self` method of that name,
tried first so a directly-wired consumer keeps its precise recovery; then every local `#[cgp_fn]` /
`#[blanket_trait]` **capability trait** declaring such a method — a blanket-impl trait that is not a
CGP component (no provider trait, no `DelegateComponent`), consumed like a consumer (`app.describe()`)
and seeding the same walkable obligation `Ctx: Describe` whose `Self` is the context. A capability
trait is not a CGP component, so its result is headed `[CGP-E009] the trait …` rather than
`[CGP-E001] the consumer trait …` — the same wording the impl-site anchor gives such a trait reached
through a wrapper — by clearing the `Resolved::consumers_are_cgp` flag the walk sets. This is what
recovers a direct call to a `#[cgp_fn]` capability the context cannot satisfy: an `E0599` whose real
cause (a field one composed capability reads) rustc buries in a mid-stack note under its method-probe
candidate list ([`cgp_fn_use_site`](../../tests/ui/acceptable/use-site/cgp_fn_use_site.rs)).

## Parameters by signature unification, not by convention

Recovering the component's parameters — the `Code` and `Input` in `CanHandle<Code, Input>` — is
where a design choice had to be made, and the choice is to assume **no calling convention at all**.

The tempting shortcut is a convention: in the handler family, the first argument is a
`PhantomData<Code>` tag, so "read the first argument's turbofish" would recover the `Code` here. But
CGP does not prescribe how a consumer method relates its arguments to its trait parameters — the
`PhantomData` tag is one family's idiom, not a rule. A consumer someone else defines may carry its
parameter in an ordinary value argument (`fn format_pair(&self, value: T)`), in a differently-shaped
tag type, spread across several arguments, or nowhere recoverable. Hard-coding any one shape would
quietly privilege one library's style and fail on every other.

What *is* always true is that the method's own declared signature records exactly where each trait
parameter appears among its inputs — that is the very information type inference consumes at a real
call. So the anchor runs a miniature of the same process the compiler would: it mints a fresh
inference variable for every parameter of the method's item, pins `Self` to the receiver's context,
and unifies each argument whose type the call *writes* syntactically against the corresponding
declared input. Whatever those unifications pin down, through the signature's own use of the trait's
generics, is recovered. The two fixture shapes show the same mechanism serving both idioms:

- In the program above, the argument `PhantomData::<Prog<Product![HandleName, HandleShout]>>` is
  unified with the declared input `_tag: PhantomData<Code>`, which binds
  `Code = Prog<Product![HandleName, HandleShout]>`.
- In [`generic_consumer_use_site`](../../tests/ui/acceptable/use-site/generic_consumer_use_site.rs),
  a consumer `CanFormatPair<T>` with the plain value method `fn format_pair(&self, value: T)` is
  called as `app.format_pair((1_u32, 2_u64))`; the written tuple type `(u32, u64)` is unified with
  the declared `value: T`, which binds `T = (u32, u64)`. No tag argument exists, and none is needed.

An argument's type counts as *written* when it is determined by the expression's own syntax: a
unit-struct or unit-variant value with its written path arguments, a struct literal, a reference to a
written expression, a literal whose type is definite (`"…"`, `true`, `'c'`, suffixed numerics), or a
call to a non-generic fn (its declared return type). Each written type is lowered by a deliberately
small syntactic HIR-type lowering — paths to ADTs and aliases through the cached `type_of`, defaulted
parameters filled in, lifetimes erased — that declines anything beyond it rather than guess.

A **tuple literal** is the one shape read *partially*: its *structure* is recovered even when some
elements' types are not written, each unwritten element seeded as a fresh inference variable (folded
into a placeholder with the rest of the seed). This matters because a provider commonly destructures
its input on a tuple shape — a branching interpreter taking `(condition_input, branch_input)`, a
comparison taking `(input_a, input_b)` — and its impl matches only against a tuple, never a flat
unknown. Collapsing a tuple whose every leaf is unwritten (`(Vec::new(), Vec::new())`) to one opaque
placeholder, as the all-or-nothing reading of the other shapes would, leaves such a provider's impl
unmatched and hides a cause that sits inside a *written* branch of the input — or, as with the
field a condition reads, a branch that does not depend on the input at all. The recovered arity and
its written elements are real call-side information; only the leaves the call does not type stay
unknown, and those are never reported. The [`call_site_tuple_input`](../../tests/ui/acceptable/use-site/call_site_tuple_input.rs)
fixture pins the shape.

## Unknown parameters become rigid placeholders

What the call does not write, the anchor does not invent. The fixture's second argument is
`Vec::new()`: its element type was never resolved (the trait resolution failing is precisely why),
and no syntactic reading can supply it. Each parameter left unconstrained after unification is
folded into a rigid **placeholder** type — rigid so it unifies with nothing concrete and can cross
between the walk's fresh inference contexts, unlike an inference variable.

The walk then treats a placeholder as an *unknown*, in both directions. It **descends through**
bounds that mention one, because a parameter-dependent bound can still lead to parameter-independent
dependencies deeper down — in the example, `HandleName`'s `Self: HasName` does not mention the
input at all, so the missing `name` field is reachable whatever the input turns out to be. But it
**never reports** a leaf that still carries one: a bound like `Input: Send` with an unknown `Input`
is unknowable, and reporting `_: Send` would fabricate a requirement the programmer cannot act on.
Only a root cause that holds *whatever the unknown parameter is* survives; a failure whose every
leaf depends on the unknown declines to the fallback exactly as before
([`generic_consumer_unwritten_arg`](../../tests/ui/acceptable/use-site/generic_consumer_unwritten_arg.rs)
pins that boundary — the same `CanFormatPair` call with the tuple passed through a plain variable,
whose type the call no longer writes). In the rendered output a placeholder prints as the `_` the
programmer would write, including one nested inside a recovered tuple (`((_, _), _)`) — the tree
renderer walks a tuple's elements rather than printing rustc's raw `!N` placeholder form.

Put together, the example's failure — three plumbing-worded blocks with no cause — becomes one
block, led by the cause:

```text
error[E0277]: [CGP-E001] the consumer trait `CanHandle<Prog<Product![HandleName, HandleShout]>, _>` is not implemented for context `App`
  --> src/main.rs:93:16
   |
93 |     app.handle(PhantomData::<Prog<Product![HandleName, HandleShout]>>, Vec::new())
   |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: root cause: [CGP-E106] missing field `name` on `App`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanHandle<Prog<Product![HandleName, HandleShout]>, _>` for context `App`
             └─ [CGP-E104] redirect lookup to `@cgp.extra.handler.HandlerComponent` in `App`
               └─ [CGP-E102] provider trait impl `Handler<Prog<Product![HandleName, HandleShout]>, _>` with context `App` for provider `PipeHandlers<Product![HandleName, HandleShout]>`
                 └─ [CGP-E102] provider trait impl `Handler<Prog<Product![HandleName, HandleShout]>, _>` with context `App` for provider `ComposeHandlers<HandleName, HandleShout>`
                   └─ [CGP-E102] provider trait impl `Handler<Prog<Product![HandleName, HandleShout]>, _>` with context `App` for provider `HandleName`
                     └─ [CGP-E105] trait impl `HasName` for `App`
                       └─ [CGP-E106] missing field `name` on `App`
```

The re-report rustc raises where the result is awaited resolves to the same cause and de-duplicates
away, and the `?`-operator cascade the call used to trail stays suppressed. A resolution from this
anchor is also planned as a use-site failure whatever its rustc code, so the header names the
consumer trait the call needs — never the dispatch plumbing rustc's own headline stopped on (see
[Emitting the transformed diagnostic](typed-resolution-output.md#emitting-the-transformed-diagnostic)).

## Why a wrong guess cannot fabricate an error

The anchor recovers from *guesses* — a method name can match several consumer traits, a receiver
binding can be misread — so every seed is gated on reality before anything is reported. The anchor
is tried last, after every span-matching recovery; a candidate obligation that actually *holds* is
skipped; and one that fails but whose walk reaches no reportable, placeholder-free leaf declines to
the fallback. A mis-guessed consumer or context therefore produces either nothing or a genuine
failing obligation of the context the programmer named — never an invented diagnostic.

## Tests

The anchor's fixtures live under
[`tests/ui/acceptable/use-site/`](../../tests/ui/acceptable/use-site) — `cascade_after_use_site`
(the worked example above), `generic_consumer_use_site` (the value-argument case),
`call_site_tuple_input` (the partial tuple recovery), `cgp_fn_use_site` (the `#[cgp_fn]` capability
call recovered as a `[CGP-E009]` block), and the `cascade_later_stage*` shapes whose pipeline stages
the walk then descends — with the decline boundary pinned by `generic_consumer_unwritten_arg`. The
consolidated catalog lives in the parent document's
[Tests](typed-root-cause-resolution.md#tests) section.

## Source

- [`crates/cargo-cgp-driver/src/resolve/call_site/`](../../crates/cargo-cgp-driver/src/resolve/call_site)
  — one file per stage: `find_call.rs` (locating the call and the candidate consumers),
  `receiver.rs` (the context off the receiver), `seed.rs` (the signature unification),
  `written_ty.rs` (the types the call's arguments write), and `lower.rs` (the small syntactic type
  lowering).

## Further reading

- [Typed root-cause resolution](typed-root-cause-resolution.md) — the pipeline overview and the
  five anchors tried before this one.
- [Typed resolution: walking to the root cause](typed-resolution-walk.md) — how the seeded
  obligation (placeholders and all) is descended.
- [rustc diagnostic internals](rustc-diagnostic-internals.md) — why `tcx.typeck` can never be
  forced from the emitter, the constraint this anchor's HIR-only reading exists to respect.
