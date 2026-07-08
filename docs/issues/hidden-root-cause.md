# Hidden root cause

This document catalogs the cases where `cargo-cgp` does not emit enough information to identify the
root cause of an error — the cases where the cause cannot be recovered from the output alone, no
matter how that output is later processed. It is the tool's most important measure of correctness,
because everything else the tool might do to a diagnostic assumes the cause is present to work with.

The test this document applies is *sufficiency*, not readability. `cargo-cgp`'s role is to produce
output from which a downstream consumer — a formatter, an IDE, or an AI agent — can recover the root
cause; how that output is spelled or laid out is a separate concern, handled in
[usability](usability.md). An issue belongs here only when the root cause is genuinely absent from,
or unrecoverable from, `cargo-cgp`'s output, so that no downstream processing of the text could
reconstruct it — and only when a fixture under
[`tests/ui/hidden-root-cause/`](../../tests/ui/hidden-root-cause) reproduces it. These are the cases
that justify the tool's compiler-internal foothold: where the ordinary text output has lost
information, only a tool reading the compiler's own state can put it back.

## A truncated type drops characters from the field name

The one reproduced case is a field name that the compiler prints with a character missing, so the
name cannot be read back from the diagnostic at all. When a provider needs a field the context lacks
and the context has a *near-miss* field, rustc reports the unmet bound through its two-line "similar
impl exists" hint — "the trait `HasField<…>` is not implemented … but trait `HasField<…>` is
implemented for it" — and in that hint it abbreviates one character of each type-level string as
`_`. In [`hidden-root-cause/base_area_1.rs`](../../tests/ui/hidden-root-cause/base_area_1.rs), a
`Rectangle` that has `width` but not `height` produces
([`.stderr`](../../tests/ui/hidden-root-cause/base_area_1.stderr)):

```
HasField<Symbol<6, Chars<'h', Chars<'e', Chars<'i', Chars<'g', Chars<_, Chars<'t', Nil>>>>>>>>
```

The fifth character is gone. A consumer reading the diagnostic alone sees `h, e, i, g, _, t` and
cannot know the field is `height` rather than any other name that fits — the information is not
merely encoded, it is absent from the text. This is what separates it from the
[encoded-but-readable field name](usability.md#the-field-name-is-an-encoded-type-level-string), a
usability issue where the full string is present and can be decoded; here the string is incomplete.
The abbreviation is specific to the two-line similar-impl hint shape, which is why the other fixtures
— reporting the same class of mistake through the collapsed list hint — spell their symbols out in
full and are usability problems instead.

This is the case that most directly justifies why `cargo-cgp` embeds the compiler rather than
post-processing text. The full `Symbol` type is intact in the compiler's interned representation, so
the driver can read the exact field name and emit it even though rustc's own textual output elides
it. Passing the text through — what the tool does today — is here provably not enough; closing the
gap requires the foothold.

## What is deliberately not here

Two absences are intentional, so that the list stays a record of reproduced problems rather than
imagined ones. The archetypal hidden failure — the compiler suppressing a cause entirely, reporting
only that a method's bounds are unsatisfied (`E0599`) without naming the failed dependency — is
**not** listed, because `cargo-cgp` already defeats it: the `-Znext-solver` injection descends to the
real bound, so the same wiring mistake now surfaces the unmet `HasField` and even an "add
`#[derive(HasField)]`" hint. The fixture that once showed the hidden form,
[`usability/unsatisfied_dependency.rs`](../../tests/ui/usability/unsatisfied_dependency.rs), now
demonstrates that recovered cause and has moved to [usability](usability.md), where its remaining
problem — a misleading, verbose presentation — actually lies. Should a wiring mistake be found whose
cause the next-gen solver still cannot surface, it belongs here with a fixture that proves it. The
upstream catalog's [hidden-versus-surfaced axis](../../../cgp/docs/errors/README.md#the-central-axis-hidden-versus-surfaced)
is the same distinction from the CGP side and is the reference for judging which classes carry their
cause and which suppress it.

The second absence is by scope: plain Rust or Cargo diagnostics that have nothing to do with CGP are
not `cargo-cgp`'s to recover and are not tracked here.
