# CGP Error Codes

`cargo-cgp` assigns a stable error code to every main message it rewrites into a CGP-specific one and
to every entry of the dependency tree it renders, and this document is the catalog of those codes. A
code names one recognized class of CGP mistake or one dependency-chain step — what it means, what
triggers it, and how to fix it — so that a reader who sees a code in the tool's output can look it up
here, and so that the tool's own tests and future JSON output can refer to a class by a short, stable
identifier rather than by its prose.

The three-digit space is split by *what* a code classifies. The **`CGP-E0xx`** range names **main
messages** — the diagnostic's headline. The **`CGP-E1xx`** range names **dependency-tree entries** —
each node of a `root cause:` note's `cargo tree`, one code per distinct rendering template. Keeping
the ranges apart lets a reader tell at a glance whether a code tags the error's headline or one hop of
the chain beneath it.

## The scheme, and why it looks unlike a Rust code

A CGP error code is the letters `CGP-E` followed by three digits — `CGP-E001`, `CGP-E002`, and so
on — shown in square brackets at the start of the rewritten main message:

```text
error[E0277]: [CGP-E001] the consumer trait `CanCalculateArea` is not implemented for context `Rectangle`
```

The `CGP-E` prefix is deliberately unlike Rust's own `E0277` shape, so a reader never mistakes one
namespace for the other: `E0277` is a Rust code you explain with `rustc --explain`, while
`[CGP-E001]` is a CGP code you look up here. The two coexist on the same line by design — **the
diagnostic's own Rust code is always kept**. A rewritten message restates the same error more
readably; it does not reclassify the error away from rustc, so `error[E0277]:` (or `error[E0599]:`
at a consumer-call use site, or `error[E0271]:` at a field-type mismatch) stays in the header, the
trailing `rustc --explain` line stays meaningful, and the CGP code rides inside the message text
where it reads as a tag on the sentence it classifies. This also keeps the code greppable: a reader
confirming a class need only search the output for `CGP-E001`.

## When a code is assigned

A code is assigned only when two things are both true: `cargo-cgp` **rewrote the main message**,
and that main message was **identified as a class of CGP error**. The rewrite preserves the
semantics of the original message — an unsatisfied `CanUseComponent<AreaCalculatorComponent>` bound
*is* the consumer trait `CanCalculateArea` failing to be implemented — and the code is the handle
for the class it was recognized as.

