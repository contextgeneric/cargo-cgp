# CGP Error Codes

`cargo-cgp` assigns a stable error code to every diagnostic whose message it fully rewrites into a
CGP-specific error, and this document is the catalog of those codes. A code names one recognized
class of CGP mistake — what the message means, what triggers it, and how to fix it — so that a
reader who sees a code in the tool's output can look it up here, and so that the tool's own tests and
future JSON output can refer to a class by a short, stable identifier rather than by its prose.

## The scheme, and why it looks unlike a Rust code

A CGP error code is the letters `CGP` followed by four digits — `CGP0001`, `CGP0002`, and so on —
shown in square brackets as a prefix on the rewritten message: `[CGP0001] missing field ...`. The
`CGP` prefix is deliberate. Rust's own codes are the letter `E` and four digits (`E0277`, `E0599`),
and `cargo-cgp` output routinely carries *both* — a rewritten CGP message can sit inside a diagnostic
that rustc still headlines with `error[E0277]:`. Keeping the CGP scheme visually distinct means a
reader never mistakes one for the other: `E0277` is a Rust code you explain with `rustc --explain`,
while `[CGP0001]` is a CGP code you look up here.

The code is placed where it reads as a tag on the message it belongs to, not on the diagnostic's
`error`/`help` label. rustc attaches `E0277` to the level word (`error[E0277]:`) because the code
classifies the whole diagnostic. A CGP code classifies only the one message `cargo-cgp` rewrote,
which is often a `help:` line nested inside a larger rustc diagnostic, so the code leads that
message's text instead — `help: [CGP0001] missing field ...`. This also keeps the code greppable: a
reader confirming a class need only search the output for `CGP0001`.

## When a code is assigned

A code is assigned only when `cargo-cgp` **replaces a whole message** with a CGP-authored one — a
rewrite that reclassifies the error into a CGP concept with its own meaning and fix. These are the
rewrites worth looking up, because the new message says something the original did not, and the code
is the handle for that new meaning.

The tool's other rewrites are **cosmetic and partial**, and they carry no code by design. They clean
up the names inside a message rustc already framed correctly — stripping a module path, resugaring a
type-level encoding, renaming an internal wiring trait to the trait a reader thinks in — without
changing what class of error it is. Tagging these would be noise: there is nothing new to look up,
and a code on every cleaned-up name would bury the codes that matter. The
[partial rewrites](#partial-rewrites-no-code) section below records each one, so the absence of a
code on them is documented rather than merely implied.

## Codes

The catalog today holds the two classes `cargo-cgp` fully rewrites, both produced by the
missing-field preprocessor
([`extract_missing_fields`](../crates/cargo-cgp-error-processing/src/preprocess/missing_field.rs)).
Each entry gives the rewritten message, the mistake behind it, the fix, and the upstream
[CGP error catalog](../../cgp/docs/errors/README.md) class it recognizes.

### `CGP0001` — missing field

- **Message:** `` [CGP0001] missing field `<field>` in `<Context>` ``
- **Means:** a provider needs the context to hold a field the getter reads (through `HasField`), and
  the context implements `HasField` for *some* field but not this one — so exactly this one field is
  missing.
- **Fix:** add the named field to the context struct.
- **Rewritten from:** rustc's `` the trait `HasField<Symbol!("<field>")>` is not implemented for
  `<Context>` `` bound, with the "similar impl" landmark that shows the context does implement
  `HasField` for other fields. That landmark is the tell that distinguishes this from `CGP0002`.
- **Upstream class:** [check-trait failure](../../cgp/docs/errors/checks/check-trait-failure.md).

### `CGP0002` — missing `#[derive(HasField)]`

- **Message:** `` [CGP0002] `#[derive(HasField)]` is required to access field `<field>` in
  `<Context>` ``
- **Means:** the same unmet `HasField` bound, but the context implements `HasField` for *no* field at
  all — so the whole derive is absent, and adding one field at a time is the wrong fix.
- **Fix:** add `#[derive(HasField)]` to the context struct (then ensure it declares the field).
- **Rewritten from:** the same `` `HasField<Symbol!("<field>")>` is not implemented `` bound, but
  with *no* similar-impl landmark anywhere in the diagnostic — its absence is what says no `HasField`
  impls exist.
- **Upstream class:** [check-trait failure](../../cgp/docs/errors/checks/check-trait-failure.md).

One degenerate input is reported as `CGP0002` even though a derive is present: a context that derives
`HasField` but declares no fields. This is correct, not a misclassification — `#[derive(HasField)]`
emits one impl per field, so on a fieldless struct it emits nothing, indistinguishable from no derive
at all. The [error-processing document](implementation/error-processing.md) explains why the two
cases cannot be told apart and need not be.

## Partial rewrites (no code)

These rewrites improve a diagnostic's readability without reclassifying it, so they carry no code.
They are listed here so their codelessness is a recorded decision.

**Prefix stripping** removes the CGP module paths rustc prints in front of CGP type names, so
`cgp::prelude::Chars` becomes `Chars`. It only shortens a name;
[`strip_cgp_prefixes`](../crates/cargo-cgp-error-processing/src/preprocess/strip_prefixes.rs) owns it.

**`Symbol!` resugaring** reverses a `Symbol!` type expansion back to its surface form, so
`Symbol<2, Chars<'x', Chars<'y', Nil>>>` becomes `Symbol!("xy")`. It restores the syntax the
programmer wrote and nothing more;
[`resugar_symbol`](../crates/cargo-cgp-error-processing/src/preprocess/resugar_symbol.rs) owns it.

**Wiring-message renaming** rewrites the internal wiring traits a CGP failure is reported through into
the consumer and provider traits a reader thinks in — `` `Person: CanUseComponent<GreeterComponent>` ``
becomes `` `Person: CanGreet` ``, and the obligation-chain notes are renamed to name the provider and
consumer traits. This is a rename of the types inside a bound rustc already framed as a trait-bound
error, not a new message, so the diagnostic keeps its Rust code (typically `E0277`) and gains no CGP
code. It runs in the driver, not the front-end processing pipeline, and is documented in
[The driver](implementation/driver.md#naming-the-traits-behind-a-component-marker); the compiler-free
rewrite itself lives in
[`rewrite.rs`](../crates/cargo-cgp-error-processing/src/rewrite.rs).

## Maintaining this catalog

This catalog is bound by the same synchronization rule as the rest of the knowledge base
([docs/AGENTS.md](AGENTS.md)): the codes are defined in the code, so this document must track them.
The codes live as constants on
[`CgpDiagnosticDetail`](../crates/cargo-cgp-error-processing/src/diagnostic.rs) (its `code` method maps
a recognized detail to its code), and each is emitted by the preprocessor that recognizes the class.
When a preprocessor learns to fully rewrite a new class, assign it the next `CGP` number, add the
constant and its `code` arm, and register the class here in the same change. When a rewrite is only
cosmetic, add it to [partial rewrites](#partial-rewrites-no-code) instead — do not spend a code on it.
