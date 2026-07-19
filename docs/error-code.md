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

The catalog today holds the classes the driver's emitter recognizes in a main message, in two
groups. The **check-failure** codes `CGP-E001`–`CGP-E003` and `CGP-E009` come from the typed
resolver; each carries the root cause in the accompanying `note`s — one per recovered cause, each
opening `root cause: …` over its dependency chain (see
[Typed root-cause resolution](implementation/typed-root-cause-resolution.md)) — except `CGP-E003`,
whose main message already states its cause in full and so carries the chain alone. (`CGP-E009` is
the check-failure code for a hand-written wrapper trait rather than a CGP consumer; it is grouped
with `CGP-E001`–`CGP-E003` below.) The **structural
wiring-conflict** codes `CGP-E004`–`CGP-E008` are different in kind: they come from the duplicate-key
conflict classifier, carry no root-cause note, and instead keep rustc's two carets, which already
point at the two colliding entries. They are five separate codes because each rewrites the `E0119`
into a distinct message form with its own fix. Each entry below gives the rewritten message, the
mistake behind it, the fix, and the upstream
[CGP error catalog](../../cgp/docs/errors/README.md) class it recognizes.

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
- **Upstream class:** [check-trait failure](../../cgp/docs/errors/checks/check-trait-failure.md).

### `CGP-E002` — provider trait not implemented

- **Message:** `` [CGP-E002] the provider trait `<Provider trait>` with context `<Context>` is not
  implemented for provider `<Provider>` ``.
- **Means:** a specific provider fails to implement its provider trait for the context — its
  impl-side `where`-clause dependencies do not hold — as asserted by `IsProviderFor`.
- **Triggered by:** an unsatisfied `Provider: IsProviderFor<Marker, Context, Params>` bound — a
  `#[check_providers(...)]` assertion, or a wiring step (such as a namespace `RedirectLookup`)
  whose provider-side failure rustc chose as the primary error.
- **Fix:** follow the `root cause:` note(s) to the dependency the provider is missing.
- **Upstream class:** [check-trait failure](../../cgp/docs/errors/checks/check-trait-failure.md)
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
- **Upstream class:** [check-trait failure](../../cgp/docs/errors/checks/check-trait-failure.md)
  (its projection-mismatch face).

### `CGP-E004`–`CGP-E008` — the duplicate-key wiring-conflict family

These five codes all rewrite the same underlying failure — an `E0119` conflicting-implementation
error on a CGP `DelegateComponent` impl, produced when a `delegate_components!` block wires one key
(or overlapping keys) more than once. The pair's redundant `IsProviderFor` half is always
suppressed, and the Rust code stays `E0119`. What differs, and why each has its own code, is the
shape of the collision — and so the message and the fix. In every case an `@`-path key renders in
bare `@a.b.*` notation (no `Path!(…)` wrapper), and the upstream reference is
[conflicting wiring](../../cgp/docs/errors/wiring/conflicting-wiring.md) with its two namespace faces
[overlapping namespace forwarding](../../cgp/docs/errors/wiring/namespace-forwarding-conflict.md) and
[namespace override conflict](../../cgp/docs/errors/wiring/namespace-override-conflict.md).

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
  more than once. **Fix:** keep a single redirect.

### `CGP-E009` — wrapper trait not implemented