Everything else is uncoded by design. A diagnostic whose main message is not a CGP class keeps
rustc's own message and plain `error[E0277]:` header even when its *sub-messages* were rewritten —
the root-cause notes, renamed obligation chains, and resugared type names are supporting detail of
one error, not classifications of their own. The [uncoded rewrites](#uncoded-rewrites) section
below records each such rewrite, so the absence of a code on them is documented rather than merely
implied.

## Codes

The catalog today holds the classes the driver's emitter recognizes in a main message, in five
groups. The **check-failure** codes `CGP-E001`–`CGP-E003`, `CGP-E009`, and `CGP-E017` come from the
typed resolver; each carries the root cause in the accompanying `note`s — one per recovered cause,
each opening `root cause: …` over its dependency chain (see
[Typed root-cause resolution](implementation/typed-root-cause-resolution.md)) — except the two
mismatch codes `CGP-E003` and `CGP-E017`, whose main messages already state their cause in full and
so carry the chain alone. (`CGP-E009` is the check-failure code for a hand-written wrapper trait
rather than a CGP consumer, and `CGP-E017` the abstract-type counterpart of `CGP-E003`; both are
grouped with `CGP-E001`–`CGP-E003` below.) The **structural
wiring-conflict** codes `CGP-E004`–`CGP-E008` are different in kind: they come from the duplicate-key
conflict classifier, carry no root-cause note, and instead keep rustc's two carets, which already
point at the two colliding entries. They are five separate codes because each rewrites the `E0119`
into a distinct message form with its own fix. The **coherence-reshape** codes `CGP-E010` and
`CGP-E011` each rewrite a whole-program coherence error into a CGP-framed one carrying its fix in a
`help`: `CGP-E010` a wiring cycle's `E0275` (a cycle has no terminal leaf to descend to), and
`CGP-E011` an orphan-rule namespace registration's `E0210`/`E0117` (recovered from the offending impl
off the compiler, like the `E0119` family). The fifth group is the **lowering** codes
`CGP-E012`–`CGP-E016`, each recovered off the compiler from the generated impl the failing token sits
in and reworded with its fix in a `help`. Two concern a used-but-undeclared dependency: `CGP-E012` a
capability used in a `#[cgp_fn]`/`#[cgp_impl]` body but not declared via `#[uses(…)]`, and `CGP-E016`
an inner provider a higher-order provider calls but never imported via `#[use_provider]`. Three
concern a trait named where a provider trait belongs: `CGP-E013`/`CGP-E014` a `#[cgp_impl]` header
naming the component's consumer trait where its provider trait belongs (`CGP-E013`), or a trait that
is not a CGP component at all (`CGP-E014`); and `CGP-E015` an inner-provider bound (typically
`#[use_provider]`) naming the consumer trait rather than the provider trait. Each entry below gives
the rewritten message, the mistake behind it, the fix, and the upstream
[CGP error catalog](https://github.com/contextgeneric/cgp/blob/main/docs/errors/README.md) class it recognizes.

### `CGP-E001` — consumer trait not implemented

- **Message:** `` [CGP-E001] the consumer trait `<Consumer>` is not implemented for context
  `<Context>` `` (pluralized when a use-site failure spans several components).
- **Means:** the context cannot use a component it is expected to use — the wiring is missing,
  or a transitive dependency of the wired provider is unmet, so the blanket impl that would give
  the context its consumer trait does not apply.
- **Triggered by:** an unsatisfied `Context: CanUseComponent<Marker, Params>` bound — a
  `check_components!` / `delegate_and_check_components!` entry failing — or a consumer-method call
  (`E0599`) on a context that cannot use a component it wires. The Rust code stays whatever rustc
  assigned (`E0277` at a check, `E0599` at a call site).
- **Fix:** follow the `root cause:` note(s): add the missing field or derive, wire the missing
  component, or satisfy the ordinary bound the chain bottoms out on.
- **Upstream class:** [check-trait failure](https://github.com/contextgeneric/cgp/blob/main/docs/errors/checks/check-trait-failure.md).

### `CGP-E002` — provider trait not implemented

- **Message:** `` [CGP-E002] the provider trait `<Provider trait>` with context `<Context>` is not
  implemented for provider `<Provider>` ``.
- **Means:** a specific provider fails to implement its provider trait for the context — its
  impl-side `where`-clause dependencies do not hold — as asserted by `IsProviderFor`.
- **Triggered by:** an unsatisfied `Provider: IsProviderFor<Marker, Context, Params>` bound — a
  `#[check_providers(...)]` assertion, or a wiring step (such as a namespace `RedirectLookup`)
  whose provider-side failure rustc chose as the primary error.
- **Fix:** follow the `root cause:` note(s) to the dependency the provider is missing.
- **Upstream class:** [check-trait failure](https://github.com/contextgeneric/cgp/blob/main/docs/errors/checks/check-trait-failure.md)
  (its provider-side face).

### `CGP-E003` — field has the wrong type

- **Message:** `` [CGP-E003] expected a `<field>` field of type `<expected>` on `<Context>`, but
  found `<actual>` ``.
- **Means:** a context field the wiring reads is present and derives `HasField`, but its type is
  not the type a provider needs. The `HasField<Symbol!("<field>")>` trait bound holds; only the
  associated-type projection `<Context as HasField<Symbol!("<field>")>>::Value == <expected>`
  fails. The expected type is read from the failing projection, and the actual type is queried from
  the struct itself (by `DefId`, so a same-named struct in another module is never read).
- **Triggered by:** a `` type mismatch resolving `<Context as HasField<Symbol!("<field>")>>::Value
  == <expected>` `` (`E0271`) that the typed resolver traced through CGP wiring to a `HasField`
  projection — a `check_components!` entry whose provider reads the field with the wrong type. The
  Rust code stays `E0271`.
- **Fix:** change the field's type on the struct to the expected type (or change the provider to
  accept the actual type). The accompanying `note` shows the dependency chain the field is read
  through.
- **Upstream class:** [check-trait failure](https://github.com/contextgeneric/cgp/blob/main/docs/errors/checks/check-trait-failure.md)
  (its projection-mismatch face).

### `CGP-E017` — abstract type has the wrong type

- **Message:** `` [CGP-E017] expected the abstract type `<assoc>` of `<Trait>` on `<Owner>` to be
  `<expected>`, but found `<actual>` `` — reading `associated type` in place of `abstract type` when
  the trait is not a CGP abstract-type component.
- **Means:** the owner supplies one concrete type for an associated type while a provider the wiring
  reaches requires another. The archetype is a CGP
  [abstract type](https://github.com/contextgeneric/cgp/blob/main/docs/concepts/abstract-types.md): a
  context binds `HasErrorType::Error` by wiring `ErrorTypeProviderComponent` to `UseType<String>`,
  while a provider pins the same type with `#[use_type(HasErrorType.{Error = AppError})]`. As with
  [`CGP-E003`](#cgp-e003--field-has-the-wrong-type), the trait bound itself holds — the context *does*
  implement `HasErrorType` — and only the associated-type projection
  `<Ctx as HasErrorType>::Error == AppError` fails. The expected type is read from the failing
  projection, and the actual type by normalizing the projection, so a `UseType<T>` wiring and a
  hand-written `impl HasErrorType for Ctx` are read the same way.
- **Triggered by:** a `` type mismatch resolving `<Ctx as Trait>::Assoc == T` `` (`E0271`) that the
  typed resolver traced through CGP wiring to a projection other than `HasField`'s. The Rust code
  stays `E0271`.
- **Fix:** reconcile the two sides. For a `#[cgp_type]` component a `help` names both ways —
  `` wire `<Marker>` to `UseType<<expected>>` in the wiring for `<Owner>`, or change the provider to
  work with `<actual>` `` — with the component marker recovered from the trait, so the reader is
  pointed at the wiring entry rather than left to find it. An ordinary trait's associated type has no
  such wiring entry, so it carries no `help`.
- **Upstream class:** [check-trait failure](https://github.com/contextgeneric/cgp/blob/main/docs/errors/checks/check-trait-failure.md)
  (its abstract-type projection-mismatch face).

### `CGP-E004`–`CGP-E008` — the duplicate-key wiring-conflict family

These five codes all rewrite the same underlying failure — an `E0119` conflicting-implementation
error on a wiring entry's impls, produced when a `delegate_components!` block wires one key (or
overlapping keys) more than once, or a `cgp_namespace!` block registers one `@`-path twice (whose
conflict lands on the user's own namespace trait rather than `DelegateComponent`). A generated
pair's redundant `IsProviderFor` half is always suppressed — including the pair a duplicate
*provider* definition produces, where the surviving conflict is on the provider trait itself — and
the Rust code stays `E0119`. What differs, and why each has its own code, is the shape of the
collision — and so the message and the fix. In every case an `@`-path key renders in
bare `@a.b.*` notation (no `Path!(…)` wrapper), and the upstream reference is
[conflicting wiring](https://github.com/contextgeneric/cgp/blob/main/docs/errors/wiring/conflicting-wiring.md) with its two namespace faces
[overlapping namespace forwarding](https://github.com/contextgeneric/cgp/blob/main/docs/errors/wiring/namespace-forwarding-conflict.md) and
[namespace override conflict](https://github.com/contextgeneric/cgp/blob/main/docs/errors/wiring/namespace-override-conflict.md).

- **`CGP-E004` — duplicate wiring.** `` [CGP-E004] duplicate wiring for <key> on `<Context>` `` — the
  same key (a component marker, or an `@`-path) mapped twice. **Fix:** remove one of the two entries
  the carets point at.
- **`CGP-E005` — overlapping wiring.** `` [CGP-E005] `<Context>` cannot wire <key> that is already
  set through <source> `` — two distinct but overlapping keys, where one cannot claim what the other
  already covers (a generic entry over a specific one, an `@`-path over a namespace forwarding, or a
  path that is a prefix of another). **Fix:** remove or narrow the overlapping entry. (A *bare* key
  the namespace resolves to a redirect is `CGP-E007` instead.)
- **`CGP-E006` — multiple namespaces.** `` [CGP-E006] only one namespace can be used for each target
  type in `delegate_components!`, but `<Context>` uses both `<A>` and `<B>` `` — two blanket
  forwardings that each cover every key, from joining two namespaces (or a namespace plus a bare-key
  `for` loop, which desugars the same way). **Fix:** join one namespace and inherit the others into
  it, or move a bare `for` key into a path.
- **`CGP-E007` — redirect collision.** `` [CGP-E007] <component> on `<Context>` is redirected to
  `<path>` `` — a direct wiring that collides with a redirect of the same key: an `open` header, an
  explicit `=>` redirect, or a `namespace` that maps the key to a redirected path (recovered by
  normalizing the namespace's `Delegate` for that key). The fix rides in a `help`:
  `` wire the provider `<Provider>` with the key `<path>` ``. **Fix:** wire the direct entry's
  provider under the redirected key rather than the bare key.
- **`CGP-E008` — duplicate redirect.** `` [CGP-E008] duplicate redirect for <component> on
  `<Context>` … `` (naming one redirect target, or both when they differ) — the same key redirected
  more than once: two `open`s or `=>` mappings on a context, or the same `@`-path registered twice
  inside one `cgp_namespace!` block (where the subject is the namespace trait rather than a
  context). **Fix:** keep a single redirect.

### `CGP-E009` — wrapper trait not implemented

- **Message:** `` [CGP-E009] the trait `<Wrapper>` is not implemented for context `<Context>` ``.
- **Means:** a trait that is *not* a CGP consumer trait — so it reads "the trait", not "the consumer
  trait" — cannot be implemented for the context, because a CGP component it depends on fails. Two
  shapes reach this code: a plain **wrapper trait** the programmer wrote (the transfer example's
  `CanHandleApiSend`, which adds a `Send` bound over a CGP consumer supertrait), and a `#[cgp_fn]` /
  `#[blanket_trait]` **capability trait** (`impl<Context> Describe for Context where Self: …`), which
  is a first-class core-CGP capability consumed like a consumer trait but is not a CGP *component*
  (it has no provider trait or `DelegateComponent`). Either way it is the
  [`CGP-E001`](#cgp-e001--consumer-trait-not-implemented) case for a trait that is not itself a CGP
  component.
- **Triggered by:** a failure surfaced *inside* a `impl Wrapper for Context` block (its header, a
  method signature, or a forwarding call) — often as a raw `E0271`/`E0277`/`E0599` that names no CGP
  construct — which the resolver anchors on the enclosing impl and traces through the wrapper's CGP
  consumer supertrait to the root cause; **or** a `#[cgp_fn]` / `#[blanket_trait]` capability the
  context cannot satisfy, reached either by a direct method call (`app.describe()`, an `E0599` the
  [call-site anchor](implementation/typed-resolution-call-site.md) recovers from the call
  expression) or through a `where` bound (`fn f<C: Describe>(…)`, an `E0277` the by-capability
  use-site anchor recovers). Whether the failing trait is a CGP consumer or one of these non-component traits is
  decided by its **fingerprint**: a CGP consumer carries a blanket impl routing to a provider trait,
  while a wrapper has only its concrete impl and a `#[cgp_fn]` capability has a blanket impl over the
  bare context with no provider.
- **Fix:** follow the `root cause:` note to the CGP dependency the trait needs. The dependency tree
  leads with the trait itself, then the capabilities it composes, down to the cause.
- **Upstream class:** [check-trait failure](https://github.com/contextgeneric/cgp/blob/main/docs/errors/checks/check-trait-failure.md)
  (reached through a hand-written wrapper or a `#[cgp_fn]` capability trait).

### `CGP-E010` — wiring never resolves

- **Message:** `` [CGP-E010] the wiring for the consumer trait `<Consumer>` on context `<Context>`
  never resolves — the lookup recurses without terminating ``, with a `help` naming the usual
  cause and the two fixes.
- **Means:** resolving the component's wiring recurses without bottoming out. This almost always
  means the wiring routes the component back to the context itself: a component delegated to
  `UseContext` whose only implementation of the consumer trait *is* that delegation.
- **Triggered by:** an `E0275` overflow whose requirement is a
  `Context: CanUseComponent<Marker, …>` bound. The Rust code stays `E0275`; the note pointing at
  the generated `__Check…` trait is dropped, since the kept caret already covers the check entry.
- **Fix:** wire the component to a real provider, or implement the consumer trait directly on the
  context, so the lookup terminates.
- **Upstream class:** [wiring cycle](https://github.com/contextgeneric/cgp/blob/main/docs/errors/wiring/wiring-cycle.md).

### `CGP-E011` — orphan-rule namespace registration

- **Message:** `` [CGP-E011] cannot register the foreign <key> into the foreign namespace
  `<Namespace>` `` — where `<key>` is `` component `<Marker>` `` or `` path `@…` `` — with a `help`
  naming the ownership-based fix.
- **Means:** the crate is registering wiring into a namespace it does not own, keyed on a component
  (or `@`-path) it does not own either. A registration lowers to `impl Namespace<_> for Key`, and
  Rust's orphan rule rejects a foreign-trait impl with no local type covering it, so with *both* the
  namespace and the key foreign the impl is an orphan.
- **Triggered by:** an `E0210` (or its sibling `E0117`) whose generated impl is a foreign
  [namespace lookup trait](https://github.com/contextgeneric/cgp/blob/main/docs/reference/traits/default_namespace.md)
  implemented for a foreign key — a `#[default_impl(… in Namespace)]` or `#[prefix(… in Namespace)]`
  registration (naming `__Components__`), or a `cgp_namespace!` block re-opening a foreign namespace
  (naming `__Table__`). The Rust code stays `E0210`/`E0117`.
- **Fix (in the `help`):** own one end of the wiring. For a registration, key it on a component your
  crate defines, or register it from the crate that defines the namespace. For a `cgp_namespace!`
  re-open, define a new local namespace that *inherits* the foreign one
  (`cgp_namespace! { new MyNamespace: <Namespace> { … } }`) rather than extending it in place.
- **Upstream class:** [orphan-rule violation](https://github.com/contextgeneric/cgp/blob/main/docs/errors/wiring/orphan-rule.md).

### `CGP-E012` — capability used but not declared

- **Message:** `` [CGP-E012] the capability `<Trait>` is used but not declared as a dependency ``,
  with a `help` naming the fix: `` declare it as a dependency with `#[uses(<Trait>)]` ``.
- **Means:** a `#[cgp_fn]`/`#[cgp_impl]` body calls a CGP capability (a consumer trait, or a
  `#[cgp_fn]`/`#[blanket_trait]` capability) on `self`, but the enclosing definition never declared
  it. The macro lowers the body into a blanket impl over a generated generic context —
  `impl<__Context__> Describe for __Context__ where __Context__: GetName` — so a capability the body
  uses must be a `where` bound on `__Context__`, added with `#[uses(…)]`. Omitted, the method cannot
  resolve on `__Context__`. This also covers a forgotten CGP *consumer* trait used the same way.
- **Triggered by:** an `E0599` "the method `…` exists for reference `&__Context__`, but its trait
  bounds were not satisfied", whose note points at a transitive `HasField` bound. The resolver
  confirms it structurally: the failing call sits in a generated blanket impl whose `Self` is a bare
  type parameter, the called method belongs to a CGP capability trait, and that trait is not among
  the impl's `where` bounds. The Rust code stays `E0599`. Any `[T]: Sized` cascade the unresolved
  return type trails (in an `async` body especially) is left as rustc wrote it: those errors can
  land off the failing expression — on the binding pattern, or a later statement the unresolved type
  flows into — where suppressing them reliably would need type information the emitter cannot obtain
  without risking the suppression of an unrelated error.
- **Fix (in the `help`):** add the capability to the definition's `#[uses(…)]` list (or a
  hand-written `where Self: <Trait>` bound), so it becomes a bound on the generated context.
- **Upstream class:** the post-codegen face of a missing impl-side dependency; closest to the
  [hidden unsatisfied-dependency](https://github.com/contextgeneric/cgp/blob/main/docs/errors/hidden/unsatisfied-dependency.md)
  class, but here the fix is declaring the dependency rather than satisfying it.

### `CGP-E013` — consumer trait used in a provider impl

- **Message:** `` [CGP-E013] `<Consumer>` is a consumer trait, but a `#[cgp_impl]` provider must
  implement its provider trait `<Provider>` ``, with a `help` naming the fix: `` change the impl
  header to target the provider trait: `impl <Provider>` (not `impl <Consumer>`) ``.
- **Means:** a `#[cgp_impl]` provider impl names the component's *consumer* trait in its header where
  the *provider* trait belongs. `#[cgp_impl(new P)] impl AreaCalculator { … }` is the idiomatic
  provider form; writing the consumer trait `CanCalculateArea` there makes the macro generate an
  inside-out impl of the wrong trait and reference a `CanCalculateAreaComponent` marker that does not
  exist, so one mistake yields a burst of cryptic errors (`E0425`/`E0107`/`E0186`/`E0207`) plus a
  downstream check failure, none naming the cause. It generalizes over the component's generic
  parameters: the macro always inserts the context as the leading generic, so the consumer trait is
  given one argument too many whatever its arity.
- **Triggered by:** the `E0107` "trait takes N generic arguments but N+1 supplied" on the impl
  header, confirmed structurally: an impl carrying the `#[cgp_impl]`-inserted `__Context__` generic,
  a concrete provider-struct `Self`, and a user-written header trait that is a CGP *consumer* trait
  (its consumer↔provider fingerprint yields the provider trait to suggest). The sibling
  macro-lowering errors and the downstream `NotAProvider` check re-report are suppressed. The Rust
  code stays `E0107`.
- **Fix (in the `help`):** change the impl header to name the provider trait the component pairs the
  consumer with.
- **Upstream class:** a macro-lowering mistake with no upstream error-catalog class of its own — the
  provider/consumer duality is described in
  [consumer and provider traits](https://github.com/contextgeneric/cgp/blob/main/docs/concepts/consumer-and-provider-traits.md).

### `CGP-E014` — `#[cgp_impl]` on a non-CGP trait

- **Message:** `` [CGP-E014] `#[cgp_impl]` can only implement a CGP component's provider trait, but
  `<Trait>` is not a CGP component ``, with a `help`: `` define `<Trait>` as a component with
  `#[cgp_component]`, or drop `#[cgp_impl]` and write a plain `impl` if it is an ordinary trait ``.
- **Means:** `#[cgp_impl]` is applied to a trait that is not a CGP component at all — neither a
  consumer nor a provider trait — so there is no provider trait to implement. Distinct from
  [`CGP-E013`](#cgp-e013--consumer-trait-used-in-a-provider-impl), where the trait *is* a component
  and only the wrong half was named: here the fix is to make the trait a component or to abandon
  `#[cgp_impl]`, not to name a different trait.
- **Triggered by:** the same `E0107` shape and structural gate as `CGP-E013` (the `__Context__`
  generic, a concrete `Self`, a user-written header trait), but the header trait's fingerprint is
  *neither* a consumer nor a provider trait. The Rust code stays `E0107`.
- **Fix (in the `help`):** annotate the trait with `#[cgp_component]` to make it a component, or use
  an ordinary `impl` if it was never meant to be one.
- **Upstream class:** a macro-lowering mistake with no upstream error-catalog class of its own; see
  [`#[cgp_impl]`](https://github.com/contextgeneric/cgp/blob/main/docs/reference/macros/cgp_impl.md)
  for what the macro requires of its target trait.

### `CGP-E015` — consumer trait in an inner-provider bound

- **Message:** `` [CGP-E015] `<Consumer>` is a consumer trait and cannot bound an inner provider; a
  higher-order provider imports its provider trait `<Provider>` ``, with a `help`: `` name the
  provider trait in the bound, idiomatically `#[use_provider(… : <Provider>)]` (not the consumer
  trait `<Consumer>`) ``.
- **Means:** a higher-order provider's inner-provider bound names the component's *consumer* trait
  where its *provider* trait belongs, most often through `#[use_provider]`.
  [`#[use_provider]`](https://github.com/contextgeneric/cgp/blob/main/docs/reference/attributes/use_provider.md)
  fills the leading context argument in, so `#[use_provider(Inner: CanCalculateArea)]` generates the
  bound `Inner: CanCalculateArea<Self>` — but the consumer trait takes no context parameter, so it is
  given one argument too many. It is the inner-bound sibling of
  [`CGP-E013`](#cgp-e013--consumer-trait-used-in-a-provider-impl) (the same consumer/provider
  confusion, in the impl header).
- **Triggered by:** the `E0107` on the consumer trait in the bound, confirmed structurally: an inner
  bound of a `#[cgp_impl]` provider impl whose trait is a CGP *consumer* trait (its consumer↔provider
  fingerprint yields the provider trait to suggest). The `E0308` body cascade the malformed bound
  trails — recognizable by its mention of the generated `__Context__` — is suppressed. The Rust code
  stays `E0107`.
- **Fix (in the `help`):** name the provider trait in the bound, idiomatically through
  `#[use_provider]`.
- **Upstream class:** a macro-lowering mistake with no upstream error-catalog class of its own — see
  [higher-order providers](https://github.com/contextgeneric/cgp/blob/main/docs/concepts/higher-order-providers.md).

### `CGP-E016` — inner provider used but not imported

- **Message:** `` [CGP-E016] the inner provider `<Inner>` is used but not imported ``, with a `help`:
  `` import it with `#[use_provider(<Inner>: <ProviderTrait>)]` ``.
- **Means:** a higher-order provider's body calls an inner provider as an associated function —
  `<Inner>::method(self)` — that it never imported, so the inner parameter carries no provider-trait
  bound and the call cannot resolve. It is the higher-order-provider counterpart of
  [`CGP-E012`](#cgp-e012--capability-used-but-not-declared): a used-but-undeclared dependency, here an
  inner provider imported with `#[use_provider]` rather than a `#[uses]` capability.
- **Triggered by:** an `E0599` "no associated function … found for type parameter `<Inner>`" whose
  help names the "type parameter is bounded by the trait" shape. The resolver confirms it
  structurally: the failing call is `Param::method(…)` on a generic parameter of an enclosing
  provider-trait impl, the method belongs to a CGP provider trait, and the parameter is not bounded
  by it. rustc's own suggestion leaks the generated `__Context__` and offers the *consumer* trait as
  a bound (the wrong fix); the rewrite names the inner provider and the `#[use_provider]` fix instead.
  The Rust code stays `E0599`.
- **Fix (in the `help`):** import the inner provider with `#[use_provider(<Inner>: <ProviderTrait>)]`,
  which supplies the leading context argument the provider trait needs.
- **Upstream class:** a macro-lowering mistake with no upstream error-catalog class of its own — see
  [higher-order providers](https://github.com/contextgeneric/cgp/blob/main/docs/concepts/higher-order-providers.md)
  and [`#[use_provider]`](https://github.com/contextgeneric/cgp/blob/main/docs/reference/attributes/use_provider.md).

## Dependency-tree entry codes (`CGP-E1xx`)

The `CGP-E1xx` range codes the entries of a `root cause:` note's dependency tree — one code per
distinct rendering template, so a reader can name a chain step (and a downstream tool can key on it)
just as with a main message. The code rides at the start of each tree entry:

```text
= note: root cause: [CGP-E106] missing field `name` on `App`
        this is required through the dependency chain:
          [CGP-E101] consumer trait impl `CanGreet` for context `App`
          └─ [CGP-E102] provider trait impl `Greeter` with context `App` for provider `GreetHello`
            └─ [CGP-E105] trait impl `HasName` for `App`
              └─ [CGP-E106] missing field `name` on `App`
```

A tree entry that merely *passes a non-CGP message through* in rustc's own phrasing — the
ordinary-bound restatement `` the trait bound `f64: Eq` is not satisfied `` — is uncoded, while an
entry the tool *rewrote* into its own template (including the general `` trait impl `Trait` for
`Type` ``) is coded. The `root cause:` note lead — the summary above the tree — carries a code of its
own, from the separate `CGP-E2xx` range (see below).

The codes divide into the inner chain-node templates and the terminal root-cause leaves.

- **`CGP-E101` — consumer trait impl.** `` consumer trait impl `<Trait>` for context `<Ctx>` `` — a
  hop through the context's own consumer-trait impl (a `CanUseComponent` step).
- **`CGP-E102` — provider trait impl.** `` provider trait impl `<Trait>` with context `<Ctx>` for
  provider `<Provider>` `` — a hop through a provider's provider-trait impl (an `IsProviderFor` step).
- **`CGP-E103` — retired.** This code named a mid-chain `HasField` accessor hop, but a `HasField`
  obligation is always the chain's terminal root-cause leaf (coded `CGP-E106`/`CGP-E108`/`CGP-E109`),
  never an interior hop, so it was never emitted and has been removed. The number is left unused
  rather than reassigned, so the other codes stay stable.
- **`CGP-E104` — redirect lookup.** `` redirect lookup to `@…` in `<Ctx>` `` — a hop through a
  namespace or `open` `RedirectLookup`. Two lookups along the same route for different dispatch keys
  render this same text but are distinct nodes in the dependency graph (the key is part of a node's
  identity), so each keeps its own branch and leaf.
- **`CGP-E105` — trait impl (general).** `` trait impl `<Trait>` for `<Type>` `` — a hop through any
  other trait: a user capability trait, or an ordinary bound restated as an impl. This is the
  "rewritten non-CGP" form that is coded even though the trait itself may not be a CGP construct.
- **`CGP-E106` — missing field (leaf).** `` missing field `<f>` on `<T>` `` — the chain bottoms out
  on a context field that is genuinely absent.
- **`CGP-E107` — missing delegate entry (leaf).** `` context `<Ctx>` does not contain any delegate
  entry for `<key>` `` — the context wires no provider for a component, or terminates no namespace
  path (the `<key>` is a component marker or an `@`-path).
- **`CGP-E108` — unimplemented accessor (leaf).** `` accessor trait `HasField` with field `<f>` is not
  implemented for `<T>` `` — the struct carries the field but has not derived `HasField` for it (the
  fix, a `#[derive(HasField)]`, rides in a separate `help`). Several such fields on *one* struct are
  one mistake — the derive emits an impl per field — so they coalesce into a single root cause under
  the same code, `` accessor trait `HasField` is not implemented for the fields `<f>` and `<g>` of
  `<T>` ``, over one merged tree whose branches still end at the per-field leaves.
- **`CGP-E109` — field type mismatch (leaf).** `` field `<f>` on `<T>` has type `<actual>`, but
  `<expected>` is required `` — the field is present and derived but has the wrong type (the leaf face
  of the `CGP-E003` main message).
- **`CGP-E110` — missing dispatch entry (leaf).** `` provider `<T>` does not contain any delegate
  entry for `<key>` `` — the chain bottoms out on a *non-context* delegation table missing a key: an
  [aggregate provider](https://github.com/contextgeneric/cgp/blob/main/docs/concepts/aggregate-providers.md) missing a component wiring, or
  a [`UseDelegate`/`UseInputDelegate`](https://github.com/contextgeneric/cgp/blob/main/docs/reference/providers/use_delegate.md) dispatch
  table missing a branch for the type it dispatches on (a `Code` fragment or an `Input` value's type).
  The sibling of `CGP-E107` for a provider table rather than the context: the fix is to add the entry
  to *that provider*, or to feed the stage a type the table already covers (the shape a handler
  pipeline hits when a stage's output type is not one a later stage's input dispatcher handles).
- **`CGP-E111` — not a provider (leaf).** `` the provider trait `<T>` is not implemented for `<X>` ``
  — the chain bottoms out on a type wired where a *provider* was expected that does not implement the
  provider trait at all. The mistake is putting a non-provider (often a request or value type) into a
  provider slot — e.g. `UseBasicAuth<QueryBalanceRequest>` with the endpoint handler omitted, so the
  request type sits where an `ApiHandler` belongs. Distinct from `CGP-E110`: the owner is not a table
  missing one entry, so the fix is to use an actual provider (wrap it in the handler), not to add a
  wiring entry.
- **`CGP-E112` — associated type mismatch (leaf).** `` abstract type `<assoc>` of `<Trait>` on
  `<Owner>` is `<actual>`, but `<expected>` is required `` — the chain bottoms out on an associated
  type the owner supplies differently from what a provider requires (the leaf face of the
  `CGP-E017` main message). It reads `associated type` in place of `abstract type` when the trait is
  not a CGP abstract-type component. The non-`HasField` sibling of `CGP-E109`.

## Root-cause lead codes (`CGP-E2xx`)

The `root cause:` line that heads a note — the plain-sentence summary above the dependency tree —
also carries a code. It **reuses the terminal leaf's `CGP-E1xx` code** where the leaf has one, so the
lead and the tree's terminal show the same code (`` root cause: [CGP-E106] missing field `name` on
`App` `` over a tree that ends in `` [CGP-E106] missing field `name` on `App` ``). The `CGP-E2xx`
range exists for the one case that needs a code of its own: a leaf that is an uncoded pass-through
bound, whose lead still names a classified root cause.

- **`CGP-E201` — ordinary-bound root cause.** `` root cause: the trait bound `<S: Trait>` is not
  satisfied `` — the failure bottoms out on an ordinary (non-CGP) trait bound. The terminal tree
  entry passes the bound through uncoded, but the `root cause:` lead takes this code so every root
  cause the tool states is tagged.

## Uncoded rewrites

These rewrites improve a diagnostic's readability without classifying its main message, so they
carry no code. They are listed here so their codelessness is a recorded decision.

**Root-cause notes and dependency chains** replace a resolved diagnostic's sub-messages with one
`root cause: …` note per recovered cause (and a `#[derive(HasField)]` `help` where that is the
fix). They accompany a coded main message or a kept rustc one; the note itself is never coded.
[Typed root-cause resolution](implementation/typed-root-cause-resolution.md) owns them.

**Obligation-note renaming** rewrites the `required for … to implement …` chain notes of an
unresolved (fallback) diagnostic to name the consumer and provider traits instead of the wiring
traits. The rename runs in the driver;
[`rewrite/message.rs`](../crates/cargo-cgp-error-processing/src/rewrite/message.rs) owns the string
transform.

**Prefix stripping** removes the CGP module paths rustc prints in front of CGP type names, so
`cgp::prelude::Chars` becomes `Chars`. It only shortens a name;
[`strip_cgp_prefixes`](../crates/cargo-cgp-error-processing/src/postprocess/strip_prefixes.rs) owns it.

**`Symbol!` resugaring** reverses a `Symbol!` type expansion back to its surface form, so
`Symbol<2, Chars<'x', Chars<'y', Nil>>>` becomes `Symbol!("xy")`. It restores the syntax the
programmer wrote and nothing more;
[`resugar_symbol`](../crates/cargo-cgp-error-processing/src/postprocess/resugar_symbol.rs) owns it,
and [Resugaring](implementation/resugaring.md) explains this and every other construct in full.

**`Product!` / `Sum!` resugaring** reverses a type-level list expansion back to its surface form, so
`Cons<u64, Cons<String, Nil>>` becomes `Product![u64, String]` and an `Either`/`Void` spine becomes
`Sum![…]`. A list whose elements are all named fields folds one step further to a `Struct! { … }` or
`Enum! { … }` — presentation-only forms, not real CGP macros, chosen because a record reads far better
than a chain of `Field` cells.
[`resugar_lists`](../crates/cargo-cgp-error-processing/src/postprocess/resugar_list.rs) owns the text
form and the driver's `render_ty` the typed one.

**`Path!` resugaring** reverses a `Path!` type expansion back to its surface form, so
`PathCons<Symbol!("app"), PathCons<GreeterComponent, Nil>>` becomes `Path!(@app.GreeterComponent)`.
It restores the syntax the programmer wrote, save for one readable extension: an open-ended path whose
spine ends in a generic parameter rustc prints as `_` gets a trailing `.*` wildcard segment
(`PathCons<Symbol!("foo"), PathCons<Symbol!("bar"), _>>` becomes `Path!(@foo.bar.*)`), which is not
real `Path!` syntax but reads far better than the raw spine.
[`resugar_path`](../crates/cargo-cgp-error-processing/src/postprocess/resugar_path.rs) owns it.

**Missing-field clause rewriting** turns an unmet `` `HasField<Symbol!("name")>` `` clause inside a
sub-message into `` missing field `name` on `Context` `` (or the `#[derive(HasField)]` form when
the context implements `HasField` for nothing);
[`rewrite_missing_fields`](../crates/cargo-cgp-error-processing/src/postprocess/missing_field.rs)
owns it.

**Method-probe advice removal** drops rustc's "this is an associated function, not a method"
framing and "use associated function syntax instead" suggestion from a consumer-method `E0599` the
resolver declined — both artifacts of CGP's `self`-less provider methods, the second actively
wrong — so the unmet wiring bound the diagnostic also names leads. It only removes noise, adding no
classification;
[`is_method_probe_advice_text`](../crates/cargo-cgp-error-processing/src/signals.rs) recognizes the
phrasings and the driver's emitter applies the drop.

## Maintaining this catalog

This catalog is bound by the same synchronization rule as the rest of the knowledge base
([docs/AGENTS.md](AGENTS.md)): the codes are defined in the code, so this document must track them.
The constants live in the [`code`](../crates/cargo-cgp-error-processing/src/code.rs) module of the
error-processing crate, and are stamped by the main-message rewrites — all rustc-free, in that
same crate. The text forms are stamped by
[`rewrite_trait_bound` and `rewrite_wiring_overflow`](../crates/cargo-cgp-error-processing/src/rewrite/message.rs),
the typed check-failure form by
[`plan_resolved`](../crates/cargo-cgp-error-processing/src/diagnosis/plan.rs)'s
`categorized_header` (fed from the resolved failure), and the `CGP-E004`–`CGP-E008` conflict forms by
[`plan_wiring_conflict`](../crates/cargo-cgp-error-processing/src/diagnosis/wiring.rs) (fed from the
conflict the driver's `resolve::conflict` classifier recovers, one code per `WiringConflict` shape),
and the `CGP-E011` orphan form by
[`plan_orphan_conflict`](../crates/cargo-cgp-error-processing/src/diagnosis/orphan.rs) (fed from the
`OrphanConflict` the driver's `resolve::orphan` classifier recovers).
The `CGP-E1xx` dependency-tree entry codes are stamped when a node renders: the inner chain nodes by
the structured [`DepNode`](../crates/cargo-cgp-error-processing/src/diagnosis/node.rs) variants (the
driver's [`resolve::label`](../crates/cargo-cgp-driver/src/resolve/label) picks the variant from the
trait kind), and the terminal leaf by the rustc-free
[`dependency_tree_leaf`](../crates/cargo-cgp-error-processing/src/diagnosis/wording/lead.rs) (which
prefixes `dependency_leaf_code`, or leaves a pass-through bound bare). The `CGP-E2xx` root-cause lead
code is stamped by
[`cause_note`](../crates/cargo-cgp-error-processing/src/diagnosis/wording/note.rs) via
`root_cause_code`, which reuses `dependency_leaf_code` and only falls back to a `CGP-E2xx` constant for
an uncoded leaf. When a new main-message class is recognized, or a new tree-entry or root-cause
template is added, assign it the next number in the matching range (`CGP-E0xx` for a headline,
`CGP-E1xx` for a tree entry, `CGP-E2xx` for a root-cause lead the leaf codes cannot cover), add the
constant, and register it here in the same change. When a rewrite does not classify its message — a
kept rustc header, or a pass-through tree entry — add it to [uncoded rewrites](#uncoded-rewrites)
instead, or leave the tree entry bare — do not spend a code on it.
