# CGP Error Codes

`cargo-cgp` assigns a stable error code to every main message it rewrites into a CGP-specific one,
and this document is the catalog of those codes. A code names one recognized class of CGP mistake —
what the message means, what triggers it, and how to fix it — so that a reader who sees a code in
the tool's output can look it up here, and so that the tool's own tests and future JSON output can
refer to a class by a short, stable identifier rather than by its prose.

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
at a consumer-call use site) stays in the header, the trailing `rustc --explain` line stays
meaningful, and the CGP code rides inside the message text where it reads as a tag on the sentence
it classifies. This also keeps the code greppable: a reader confirming a class need only search the
output for `CGP-E001`.

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

The catalog today holds the two classes the driver's emitter recognizes in a main message. Each
entry gives the rewritten message, the mistake behind it, the fix, and the upstream
[CGP error catalog](../../cgp/docs/errors/README.md) class it recognizes. The root cause behind
either message is carried by the accompanying `note`s — one per recovered cause, each opening
`root cause: …` over its dependency chain (see
[Typed root-cause resolution](implementation/typed-root-cause-resolution.md)).

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
[`rewrite.rs`](../crates/cargo-cgp-error-processing/src/rewrite.rs) owns the string transform.

**Prefix stripping** removes the CGP module paths rustc prints in front of CGP type names, so
`cgp::prelude::Chars` becomes `Chars`. It only shortens a name;
[`strip_cgp_prefixes`](../crates/cargo-cgp-error-processing/src/preprocess/strip_prefixes.rs) owns it.

**`Symbol!` resugaring** reverses a `Symbol!` type expansion back to its surface form, so
`Symbol<2, Chars<'x', Chars<'y', Nil>>>` becomes `Symbol!("xy")`. It restores the syntax the
programmer wrote and nothing more;
[`resugar_symbol`](../crates/cargo-cgp-error-processing/src/preprocess/resugar_symbol.rs) owns it.

**Missing-field clause rewriting** turns an unmet `` `HasField<Symbol!("name")>` `` clause inside a
sub-message into `` missing field `name` on `Context` `` (or the `#[derive(HasField)]` form when
the context implements `HasField` for nothing);
[`extract_missing_fields`](../crates/cargo-cgp-error-processing/src/preprocess/missing_field.rs)
owns it.

## Maintaining this catalog

This catalog is bound by the same synchronization rule as the rest of the knowledge base
([docs/AGENTS.md](AGENTS.md)): the codes are defined in the code, so this document must track them.
The constants live in the [`code`](../crates/cargo-cgp-error-processing/src/code.rs) module of the
error-processing crate, and are stamped by the two main-message rewrites — the text form in
[`rewrite_trait_bound`](../crates/cargo-cgp-error-processing/src/rewrite.rs) and the typed form in
the driver's [`emitter`](../crates/cargo-cgp-driver/src/emitter.rs) (`categorized_header`). When a
new class of main message is recognized and rewritten, assign it the next `CGP-E` number, add the
constant, and register the class here in the same change. When a rewrite does not classify the main
message, add it to [uncoded rewrites](#uncoded-rewrites) instead — do not spend a code on it.