- **Message:** `` [CGP-E009] the trait `<Wrapper>` is not implemented for context `<Context>` ``.
- **Means:** a hand-written trait implemented directly on the context — *not* a CGP consumer trait,
  but a plain wrapper the programmer wrote (the transfer example's `CanHandleApiSend`, which adds a
  `Send` bound over a CGP consumer supertrait) — cannot be implemented, because a CGP component it
  depends on fails. It is the [`CGP-E001`](#cgp-e001--consumer-trait-not-implemented) case for a
  trait that is not itself a CGP component, so it reads "the trait", not "the consumer trait".
- **Triggered by:** a failure surfaced *inside* a `impl Wrapper for Context` block (its header, a
  method signature, or a forwarding call) — often as a raw `E0271`/`E0277`/`E0599` that names no CGP
  construct — which the resolver anchors on the enclosing impl and traces through the wrapper's CGP
  consumer supertrait to the root cause. Whether the impl's trait is a CGP consumer or a plain
  wrapper is decided by its **fingerprint**: a CGP consumer carries a blanket impl routing to a
  provider trait, a wrapper has only its concrete impl.
- **Fix:** follow the `root cause:` note to the CGP dependency the wrapper's supertrait needs. The
  dependency tree leads with the wrapper itself, then its CGP supertrait, down to the cause.
- **Upstream class:** [check-trait failure](../../cgp/docs/errors/checks/check-trait-failure.md)
  (reached through a hand-written wrapper trait).

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
- **`CGP-E103` — field trait impl.** `` field trait impl `HasField` with field `<f>` for `<T>` `` — a
  hop through a `HasField` accessor impl that is *not* the terminal leaf (rare: a `HasField` bound is
  almost always the chain's terminal, coded `CGP-E106`/`CGP-E108`/`CGP-E109` instead).
- **`CGP-E104` — redirect lookup.** `` redirect lookup to `@…` in `<Ctx>` `` — a hop through a
  namespace or `open` `RedirectLookup`.
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
  fix, a `#[derive(HasField)]`, rides in a separate `help`).
- **`CGP-E109` — field type mismatch (leaf).** `` field `<f>` on `<T>` has type `<actual>`, but
  `<expected>` is required `` — the field is present and derived but has the wrong type (the leaf face
  of the `CGP-E003` main message).

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
[`resugar_symbol`](../crates/cargo-cgp-error-processing/src/postprocess/resugar_symbol.rs) owns it.

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

## Maintaining this catalog

This catalog is bound by the same synchronization rule as the rest of the knowledge base
([docs/AGENTS.md](AGENTS.md)): the codes are defined in the code, so this document must track them.
The constants live in the [`code`](../crates/cargo-cgp-error-processing/src/code.rs) module of the
error-processing crate, and are stamped by the main-message rewrites — all rustc-free, in that
same crate. The text form is stamped by
[`rewrite_trait_bound`](../crates/cargo-cgp-error-processing/src/rewrite/message.rs), the typed
check-failure form by [`plan_resolved`](../crates/cargo-cgp-error-processing/src/diagnosis/plan.rs)'s
`categorized_header` (fed from the resolved failure), and the `CGP-E004`–`CGP-E008` conflict forms by
[`plan_wiring_conflict`](../crates/cargo-cgp-error-processing/src/diagnosis/wiring.rs) (fed from the
conflict the driver's `resolve::conflict` classifier recovers, one code per `WiringConflict` shape).
The `CGP-E1xx` dependency-tree entry codes are stamped at tree-construction time: the inner chain
nodes by the driver's [`resolve::label`](../crates/cargo-cgp-driver/src/resolve/label.rs) (which
chooses a template from the trait kind and prefixes its code), and the terminal leaf by the rustc-free
[`dependency_tree_leaf`](../crates/cargo-cgp-error-processing/src/diagnosis/wording.rs) (which prefixes
`dependency_leaf_code`, or leaves a pass-through bound bare). The `CGP-E2xx` root-cause lead code is
stamped by [`cause_note`](../crates/cargo-cgp-error-processing/src/diagnosis/wording.rs) via
`root_cause_code`, which reuses `dependency_leaf_code` and only falls back to a `CGP-E2xx` constant for
an uncoded leaf. When a new main-message class is recognized, or a new tree-entry or root-cause
template is added, assign it the next number in the matching range (`CGP-E0xx` for a headline,
`CGP-E1xx` for a tree entry, `CGP-E2xx` for a root-cause lead the leaf codes cannot cover), add the
constant, and register it here in the same change. When a rewrite does not classify its message — a
kept rustc header, or a pass-through tree entry — add it to [uncoded rewrites](#uncoded-rewrites)
instead, or leave the tree entry bare — do not spend a code on it.
